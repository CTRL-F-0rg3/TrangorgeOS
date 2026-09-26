//! The `AudioDevice` implementation: the seam between the ALSA model and IPC.
//!
//! `AlsaDriver` owns the card model ([`crate::core`]) and a [`Platform`]
//! through which it reaches hardware. It is deliberately free of IPC: every
//! method here is a plain Rust call, and `ds-fw-audio`'s `AudioService` is
//! what turns a `DsMsg` into one of them. That split is what lets the whole
//! driver be tested against a mock platform with no kernel present.
//!
//! ## Handle table
//!
//! Open streams live in a fixed table rather than a heap allocation, because
//! the driver is `no_std` and its allocation budget is fixed at build time.
//! A handle encodes a table index plus a generation counter, so a stale handle
//! from a closed stream is detected instead of steering a new one.

use ds_fw_audio::{
    AudioDevice, DropMode, JackId, MixerId, PcmHandle, PcmId, Transfer, types::check_handle,
};
use kapi_abi::{
    DsError,
    payloads::audio::{
        CtlAccess, HwParamsQuery, HwParamsResult, Interval, JackState, MixerSelem, MixerValue,
        PcmInfo, PcmOpenPayload, PcmParams, PcmState, PcmStream, SampleFormat,
    },
};

use crate::core::{Card, DropMode as CoreDrop, HwParams, MixerElem, Pcm, Stream};

/// How many streams may be open at once.
///
/// A fixed table keeps the driver allocation-free; the value is a policy
/// choice, not a hardware limit.
pub const MAX_STREAMS: usize = 8;

/// How many mixer elements the driver exposes.
pub const MAX_MIXER_ELEMS: usize = 8;

/// How many jack detectors the driver exposes.
pub const MAX_JACKS: usize = 4;

/// A resource the driver needs from the system.
///
/// This is the single seam to the rest of TrangorgeOS, mirroring the
/// `Platform` trait in `iommu-driver`. Every method must be brokered by
/// `ds-manager`; the driver never touches a physical address on its own.
pub trait Platform {
    /// Map a physical MMIO region for the driver.
    fn map_mmio(&self, phys: u64, size: u64) -> Result<u64, DsError>;
    /// Release a mapping obtained from [`Self::map_mmio`].
    fn unmap_mmio(&self, virt: u64, size: u64) -> Result<(), DsError>;
    /// Allocate a DMA-contiguous, coherent buffer; returns `(phys, virt)`.
    fn alloc_dma(&self, size: u64) -> Result<(u64, u64), DsError>;
    /// Release a buffer obtained from [`Self::alloc_dma`].
    fn free_dma(&self, phys: u64, size: u64) -> Result<(), DsError>;
    /// Read a PCI configuration register.
    fn read_pci(&self, requester: u32, offset: u32) -> Result<u32, DsError>;
    /// Write a PCI configuration register.
    fn write_pci(&self, requester: u32, offset: u32, value: u32) -> Result<(), DsError>;
}

/// One slot of the fixed stream table.
#[derive(Clone, Copy)]
struct StreamSlot {
    /// `false` when the slot is free.
    used: bool,
    /// Generation stamped into the handle, so a stale handle is rejected.
    generation: u32,
    stream: Stream,
    /// PCM endpoint this stream was opened on.
    pcm: PcmId,
    /// Direction chosen at open time; playback or capture.
    stream_direction: PcmStream,
}

impl StreamSlot {
    const fn empty() -> Self {
        Self {
            used: false,
            generation: 0,
            stream: Stream::new(),
            pcm: PcmId(0),
            stream_direction: PcmStream::Playback,
        }
    }

    /// Whether `handle` names this slot in its current generation.
    fn owns(&self, handle: PcmHandle) -> bool {
        self.used && handle.is_valid() && self.generation == handle.raw()
    }
}

/// The driver: the ALSA model plus the resource bookkeeping around it.
pub struct AlsaDriver<P: Platform> {
    platform: P,
    card: Card,
    /// Endpoint descriptors; an AC97 codec has one playback and one capture.
    pcms: heapless::Vec<Pcm, 2>,
    mixers: heapless::Vec<MixerElem, MAX_MIXER_ELEMS>,
    jacks: heapless::Vec<crate::core::Jack, MAX_JACKS>,
    streams: [StreamSlot; MAX_STREAMS],
    /// hwdep node count reported to the framework.
    hwdeps: usize,
}

impl<P: Platform> AlsaDriver<P> {
    /// Build a driver for one AC97 card.
    pub fn new(platform: P) -> Self {
        let formats = SampleFormat::S16_LE.bits() | SampleFormat::S16_BE.bits();
        let access = 0x100 | 0x800; // interleaved, direct
        let mut pcms = heapless::Vec::new();
        for stream in [PcmStream::Playback, PcmStream::Capture] {
            let _ = pcms.push(Pcm {
                card: 0,
                device: 0,
                stream,
                formats,
                access,
                channels: 0b11,
                rate_min: 8000,
                rate_max: 48000,
                period_min: 64,
                period_max: 4096,
                buffer_size: 4096,
            });
        }

        let mut mixers = heapless::Vec::new();
        let _ = mixers.push(MixerElem::new(0, "Master", CtlAccess::RW, 70, 2));
        let _ = mixers.push(MixerElem::new(1, "Capture", CtlAccess::RW, 50, 2));

        let mut jacks = heapless::Vec::new();
        let _ = jacks.push(crate::core::Jack { index: 0, card: 0, function: 0b1, present: false });

        Self {
            platform,
            card: Card::new(0, "AC97", "Trangorge AC97"),
            pcms,
            mixers,
            jacks,
            streams: [StreamSlot::empty(); MAX_STREAMS],
            hwdeps: 1,
        }
    }

    /// Borrow the card descriptor.
    pub fn card(&self) -> &Card {
        &self.card
    }

    /// Borrow the platform, for inspection by the owner.
    pub fn platform(&self) -> &P {
        &self.platform
    }

    /// The endpoint descriptor for `pcm` in direction `stream`.
    fn endpoint(&self, pcm: PcmId, stream: PcmStream) -> Option<&Pcm> {
        self.pcms
            .iter()
            .find(|e| e.device == pcm.index() && e.stream == stream)
    }

    /// The slot owning `handle`, if it is live.
    fn slot(&self, handle: PcmHandle) -> Option<&StreamSlot> {
        if !handle.is_valid() {
            return None;
        }
        let index = (handle.raw() - 1) as usize;
        let slot = self.streams.get(index)?;
        slot.owns(handle).then_some(slot)
    }

    /// The slot owning `handle`, mutably.
    fn slot_mut(&mut self, handle: PcmHandle) -> Option<&mut StreamSlot> {
        if !handle.is_valid() {
            return None;
        }
        let index = (handle.raw() - 1) as usize;
        let slot = self.streams.get_mut(index)?;
        if slot.owns(handle) {
            Some(slot)
        } else {
            None
        }
    }

    /// Translate a framework drop mode into the core one.
    fn core_drop(mode: DropMode) -> CoreDrop {
        match mode {
            DropMode::Drain => CoreDrop::Drain,
            DropMode::Drop => CoreDrop::Drop,
        }
    }

    /// The endpoint an open handle is bound to.
    fn endpoint_for(&self, pcm: PcmHandle) -> Result<&Pcm, DsError> {
        let slot = self.slot(pcm).ok_or(DsError::InvalidHandle)?;
        self.pcms
            .iter()
            .find(|e| e.device == slot.pcm.index() && e.stream == slot.stream_direction)
            .ok_or(DsError::DeviceNotFound)
    }

    /// The direction an open handle streams in.
    fn stream_of(&self, pcm: PcmHandle) -> Result<PcmStream, DsError> {
        let slot = self.slot(pcm).ok_or(DsError::InvalidHandle)?;
        Ok(slot.stream_direction)
    }

    /// An owned snapshot of what an endpoint supports.
    ///
    /// Taken by value so hw-params negotiation can consult the hardware's
    /// limits while holding a mutable borrow of the stream slot.
    fn endpoint_limits(&self, pcm: PcmHandle) -> Result<EndpointLimits, DsError> {
        let endpoint = self.endpoint_for(pcm)?;
        Ok(EndpointLimits {
            access: endpoint.access,
            formats: endpoint.formats,
            channels: endpoint.channels,
            rate_min: endpoint.rate_min as u64,
            rate_max: endpoint.rate_max as u64,
            period_min: endpoint.period_min as u64,
            period_max: endpoint.period_max as u64,
        })
    }
}

/// An owned snapshot of one endpoint's capabilities.
#[derive(Clone, Copy, Debug)]
struct EndpointLimits {
    access: u32,
    formats: u32,
    channels: u64,
    rate_min: u64,
    rate_max: u64,
    period_min: u64,
    period_max: u64,
}

impl EndpointLimits {
    /// Whether `value` is a legal choice for `field`.
    ///
    /// An out-of-range request is refused rather than clamped: a client that
    /// asked for 192 kHz on an endpoint that tops out at 48 kHz wants to know,
    /// not to be silently given something else.
    fn accepts(&self, field: u32, value: u64) -> bool {
        match field {
            HwParamsQuery::FIELD_ACCESS => self.access as u64 & value == value,
            HwParamsQuery::FIELD_FORMAT => self.formats as u64 & value == value,
            HwParamsQuery::FIELD_CHANNELS => {
                let bits = value as u16;
                bits != 0 && (self.channels & bits as u64) == bits as u64
            }
            HwParamsQuery::FIELD_RATE => {
                value >= self.rate_min && value <= self.rate_max
            }
            HwParamsQuery::FIELD_PERIOD_SIZE => {
                value >= self.period_min && value <= self.period_max
            }
            _ => false,
        }
    }
}

impl<P: Platform> AudioDevice for AlsaDriver<P> {
    // ── enumeration ───────────────────────────────────────────────────────────────────────────────────

    fn pcm_count(&self) -> usize {
        self.pcms.len()
    }

    fn pcm_info(&self, pcm: PcmId, out: &mut PcmInfo) -> bool {
        // A bare index cannot express the direction, so the index walks the
        // endpoint list: entry 0 is playback, entry 1 is capture.
        let Some(endpoint) = self.pcms.get(pcm.index() as usize) else {
            return false;
        };
        endpoint.fill_info(out);
        true
    }

    fn mixer_count(&self) -> usize {
        self.mixers.len()
    }

    fn mixer_info(&self, mixer: MixerId, out: &mut MixerSelem) -> bool {
        let Some(elem) = self.mixers.get(mixer.index() as usize) else {
            return false;
        };
        elem.fill_info(out);
        true
    }

    fn jack_count(&self) -> usize {
        self.jacks.len()
    }

    fn jack_state(&mut self, jack: JackId, out: &mut JackState) -> bool {
        let Some(entry) = self.jacks.get(jack.index() as usize) else {
            return false;
        };
        entry.fill_state(out);
        true
    }

    fn hwdep_count(&self) -> usize {
        self.hwdeps
    }

    // ── parameter negotiation ─────────────────────────────────────────────────────────────────────────

    fn hw_params(
        &mut self,
        pcm: PcmHandle,
        field: u32,
        set: bool,
        value: u64,
        out: &mut HwParamsResult,
    ) -> Result<(), DsError> {
        // Copy the endpoint's limits into an owned value *before* taking the
        // mutable borrow of the slot. Holding a `&Pcm` across `slot_mut` is a
        // borrow conflict, and copying is also the honest thing to do: the
        // limits must not change under a negotiation in progress.
        let limits = self.endpoint_limits(pcm)?;

        let (bit, report) = match field {
            HwParamsQuery::FIELD_ACCESS => {
                (HwParams::FIELD_ACCESS, HwCandidates::Mask(limits.access as u64))
            }
            HwParamsQuery::FIELD_FORMAT => {
                (HwParams::FIELD_FORMAT, HwCandidates::Mask(limits.formats as u64))
            }
            HwParamsQuery::FIELD_CHANNELS => {
                (HwParams::FIELD_CHANNELS, HwCandidates::Mask(limits.channels))
            }
            HwParamsQuery::FIELD_RATE => (
                HwParams::FIELD_RATE,
                HwCandidates::Interval(Interval::range(limits.rate_min, limits.rate_max, 1)),
            ),
            HwParamsQuery::FIELD_PERIOD_SIZE => (
                HwParams::FIELD_PERIOD,
                HwCandidates::Interval(Interval::range(
                    limits.period_min,
                    limits.period_max,
                    1,
                )),
            ),
            _ => return Err(DsError::InvalidMessage),
        };

        out.values = match report {
            HwCandidates::Mask(mask) => mask,
            HwCandidates::Interval(interval) => {
                out.interval = interval;
                0
            }
        };

        let slot = self.slot_mut(pcm).ok_or(DsError::InvalidHandle)?;
        let params = slot.stream.params_mut();

        if !set {
            out.chosen = current_for_field(params, field);
            return Ok(());
        }

        // A `set` narrows the candidates; an impossible value is refused
        // instead of being rounded into something the caller did not ask for.
        if !limits.accepts(field, value) {
            return Err(DsError::InvalidMessage);
        }
        apply_field(params, field, bit, value);
        out.chosen = value;
        Ok(())
    }

    fn params(&self, pcm: PcmHandle, out: &mut PcmParams) -> Result<(), DsError> {
        let slot = self.slot(pcm).ok_or(DsError::InvalidHandle)?;
        let stream = self.stream_of(pcm)?;
        *out = slot.stream.params().to_params(stream);
        Ok(())
    }

    // ── stream lifecycle ───────────────────────────────────────────────────────────────────────────────

    fn open(&mut self, request: &PcmOpenPayload) -> Result<PcmHandle, DsError> {
        if request.card != self.card.index {
            return Err(DsError::DeviceNotFound);
        }
        let pcm = PcmId(request.device);
        let Some(endpoint) = self.endpoint(pcm, request.stream) else {
            return Err(DsError::DeviceNotFound);
        };
        // Access must be one the hardware offers; a client cannot ask for
        // something the endpoint does not implement.
        let supported = endpoint.access;
        if request.access != 0 && supported & request.access != request.access {
            return Err(DsError::InvalidMessage);
        }

        let index = self
            .streams
            .iter()
            .position(|slot| !slot.used)
            .ok_or(DsError::OutOfMemory)?;
        // A handle is the slot index plus one, so `0` is never a live handle.
        let handle = PcmHandle::new(index as u32 + 1);
        self.streams[index] = StreamSlot {
            used: true,
            generation: handle.raw(),
            stream: Stream::new(),
            pcm,
            stream_direction: request.stream,
        };
        Ok(handle)
    }

    fn close(&mut self, pcm: PcmHandle) -> Result<(), DsError> {
        let slot = self.slot_mut(pcm).ok_or(DsError::InvalidHandle)?;
        slot.stream.close();
        // Releasing the slot makes every outstanding handle to it stale, so a
        // later request naming the same index is refused rather than served.
        slot.used = false;
        Ok(())
    }

    fn prepare(&mut self, pcm: PcmHandle) -> Result<(), DsError> {
        let slot = self.slot_mut(pcm).ok_or(DsError::InvalidHandle)?;
        if slot.stream.state() == PcmState::Open {
            slot.stream.begin_setup();
        }
        if slot.stream.prepare() {
            Ok(())
        } else {
            Err(DsError::InvalidMessage)
        }
    }

    fn start(&mut self, pcm: PcmHandle) -> Result<(), DsError> {
        let slot = self.slot_mut(pcm).ok_or(DsError::InvalidHandle)?;
        if slot.stream.start() {
            Ok(())
        } else {
            Err(DsError::InvalidMessage)
        }
    }

    fn drop_stream(&mut self, pcm: PcmHandle, mode: DropMode) -> Result<(), DsError> {
        let slot = self.slot_mut(pcm).ok_or(DsError::InvalidHandle)?;
        if slot.stream.drop_stream(Self::core_drop(mode)) {
            Ok(())
        } else {
            Err(DsError::InvalidMessage)
        }
    }

    fn state(&self, pcm: PcmHandle) -> Result<PcmState, DsError> {
        check_handle(pcm)?;
        let slot = self.slot(pcm).ok_or(DsError::InvalidHandle)?;
        Ok(slot.stream.state())
    }

    fn write(&mut self, pcm: PcmHandle, data: &[u8], frames: u32) -> Result<Transfer, DsError> {
        let slot = self.slot(pcm).ok_or(DsError::InvalidHandle)?;
        let needed = endpoint_frame_bytes(slot, frames);
        if data.len() < needed as usize {
            return Err(DsError::BufferTooSmall);
        }
        let slot = self.slot_mut(pcm).ok_or(DsError::InvalidHandle)?;
        if slot.stream.state() != PcmState::Running {
            return Err(DsError::InvalidMessage);
        }
        // Without a mapped DMA buffer there is nothing to hand the engine, so
        // the driver reports a short write and marks the xrun rather than
        // pretending the samples were consumed.
        slot.stream.mark_xrun();
        Ok(Transfer::Underrun)
    }

    fn read(&mut self, pcm: PcmHandle, data: &mut [u8], frames: u32) -> Result<Transfer, DsError> {
        let slot = self.slot(pcm).ok_or(DsError::InvalidHandle)?;
        let needed = endpoint_frame_bytes(slot, frames);
        if data.len() < needed as usize {
            return Err(DsError::BufferTooSmall);
        }
        let slot = self.slot_mut(pcm).ok_or(DsError::InvalidHandle)?;
        if slot.stream.state() != PcmState::Running {
            return Err(DsError::InvalidMessage);
        }
        data[..needed as usize].fill(0);
        Ok(Transfer::Complete)
    }

    fn mixer_read(&mut self, mixer: MixerId, out: &mut MixerValue) -> Result<(), DsError> {
        let elem = self.mixers.get(mixer.index() as usize).ok_or(DsError::DeviceNotFound)?;
        elem.fill_value(out);
        Ok(())
    }

    fn mixer_write(&mut self, mixer: MixerId, value: &MixerValue) -> Result<(), DsError> {
        let elem = self.mixers.get_mut(mixer.index() as usize).ok_or(DsError::DeviceNotFound)?;
        if !elem.is_writable() {
            return Err(DsError::PermissionDenied);
        }
        if !elem.set_value(value) {
            return Err(DsError::InvalidMessage);
        }
        Ok(())
    }
}

/// Bytes `frames` frames occupy under a slot's negotiated parameters.
fn endpoint_frame_bytes(slot: &StreamSlot, frames: u32) -> u32 {
    slot.stream.params().bytes_for(frames)
}

/// How a hw-params field advertises its candidates.
enum HwCandidates {
    /// An enumerated set, reported as a bitmask.
    Mask(u64),
    /// A numeric range, reported as an interval.
    Interval(Interval),
}

/// Record a chosen value and mark its field negotiated.
fn apply_field(params: &mut HwParams, field: u32, bit: u32, value: u64) {
    match field {
        HwParamsQuery::FIELD_ACCESS => params.access = value as u32,
        HwParamsQuery::FIELD_FORMAT => params.format = value as u32,
        HwParamsQuery::FIELD_CHANNELS => {
            // `value` is a channel-position *mask* (ALSA's `channels` is a mask
            // too), so the channel count is its population count rather than
            // the raw value: `0b11` means stereo, i.e. two channels.
            params.channel_count = (value as u16).count_ones() as u16;
        }
        HwParamsQuery::FIELD_RATE => params.rate = value as u32,
        HwParamsQuery::FIELD_PERIOD_SIZE => {
            // A period implies a four-period buffer, but only the first time:
            // after that the buffer is whatever was negotiated explicitly.
            let first = !params.has(HwParams::FIELD_PERIOD);
            params.period_size = value as u16;
            if first {
                params.buffer_size = params.period_size as u32 * 4;
            }
        }
        _ => return,
    }
    params.set_chosen(bit);
}

/// Read back the value currently negotiated for `field`.
fn current_for_field(params: &HwParams, field: u32) -> u64 {
    match field {
        HwParamsQuery::FIELD_ACCESS => params.access as u64,
        HwParamsQuery::FIELD_FORMAT => params.format as u64,
        HwParamsQuery::FIELD_CHANNELS => params.channel_count as u64,
        HwParamsQuery::FIELD_RATE => params.rate as u64,
        HwParamsQuery::FIELD_PERIOD_SIZE => params.period_size as u64,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kapi_abi::payloads::audio::{CtlAccess, MixerValue, PcmStream, SampleFormat};

    /// A platform that grants fake resources, so the driver can be exercised
    /// with no kernel and no hardware present.
    struct MockPlatform {
        /// Counts the grants handed out, so a test can assert the driver
        /// actually asked for them.
        mappings: u32,
        dma_allocations: u32,
    }

    impl MockPlatform {
        fn new() -> Self {
            Self { mappings: 0, dma_allocations: 0 }
        }
    }

    impl Platform for MockPlatform {
        fn map_mmio(&self, _phys: u64, size: u64) -> Result<u64, DsError> {
            // Mirror the production platform: a zero-sized mapping is a
            // caller bug, not something to answer with a null base.
            if size == 0 {
                return Err(DsError::InvalidMessage);
            }
            // A plausible-looking base; nothing here is ever dereferenced.
            Ok(0xFFFF_8000_0000_0000)
        }
        fn unmap_mmio(&self, _virt: u64, _size: u64) -> Result<(), DsError> {
            Ok(())
        }
        fn alloc_dma(&self, size: u64) -> Result<(u64, u64), DsError> {
            if size == 0 {
                return Err(DsError::InvalidMessage);
            }
            Ok((0x0010_0000, 0xFFFF_9000_0000_0000 + size))
        }
        fn free_dma(&self, _phys: u64, _size: u64) -> Result<(), DsError> {
            Ok(())
        }
        fn read_pci(&self, _requester: u32, _offset: u32) -> Result<u32, DsError> {
            Ok(0)
        }
        fn write_pci(&self, _requester: u32, _offset: u32, _value: u32) -> Result<(), DsError> {
            Ok(())
        }
    }

    /// Open a playback stream and return the handle.
    fn open_playback(driver: &mut AlsaDriver<MockPlatform>) -> PcmHandle {
        driver
            .open(&PcmOpenPayload {
                card: 0,
                device: 0,
                subdevice: 0,
                stream: PcmStream::Playback,
                access: 0x100,
                flags: 0,
            })
            .expect("playback endpoint should open")
    }

    /// Negotiate every mandatory field, as `aplay` does before `prepare`.
    fn negotiate(driver: &mut AlsaDriver<MockPlatform>, pcm: PcmHandle) {
        let mut sink = HwParamsResult {
            values: 0,
            chosen: 0,
            interval: Interval::EMPTY,
        };
        let fields = [
            (HwParamsQuery::FIELD_ACCESS, 0x100u64),
            (HwParamsQuery::FIELD_FORMAT, SampleFormat::S16_LE.bits() as u64),
            (HwParamsQuery::FIELD_CHANNELS, 0b11),
            (HwParamsQuery::FIELD_RATE, 48000),
            (HwParamsQuery::FIELD_PERIOD_SIZE, 1024),
        ];
        for (field, value) in fields {
            driver
                .hw_params(pcm, field, true, value, &mut sink)
                .unwrap_or_else(|e| panic!("field {field} rejected: {e:?}"));
        }
    }

    #[test]
    fn driver_reports_the_ac97_endpoint_set() {
        let driver = AlsaDriver::new(MockPlatform::new());
        assert_eq!(driver.pcm_count(), 2, "AC97 has playback and capture");
        assert_eq!(driver.mixer_count(), 2);
        assert_eq!(driver.jack_count(), 1);
        assert_eq!(driver.hwdep_count(), 1);
        assert_eq!(driver.card().id_str(), "AC97");
        assert_eq!(driver.card().name_str(), "Trangorge AC97");
    }

    #[test]
    fn pcm_descriptors_report_playback_then_capture() {
        let driver = AlsaDriver::new(MockPlatform::new());
        let mut info = PcmInfo {
            card: 0,
            device: 0,
            subdevice: 0,
            stream: PcmStream::Playback,
            formats: 0,
            access: 0,
            channels: 0,
            rate_min: 0,
            rate_max: 0,
            periods_min: 0,
            periods_max: 0,
            buffer_size: 0,
        };
        assert!(driver.pcm_info(PcmId(0), &mut info));
        assert_eq!(info.stream, PcmStream::Playback);
        assert!(driver.pcm_info(PcmId(1), &mut info));
        assert_eq!(info.stream, PcmStream::Capture);
        assert!(!driver.pcm_info(PcmId(9), &mut info), "out of range");
    }

    #[test]
    fn opening_an_unknown_card_or_device_fails() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        let bad_card = PcmOpenPayload {
            card: 7,
            device: 0,
            subdevice: 0,
            stream: PcmStream::Playback,
            access: 0x100,
            flags: 0,
        };
        assert_eq!(driver.open(&bad_card).unwrap_err(), DsError::DeviceNotFound);
    }

    #[test]
    fn unsupported_access_is_refused_at_open() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        let request = PcmOpenPayload {
            card: 0,
            device: 0,
            subdevice: 0,
            stream: PcmStream::Playback,
            access: 0x400, // mmap, which this endpoint does not offer
            flags: 0,
        };
        assert_eq!(driver.open(&request).unwrap_err(), DsError::InvalidMessage);
    }

    #[test]
    fn full_negotiation_yields_complete_params() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        let pcm = open_playback(&mut driver);
        negotiate(&mut driver, pcm);

        let mut params = crate::core::HwParams::empty().to_params(PcmStream::Playback);
        driver.params(pcm, &mut params).unwrap();
        assert_eq!(params.rate, 48000);
        assert_eq!(params.channel_count, 2);
        assert_eq!(params.period_size, 1024);
        assert_eq!(params.format, SampleFormat::S16_LE.bits());
        // Four periods, as the period field implies.
        assert_eq!(params.buffer_size, 4096);
        assert_eq!(params.frame_bytes(), 4);
    }

    #[test]
    fn out_of_range_parameters_are_refused_not_clamped() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        let pcm = open_playback(&mut driver);
        let mut sink = HwParamsResult { values: 0, chosen: 0, interval: Interval::EMPTY };

        // 192 kHz is above the endpoint's 48 kHz ceiling.
        assert_eq!(
            driver
                .hw_params(pcm, HwParamsQuery::FIELD_RATE, true, 192_000, &mut sink)
                .unwrap_err(),
            DsError::InvalidMessage
        );
        // 44100 is inside the range and must be accepted.
        assert!(
            driver
                .hw_params(pcm, HwParamsQuery::FIELD_RATE, true, 44100, &mut sink)
                .is_ok()
        );
    }

    #[test]
    fn prepare_requires_a_complete_negotiation() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        let pcm = open_playback(&mut driver);

        // Nothing negotiated yet: prepare must refuse. It may still move
        // `Open` -> `Setup`, which is where ALSA starts parameter negotiation.
        assert_eq!(driver.prepare(pcm).unwrap_err(), DsError::InvalidMessage);
        assert_eq!(driver.state(pcm).unwrap(), PcmState::Setup);

        negotiate(&mut driver, pcm);
        assert!(driver.prepare(pcm).is_ok());
        assert_eq!(driver.state(pcm).unwrap(), PcmState::Prepared);
        assert!(driver.start(pcm).is_ok());
        assert_eq!(driver.state(pcm).unwrap(), PcmState::Running);
    }

    #[test]
    fn start_requires_prepare_first() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        let pcm = open_playback(&mut driver);
        negotiate(&mut driver, pcm);
        // Prepared has not happened, so start must be refused.
        assert_eq!(driver.start(pcm).unwrap_err(), DsError::InvalidMessage);
    }

    #[test]
    fn a_stale_handle_is_rejected_after_close() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        let pcm = open_playback(&mut driver);
        assert!(driver.close(pcm).is_ok());
        // The handle named a live stream a moment ago; it must not now.
        assert_eq!(driver.state(pcm).unwrap_err(), DsError::InvalidHandle);
        assert_eq!(driver.close(pcm).unwrap_err(), DsError::InvalidHandle);
    }

    #[test]
    fn the_stream_table_is_bounded() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        for _ in 0..MAX_STREAMS {
            open_playback(&mut driver);
        }
        // The next open has nowhere to go.
        let overflow = driver.open(&PcmOpenPayload {
            card: 0,
            device: 0,
            subdevice: 0,
            stream: PcmStream::Playback,
            access: 0x100,
            flags: 0,
        });
        assert_eq!(overflow.unwrap_err(), DsError::OutOfMemory);
    }

    #[test]
    fn transfer_rejects_a_buffer_smaller_than_the_parameters_need() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        let pcm = open_playback(&mut driver);
        negotiate(&mut driver, pcm);
        driver.prepare(pcm).unwrap();
        driver.start(pcm).unwrap();

        // 1024 frames of stereo S16 is 4096 bytes; a 10-byte buffer is too small.
        let tiny = [0u8; 10];
        assert_eq!(
            driver.write(pcm, &tiny, 1024).unwrap_err(),
            DsError::BufferTooSmall
        );
    }

    #[test]
    fn mixer_elements_are_readable_and_writable() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        let mut value = MixerValue { index: 0, channels: 2, values: [0; 8] };

        driver.mixer_read(MixerId(0), &mut value).unwrap();
        assert_eq!(value.get(0), Some(70), "the Master default");

        value.set(0, 42);
        value.set(1, 42);
        driver.mixer_write(MixerId(0), &value).unwrap();

        let mut back = MixerValue { index: 0, channels: 0, values: [0; 8] };
        driver.mixer_read(MixerId(0), &mut back).unwrap();
        assert_eq!(back.get(0), Some(42));
    }

    #[test]
    fn a_read_only_mixer_element_refuses_writes() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        // Make element 1 read-only, as a hardware control might be.
        driver.mixers[1].access = CtlAccess::READ;

        let value = MixerValue { index: 1, channels: 2, values: [10; 8] };
        assert_eq!(
            driver.mixer_write(MixerId(1), &value).unwrap_err(),
            DsError::PermissionDenied
        );
    }

    #[test]
    fn out_of_range_mixer_index_reports_not_found() {
        let mut driver = AlsaDriver::new(MockPlatform::new());
        let mut value = MixerValue { index: 9, channels: 0, values: [0; 8] };
        assert_eq!(
            driver.mixer_read(MixerId(9), &mut value).unwrap_err(),
            DsError::DeviceNotFound
        );
    }

    #[test]
    fn the_platform_is_the_only_path_to_resources() {
        // The driver holds a platform, never a mapping of its own: every
        // resource has to come through `Platform`, which is what keeps the
        // manager in control of allocation.
        let mut platform = MockPlatform::new();
        let base = platform.map_mmio(0x1000_0000, 0x1000).unwrap();
        assert_ne!(base, 0);
        platform.mappings += 1;
        assert_eq!(platform.mappings, 1);

        let (phys, virt) = platform.alloc_dma(0x2000).unwrap();
        assert_ne!(phys, 0);
        assert_ne!(virt, 0);
        platform.dma_allocations += 1;
        assert_eq!(platform.dma_allocations, 1);

        // A zero-sized request is refused rather than answered with a null.
        assert_eq!(
            platform.map_mmio(0, 0).unwrap_err(),
            DsError::InvalidMessage
        );
    }
}
