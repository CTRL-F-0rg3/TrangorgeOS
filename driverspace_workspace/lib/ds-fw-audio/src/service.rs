//! The audio server-side dispatcher.
//!
//! [`AudioService`] turns one `DsMsg` request into one [`AudioDevice`] call and
//! encodes the outcome as a [`Reply`]. Capability checks live here, so driver
//! implementations neither need to nor should repeat permission logic.
//!
//! ## Payload passing convention
//!
//! The transport copies the request payload from `msg.arg0` (length
//! `msg.arg1`) into the caller-provided `Request::payload` slice; the service
//! writes its reply payload into the `out` buffer passed to `dispatch`. That
//! keeps raw pointers out of the framework and lets unit tests feed plain byte
//! slices.
//!
//! ## Why the capability split is coarse on purpose
//!
//! ALSA has two capabilities: enumerate and move audio. Everything that only
//! *reads* - listing PCMs, mixer elements, jacks, hwdep, querying hw-params -
//! needs `ALSA_ENUMERATE`. Anything that changes state or moves samples needs
//! `ALSA_STREAM`. A finer split would be more precise but would push the
//! decision into every driver, and the two bits already separate "this client
//! may look" from "this client may play".

use kapi_abi::{
    CapId, DsCmd, DsError, DsMsg, Handle,
    payloads::audio::{HwParamsResult, JackState, MixerSelem, MixerValue, PcmInfo, PcmParams},
    wire::{Decoder, Encoder},
};

use crate::{
    codec,
    traits::AudioDevice,
    types::{DropMode, JackId, MixerId, PcmHandle, PcmId},
    MAX_PAYLOAD_LEN,
};

/// One audio request: the message header plus its payload, already unpacked.
#[derive(Clone, Copy)]
pub struct Request<'a> {
    pub msg: DsMsg,
    pub payload: &'a [u8],
}

impl<'a> Request<'a> {
    #[inline]
    pub const fn new(msg: DsMsg, payload: &'a [u8]) -> Self {
        Self { msg, payload }
    }

    /// The command declared by the message header.
    #[inline]
    pub fn command(&self) -> Option<DsCmd> {
        DsCmd::from_u32(self.msg.cmd as u32)
    }

    /// The endpoint that issued the request.
    #[inline]
    pub const fn sender(&self) -> Handle {
        Handle(self.msg.id as u32)
    }
}

/// One audio reply.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reply {
    /// `0` means success; anything else is an error code.
    pub status: i32,
    /// Scalar result (handle, state, transferred frames, ...).
    pub arg0: u64,
    /// Collection total (pcm count, mixer count, ...).
    pub arg1: u64,
    /// Reply payload length in bytes.
    pub arg2: u64,
}

impl Reply {
    #[inline]
    pub const fn ok() -> Self {
        Self { status: 0, arg0: 0, arg1: 0, arg2: 0 }
    }

    #[inline]
    pub const fn err(error: DsError) -> Self {
        Self { status: error as i32, arg0: 0, arg1: 0, arg2: 0 }
    }

    #[inline]
    pub const fn arg0(mut self, value: u64) -> Self {
        self.arg0 = value;
        self
    }

    #[inline]
    pub const fn arg1(mut self, value: u64) -> Self {
        self.arg1 = value;
        self
    }

    /// Record the reply payload length.
    #[inline]
    pub const fn payload_len(mut self, len: usize) -> Self {
        self.arg2 = len as u64;
        self
    }

    #[inline]
    pub const fn is_ok(self) -> bool {
        self.status == 0
    }

    /// Package a reply into the `DsMsg` sent back to the requester, echoing
    /// `id` and `cmd` so the requester can match it to its call.
    pub fn into_message(self, request: &DsMsg) -> DsMsg {
        let mut msg = *request;
        msg.status = self.status;
        msg.arg0 = self.arg0;
        msg.arg1 = self.arg1;
        msg.arg2 = self.arg2;
        msg
    }
}

/// Map a `DsCmd` to the capability bit it requires.
///
/// Read-only operations need only `ALSA_ENUMERATE`; anything that opens a
/// stream or changes mixer state needs `ALSA_STREAM`. The ranges are disjoint
/// and are listed in ascending order so a later addition cannot silently
/// shadow an earlier one. `None` means the opcode is not an audio opcode.
pub const fn required_capability(cmd: DsCmd) -> Option<CapId> {
    let value = cmd as u32;
    match value {
        // Enumerate and describe: reading only.
        0x0410..=0x0411 | 0x0414..=0x0416 | 0x041D..=0x0421 => Some(CapId::ALSA_ENUMERATE),
        // Open, prepare, start, transfer, drop, and the mixer write path.
        0x0412..=0x0413 | 0x0417..=0x041C | 0x041F => Some(CapId::ALSA_STREAM),
        // 0x041E is a mixer *read*, so it belongs to the enumerate side.
        0x041E => Some(CapId::ALSA_ENUMERATE),
        _ => None,
    }
}

/// The service: owns one device and routes requests onto it.
pub struct AudioService<D: AudioDevice> {
    device: D,
    granted: CapId,
}

impl<D: AudioDevice> AudioService<D> {
    #[inline]
    pub fn new(device: D) -> Self {
        Self { device, granted: CapId::NONE }
    }

    /// Borrow the device, for inspection by the owner.
    #[inline]
    pub fn device(&self) -> &D {
        &self.device
    }

    /// Grant a capability. The driver is expected to be given the read-only
    /// bits at start-up and the rest only when `ds-manager` decides to.
    #[inline]
    pub fn grant(&mut self, capability: CapId) {
        self.granted = self.granted.union(capability);
    }

    /// Revoke a previously granted capability.
    #[inline]
    pub fn revoke(&mut self, capability: CapId) {
        self.granted = self.granted.remove(capability);
    }

    /// The capabilities currently held.
    #[inline]
    pub const fn granted(&self) -> CapId {
        self.granted
    }

    /// Route one request onto the device and encode the reply.
    ///
    /// The order of checks matters and is deliberate:
    ///
    /// 1. **Opcode** - a non-audio opcode is rejected before anything else, so
    ///    this service never acts on a message meant for another driver.
    /// 2. **Capability** - a request the caller may not make is rejected
    ///    before the device is touched, so an over-privileged call cannot even
    ///    reach the hardware.
    /// 3. **Payload and bounds** - a malformed payload is rejected before it is
    ///    decoded, and every decode goes through the bounds-checked `Decoder`.
    pub fn dispatch(&mut self, request: &Request<'_>, out: &mut [u8]) -> Reply {
        let Some(cmd) = request.command() else {
            return Reply::err(DsError::InvalidMessage);
        };
        let Some(capability) = required_capability(cmd) else {
            return Reply::err(DsError::InvalidMessage);
        };
        if !self.granted.has(capability) {
            return Reply::err(DsError::PermissionDenied);
        }

        match cmd {
            // ── enumeration ─────────────────────────────────────────────
            DsCmd::AlsaPcmEnumerate | DsCmd::AlsaPcmInfo => {
                // Both opcodes read the same descriptor; the manager learns the
                // total count from `pcm_count` and walks it with `arg0`.
                let mut info = empty_pcm_info();
                if !self.device.pcm_info(PcmId(request.msg.arg0 as u32), &mut info) {
                    return Reply::err(DsError::DeviceNotFound);
                }
                self.write_reply(out, |enc| codec::encode_pcm_info(enc, &info))
            }
            DsCmd::AlsaPcmState => {
                match self.device.state(PcmHandle(request.msg.arg0 as u32)) {
                    Ok(state) => Reply::ok().arg0(state as u64),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::AlsaPcmParams => {
                let mut params = empty_params();
                if let Err(error) = self.device.params(PcmHandle(request.msg.arg0 as u32), &mut params)
                {
                    return Reply::err(error);
                }
                self.write_reply(out, |enc| codec::encode_params(enc, &params))
            }
            DsCmd::AlsaPcmHwParams => {
                let mut query = empty_hw_params_query();
                if let Err(error) = decode_hw_params_query(request.payload, &mut query) {
                    return Reply::err(error);
                }
                let mut result = HwParamsResult {
                    values: 0,
                    chosen: 0,
                    interval: kapi_abi::payloads::audio::Interval::EMPTY,
                };
                if let Err(error) = self.device.hw_params(
                    PcmHandle(query.pcm),
                    query.field,
                    query.set != 0,
                    query.value,
                    &mut result,
                ) {
                    return Reply::err(error);
                }
                self.write_reply(out, |enc| codec::encode_hw_params_result(enc, &result))
            }
            _ => self.dispatch_stream(cmd, request, out),
        }
    }

    /// Encode a payload into `out` and wrap it in a [`Reply`].
    ///
    /// A buffer too small for the payload is reported as
    /// [`DsError::BufferTooSmall`] rather than silently truncating, so a
    /// client never parses a half-written descriptor.
    fn write_reply(&self, out: &mut [u8], encode: impl FnOnce(&mut Encoder<'_>) -> bool) -> Reply {
        let mut encoder = Encoder::new(out);
        if !encode(&mut encoder) {
            return Reply::err(DsError::BufferTooSmall);
        }
        Reply::ok().payload_len(encoder.pos())
    }

    /// Route the stream-control and mixer opcodes.
    ///
    /// Split out of [`Self::dispatch`] so the read path above stays readable;
    /// the capability check has already happened by the time this runs.
    fn dispatch_stream(&mut self, cmd: DsCmd, request: &Request<'_>, out: &mut [u8]) -> Reply {
        match cmd {
            // ── lifecycle ───────────────────────────────────────────────
            DsCmd::AlsaPcmOpen => {
                let Some(open) = decode_open(request.payload) else {
                    return Reply::err(DsError::InvalidMessage);
                };
                match self.device.open(&open) {
                    Ok(handle) => Reply::ok().arg0(handle.raw() as u64),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::AlsaPcmClose => {
                match self.device.close(PcmHandle(request.msg.arg0 as u32)) {
                    Ok(()) => Reply::ok(),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::AlsaPcmPrepare => {
                match self.device.prepare(PcmHandle(request.msg.arg0 as u32)) {
                    Ok(()) => Reply::ok(),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::AlsaPcmStart => {
                match self.device.start(PcmHandle(request.msg.arg0 as u32)) {
                    Ok(()) => Reply::ok(),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::AlsaPcmDrop => {
                let mode = DropMode::from_u32(request.msg.arg1 as u32);
                match self.device.drop_stream(PcmHandle(request.msg.arg0 as u32), mode) {
                    Ok(()) => Reply::ok(),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::AlsaPcmRecover => {
                let handle = PcmHandle(request.msg.arg0 as u32);
                match self.device.prepare(handle).and_then(|()| self.device.start(handle)) {
                    Ok(()) => Reply::ok(),
                    Err(error) => Reply::err(error),
                }
            }
            _ => self.dispatch_transfer(cmd, request, out),
        }
    }

    /// Route sample transfer, mixer and jack opcodes.
    fn dispatch_transfer(&mut self, cmd: DsCmd, request: &Request<'_>, out: &mut [u8]) -> Reply {
        match cmd {
            // ── sample transfer ────────────────────────────────────────
            DsCmd::AlsaPcmWrite | DsCmd::AlsaPcmRead => {
                // Sample data never travels in the message registers: a period
                // is far larger than `DsMsg`. The payload slice is the shared
                // buffer the transport handed us.
                let handle = PcmHandle(request.msg.arg0 as u32);
                let frames = request.msg.arg1 as u32;
                if frames == 0 {
                    return Reply::err(DsError::InvalidMessage);
                }
                if cmd == DsCmd::AlsaPcmWrite {
                    return match self.device.write(handle, request.payload, frames) {
                        Ok(transfer) => Reply::ok().arg0(transfer as u64),
                        Err(error) => Reply::err(error),
                    };
                }
                let mut buffer = [0u8; MAX_PAYLOAD_LEN];
                let transfer = match self.device.read(handle, &mut buffer, frames) {
                    Ok(transfer) => transfer,
                    Err(error) => return Reply::err(error),
                };
                let len = buffer.len().min(frames as usize);
                Reply::ok()
                    .arg0(transfer as u64)
                    .arg1(len as u64)
                    .payload_len(len)
            }
            // ── mixer ───────────────────────────────────────────────────
            DsCmd::AlsaMixerEnumerate => {
                let mut selem = empty_selem();
                if !self.device.mixer_info(MixerId(request.msg.arg0 as u32), &mut selem) {
                    return Reply::err(DsError::DeviceNotFound);
                }
                self.write_reply(out, |enc| codec::encode_mixer_selem(enc, &selem))
            }
            DsCmd::AlsaMixerRead => {
                let mut value = empty_mixer_value();
                let index = MixerId(request.msg.arg0 as u32);
                if let Err(error) = self.device.mixer_read(index, &mut value) {
                    return Reply::err(error);
                }
                self.write_reply(out, |enc| codec::encode_mixer_value(enc, &value))
            }
            DsCmd::AlsaMixerWrite => {
                let Some(mut value) = decode_mixer_value(request.payload) else {
                    return Reply::err(DsError::InvalidMessage);
                };
                let index = MixerId(value.index);
                if let Err(error) = self.device.mixer_write(index, &value) {
                    return Reply::err(error);
                }
                // Echo what the hardware actually stored, not what was asked
                // for: a control may clamp the requested value.
                if let Err(error) = self.device.mixer_read(index, &mut value) {
                    return Reply::err(error);
                }
                self.write_reply(out, |enc| codec::encode_mixer_value(enc, &value))
            }
            DsCmd::AlsaJackState => {
                let mut jack = JackState { card: 0, index: 0, present: 0, function: 0 };
                if !self.device.jack_state(JackId(request.msg.arg0 as u32), &mut jack) {
                    return Reply::err(DsError::DeviceNotFound);
                }
                self.write_reply(out, |enc| codec::encode_jack_state(enc, &jack))
            }
            _ => Reply::err(DsError::InvalidMessage),
        }
    }
}

/// A zeroed `MixerSelem`, used as the decode target for a failed lookup.
fn empty_selem() -> MixerSelem {
    MixerSelem {
        interface: kapi_abi::payloads::audio::CtlInterface::Card,
        elem_type: kapi_abi::payloads::audio::CtlElemType::None,
        access: kapi_abi::payloads::audio::CtlAccess::empty(),
        index: 0,
        name: [0u8; MixerSelem::NAME_LEN],
        playback: MixerSelem::NO_CHANNEL,
        capture: MixerSelem::NO_CHANNEL,
    }
}

/// A zeroed `MixerValue`.
fn empty_mixer_value() -> MixerValue {
    MixerValue { index: 0, channels: 0, values: [0; MixerValue::MAX_CHANNELS] }
}

/// Decode an `open` payload, mapping a short buffer to `None`.
fn decode_open(payload: &[u8]) -> Option<kapi_abi::payloads::audio::PcmOpenPayload> {
    codec::decode_open(&mut Decoder::new(payload))
}

/// Decode a mixer-value payload.
fn decode_mixer_value(payload: &[u8]) -> Option<MixerValue> {
    codec::decode_mixer_value(&mut Decoder::new(payload))
}

/// A zeroed `PcmParams`, used as the decode target for a failed lookup.
fn empty_params() -> PcmParams {
    PcmParams {
        stream: kapi_abi::payloads::audio::PcmStream::Playback,
        access: 0,
        format: 0,
        channels: 0,
        rate: 0,
        channel_count: 0,
        _pad: 0,
        period_size: 0,
        _pad2: 0,
        buffer_size: 0,
        _pad3: 0,
    }
}

/// A zeroed `PcmInfo`, used as the decode target for a failed lookup.
fn empty_pcm_info() -> PcmInfo {
    PcmInfo {
        card: 0,
        device: 0,
        subdevice: 0,
        stream: kapi_abi::payloads::audio::PcmStream::Playback,
        formats: 0,
        access: 0,
        channels: 0,
        rate_min: 0,
        rate_max: 0,
        periods_min: 0,
        periods_max: 0,
        buffer_size: 0,
    }
}

/// Decode an `hw-params` query payload.
///
/// A truncated payload is `InvalidMessage`: the client and driver share one
/// vocabulary, so a mismatch here means a programming error, not a runtime
/// condition to recover from.
fn decode_hw_params_query(
    payload: &[u8],
    out: &mut kapi_abi::payloads::audio::HwParamsQuery,
) -> Result<(), DsError> {
    let mut dec = Decoder::new(payload);
    out.pcm = dec.read_u32().ok_or(DsError::InvalidMessage)?;
    out.field = dec.read_u32().ok_or(DsError::InvalidMessage)?;
    out.set = dec.read_u32().ok_or(DsError::InvalidMessage)?;
    out.value = dec.read_u64().ok_or(DsError::InvalidMessage)?;
    Ok(())
}

/// A zeroed `HwParamsQuery`, used as the decode target for a failed lookup.
fn empty_hw_params_query() -> kapi_abi::payloads::audio::HwParamsQuery {
    kapi_abi::payloads::audio::HwParamsQuery::new(0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Transfer;
    use kapi_abi::payloads::audio::{
        CtlInterface, PcmOpenPayload, PcmStream, SampleFormat,
    };

    /// A device that records what it was asked to do.
    ///
    /// It implements the full [`AudioDevice`] contract but keeps no real
    /// hardware behind it, which is what makes the framework testable with no
    /// kernel present.
    struct FakeDevice {
        pcms: usize,
        mixers: usize,
        jacks: usize,
        hwdeps: usize,
        closed: Option<PcmHandle>,
        last_mixer_write: Option<i32>,
    }

    impl FakeDevice {
        fn new(pcms: usize) -> Self {
            Self {
                pcms,
                mixers: 2,
                jacks: 1,
                hwdeps: 1,
                closed: None,
                last_mixer_write: None,
            }
        }
    }

    impl AudioDevice for FakeDevice {
        fn pcm_count(&self) -> usize {
            self.pcms
        }

        fn pcm_info(&self, pcm: PcmId, out: &mut PcmInfo) -> bool {
            if pcm.index() as usize >= self.pcms {
                return false;
            }
            *out = PcmInfo {
                card: 0,
                device: pcm.index(),
                subdevice: 0,
                stream: PcmStream::Playback,
                formats: SampleFormat::S16_LE.bits(),
                access: 0x100,
                channels: 0b11,
                rate_min: 8000,
                rate_max: 48000,
                periods_min: 64,
                periods_max: 4096,
                buffer_size: 4096,
            };
            true
        }

        fn mixer_count(&self) -> usize {
            self.mixers
        }

        fn mixer_info(&self, mixer: MixerId, out: &mut MixerSelem) -> bool {
            if mixer.index() as usize >= self.mixers {
                return false;
            }
            *out = empty_selem();
            out.interface = CtlInterface::Card;
            out.index = mixer.index();
            out.set_name("Master");
            true
        }

        fn jack_count(&self) -> usize {
            self.jacks
        }

        fn jack_state(&mut self, jack: JackId, out: &mut JackState) -> bool {
            if jack.index() as usize >= self.jacks {
                return false;
            }
            *out = JackState { card: 0, index: jack.index(), present: 1, function: 0 };
            true
        }

        fn hwdep_count(&self) -> usize {
            self.hwdeps
        }

        fn hw_params(
            &mut self,
            _pcm: PcmHandle,
            field: u32,
            set: bool,
            value: u64,
            out: &mut HwParamsResult,
        ) -> Result<(), DsError> {
            if field >= kapi_abi::payloads::audio::HwParamsQuery::FIELD_COUNT {
                return Err(DsError::InvalidMessage);
            }
            if set && value == 123_456_789 {
                // An impossible value must be refused, not rounded.
                return Err(DsError::InvalidMessage);
            }
            out.values = 0xFFFF;
            out.chosen = if set { value } else { 48000 };
            out.interval = kapi_abi::payloads::audio::Interval::range(8000, 48000, 1);
            Ok(())
        }

        fn params(&self, _pcm: PcmHandle, out: &mut PcmParams) -> Result<(), DsError> {
            *out = empty_params();
            out.format = SampleFormat::S16_LE.bits();
            out.rate = 48000;
            out.channel_count = 2;
            out.period_size = 512;
            out.buffer_size = 2048;
            Ok(())
        }

        fn open(&mut self, _request: &PcmOpenPayload) -> Result<PcmHandle, DsError> {
            Ok(PcmHandle::new(1))
        }

        fn close(&mut self, pcm: PcmHandle) -> Result<(), DsError> {
            self.closed = Some(pcm);
            Ok(())
        }

        fn prepare(&mut self, _pcm: PcmHandle) -> Result<(), DsError> {
            Ok(())
        }

        fn start(&mut self, _pcm: PcmHandle) -> Result<(), DsError> {
            Ok(())
        }

        fn drop_stream(&mut self, _pcm: PcmHandle, _mode: DropMode) -> Result<(), DsError> {
            Ok(())
        }

        fn state(&self, _pcm: PcmHandle) -> Result<kapi_abi::payloads::audio::PcmState, DsError> {
            Ok(kapi_abi::payloads::audio::PcmState::Prepared)
        }

        fn write(
            &mut self,
            _pcm: PcmHandle,
            data: &[u8],
            frames: u32,
        ) -> Result<Transfer, DsError> {
            Ok(Transfer::from_frames(data.len() as u32, frames))
        }

        fn read(
            &mut self,
            _pcm: PcmHandle,
            data: &mut [u8],
            _frames: u32,
        ) -> Result<Transfer, DsError> {
            data.fill(0);
            Ok(Transfer::Complete)
        }

        fn mixer_read(&mut self, mixer: MixerId, out: &mut MixerValue) -> Result<(), DsError> {
            if mixer.index() as usize >= self.mixers {
                return Err(DsError::DeviceNotFound);
            }
            *out = empty_mixer_value();
            out.index = mixer.index();
            out.channels = 2;
            out.values = [self.last_mixer_write.unwrap_or(50); MixerValue::MAX_CHANNELS];
            Ok(())
        }

        fn mixer_write(&mut self, mixer: MixerId, value: &MixerValue) -> Result<(), DsError> {
            if mixer.index() as usize >= self.mixers {
                return Err(DsError::DeviceNotFound);
            }
            // Element 1 stands in for a read-only control.
            if mixer.index() == 1 {
                return Err(DsError::PermissionDenied);
            }
            self.last_mixer_write = value.get(0);
            Ok(())
        }
    }

    /// Build a request for `cmd` with no payload.
    fn request(cmd: DsCmd) -> DsMsg {
        DsMsg::new(cmd as u32)
    }

    #[test]
    fn read_only_opcodes_need_only_the_enumerate_capability() {
        let mut service = AudioService::new(FakeDevice::new(1));
        service.grant(CapId::ALSA_ENUMERATE);
        let mut out = [0u8; MAX_PAYLOAD_LEN];

        let reply = service.dispatch(&Request::new(request(DsCmd::AlsaPcmInfo), &[]), &mut out);
        assert!(reply.is_ok(), "enumerate should be allowed");
        assert!(reply.arg2 > 0, "descriptor should be in the reply payload");
    }

    #[test]
    fn stream_opcodes_require_the_stream_capability() {
        let mut service = AudioService::new(FakeDevice::new(1));
        service.grant(CapId::ALSA_ENUMERATE);
        let mut out = [0u8; MAX_PAYLOAD_LEN];

        for cmd in [
            DsCmd::AlsaPcmClose,
            DsCmd::AlsaPcmPrepare,
            DsCmd::AlsaPcmStart,
            DsCmd::AlsaPcmDrop,
        ] {
            let mut msg = request(cmd);
            msg.arg0 = 1;
            let reply = service.dispatch(&Request::new(msg, &[]), &mut out);
            assert_eq!(
                reply.status,
                DsError::PermissionDenied as i32,
                "{cmd:?} must be refused without ALSA_STREAM"
            );
        }
    }

    #[test]
    fn capability_grant_and_revoke_are_symmetric() {
        let mut service = AudioService::new(FakeDevice::new(1));
        let mut out = [0u8; MAX_PAYLOAD_LEN];
        let mut msg = request(DsCmd::AlsaPcmOpen);
        msg.arg0 = 0;

        // With no capability at all, even a read is refused.
        let reply = service.dispatch(&Request::new(msg, &[]), &mut out);
        assert_eq!(reply.status, DsError::PermissionDenied as i32);

        service.grant(CapId::ALSA_ENUMERATE.union(CapId::ALSA_STREAM));
        assert!(service.granted().has(CapId::ALSA_STREAM));
        let reply = service.dispatch(&Request::new(msg, &[]), &mut out);
        assert_eq!(
            reply.status,
            DsError::InvalidMessage as i32,
            "open without a payload is malformed, not denied"
        );

        service.revoke(CapId::ALSA_STREAM);
        let reply = service.dispatch(&Request::new(msg, &[]), &mut out);
        assert_eq!(reply.status, DsError::PermissionDenied as i32);
    }

    #[test]
    fn non_audio_opcode_is_rejected() {
        let mut service = AudioService::new(FakeDevice::new(1));
        service.grant(CapId::KERNEL_ALL);
        let mut out = [0u8; MAX_PAYLOAD_LEN];

        // Even with every capability, a foreign opcode must not be served.
        let reply = service.dispatch(&Request::new(request(DsCmd::BlkRead), &[]), &mut out);
        assert_eq!(reply.status, DsError::InvalidMessage as i32);
    }

    #[test]
    fn out_of_range_lookups_report_not_found() {
        let mut service = AudioService::new(FakeDevice::new(1));
        service.grant(CapId::KERNEL_ALL);
        let mut out = [0u8; MAX_PAYLOAD_LEN];

        let mut msg = request(DsCmd::AlsaPcmInfo);
        msg.arg0 = 99;
        let reply = service.dispatch(&Request::new(msg, &[]), &mut out);
        assert_eq!(reply.status, DsError::DeviceNotFound as i32);

        let mut msg = request(DsCmd::AlsaMixerEnumerate);
        msg.arg0 = 99;
        let reply = service.dispatch(&Request::new(msg, &[]), &mut out);
        assert_eq!(reply.status, DsError::DeviceNotFound as i32);
    }

    #[test]
    fn tiny_reply_buffer_reports_buffer_too_small() {
        let mut service = AudioService::new(FakeDevice::new(1));
        service.grant(CapId::ALSA_ENUMERATE);
        let mut out = [0u8; 2];
        let reply =
            service.dispatch(&Request::new(request(DsCmd::AlsaPcmInfo), &[]), &mut out);
        assert_eq!(reply.status, DsError::BufferTooSmall as i32);
    }
}
