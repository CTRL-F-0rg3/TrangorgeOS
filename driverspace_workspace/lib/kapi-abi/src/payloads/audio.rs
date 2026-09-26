//! Audio / ALSA IPC message payloads.
//!
//! The `Alsa-TrangorgeOS` driver and its clients exchange PCM descriptors,
//! hardware-parameter queries, mixer elements, jack state and hwdep
//! descriptions through these structures. Every struct is strictly
//! `#[repr(C)]`, because they are interpreted by Rust, C and Odin alike in
//! shared memory.
//!
//! All requests follow the same convention as `LogPayload` and the IOMMU
//! payloads: `DsMsg::arg0` carries the payload pointer and `DsMsg::arg1` the
//! length in bytes.
//!
//! The vocabulary mirrors ALSA (see `drivers/Alsa-TrangorgeOS`): a *card* is
//! the hardware unit, a PCM is one endpoint of it, and `PcmAccess` /
//! `SampleFormat` keep the ALSA meanings so a ported tool reads side-by-side
//! with the original C.

use bitflags::bitflags;

/// Direction of a PCM, from the caller's point of view.
///
/// In ALSA terms this is `SND_PCM_STREAM_PLAYBACK` / `SND_PCM_STREAM_CAPTURE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PcmStream {
    Playback = 0,
    Capture = 1,
}

impl PcmStream {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Capture,
            _ => Self::Playback,
        }
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Playback => "playback",
            Self::Capture => "capture",
        }
    }
}

/// Access model of a PCM handle.
///
/// Mirrors ALSA's `SND_PCM_ACCESS_*` flags, reduced to the bits that change
/// behaviour at the framework level. The driver translates the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PcmAccess {
    /// Readable only (capture side).
    R = 1,
    /// Writable only (playback side).
    W = 2,
    /// Read and write.
    RW = 3,
    /// Samples of all channels are interleaved in one buffer.
    Interleaved = 0x100,
    /// Each channel has its own buffer region.
    NonInterleaved = 0x200,
    /// The buffer is a memory-mapped ring.
    Mmap = 0x400,
    /// I/O goes straight to the DMA ring.
    Direct = 0x800,
}

impl PcmAccess {
    #[inline]
    pub const fn bits(self) -> u32 {
        self as u32
    }

    /// Whether a handle opened with this access may write samples.
    #[inline]
    pub const fn is_writable(self) -> bool {
        (self as u32) & (Self::W as u32) != 0
    }

    /// Whether a handle opened with this access may read samples.
    #[inline]
    pub const fn is_readable(self) -> bool {
        (self as u32) & (Self::R as u32) != 0
    }

    /// Whether channels are interleaved; a non-interleaved stream keeps one
    /// region per channel instead.
    #[inline]
    pub const fn is_interleaved(self) -> bool {
        (self as u32) & (Self::Interleaved as u32) != 0
    }
}

/// Lifecycle state of a PCM.
///
/// This is the state machine every ALSA PCM moves through
/// (`SND_PCM_STATE_*`); the framework refuses transitions that do not follow
/// it, which is what keeps an xrun from being silently ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PcmState {
    Open = 0,
    Setup = 1,
    Prepared = 2,
    Running = 3,
    XRun = 4,
    Draining = 5,
    Paused = 6,
    Disconnected = 7,
    Closed = 8,
}

impl PcmState {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Setup,
            2 => Self::Prepared,
            3 => Self::Running,
            4 => Self::XRun,
            5 => Self::Draining,
            6 => Self::Paused,
            7 => Self::Disconnected,
            8 => Self::Closed,
            _ => Self::Open,
        }
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Setup => "setup",
            Self::Prepared => "prepared",
            Self::Running => "running",
            Self::XRun => "xrun",
            Self::Draining => "draining",
            Self::Paused => "paused",
            Self::Disconnected => "disconnected",
            Self::Closed => "closed",
        }
    }

    /// Whether the stream currently moves samples.
    #[inline]
    pub const fn is_running(self) -> bool {
        matches!(self, Self::Running | Self::Draining)
    }
}

bitflags! {
    /// Sample format, mirroring ALSA's `SND_PCM_FORMAT_*` family.
    ///
    /// The values are the ALSA ones on purpose: a ported `aplay` accepts the
    /// same `-F` names, and the driver validates them identically.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(C)]
    pub struct SampleFormat: u32 {
        const S8 = 0x0000_0001;
        const U8 = 0x0000_0002;
        const S16_LE = 0x0000_0010;
        const S16_BE = 0x0000_0020;
        const U16_LE = 0x0000_0030;
        const U16_BE = 0x0000_0040;
        const S24_LE = 0x0000_0100;
        const S24_BE = 0x0000_0200;
        const S24_3LE = 0x0000_0300;
        const S24_3BE = 0x0000_0400;
        const S32_LE = 0x0000_1000;
        const S32_BE = 0x0000_2000;
        const FLOAT_LE = 0x0001_0000;
        const FLOAT_BE = 0x0002_0000;
    }
}

impl SampleFormat {
    /// Storage width of one sample in bytes.
    ///
    /// This is the number that decides buffer arithmetic, so it is the piece
    /// of format knowledge the framework needs most.
    ///
    /// The ALSA format codes are not contiguous, so this matches explicitly
    /// rather than testing ranges: the 8-bit formats sit at 1..2, the 16-bit
    /// ones at 0x10..0x40 and the packed 24-bit at 0x300 / 0x400, which a
    /// naive range test would misclassify.
    #[inline]
    pub const fn sample_bytes(self) -> u32 {
        let bits = self.bits();
        if bits == SampleFormat::S8.bits() || bits == SampleFormat::U8.bits() {
            1
        } else if bits == SampleFormat::S16_LE.bits()
            || bits == SampleFormat::S16_BE.bits()
            || bits == SampleFormat::U16_LE.bits()
            || bits == SampleFormat::U16_BE.bits()
        {
            2
        } else if bits == SampleFormat::S24_3LE.bits() || bits == SampleFormat::S24_3BE.bits() {
            3
        } else {
            4
        }
    }

    /// Signed samples represent a bipolar signal, unsigned ones unipolar.
    ///
    /// The ALSA format codes are *overlapping*, not bit flags:
    /// `U16_LE` is `0x30`, which contains both `S16_LE` (`0x10`) and
    /// `S16_BE` (`0x20`). A bitmask test would therefore report every signed
    /// 16-bit format as unsigned, so each code is compared for exact equality.
    #[inline]
    pub const fn is_signed(self) -> bool {
        let bits = self.bits();
        bits != SampleFormat::U8.bits()
            && bits != SampleFormat::U16_LE.bits()
            && bits != SampleFormat::U16_BE.bits()
    }

    /// Whether samples are stored most-significant byte first.
    ///
    /// Exact comparisons, for the same reason as [`Self::is_signed`].
    #[inline]
    pub const fn is_big_endian(self) -> bool {
        let bits = self.bits();
        bits == SampleFormat::S16_BE.bits()
            || bits == SampleFormat::U16_BE.bits()
            || bits == SampleFormat::S24_BE.bits()
            || bits == SampleFormat::S24_3BE.bits()
            || bits == SampleFormat::S32_BE.bits()
            || bits == SampleFormat::FLOAT_BE.bits()
    }

    #[inline]
    pub const fn is_float(self) -> bool {
        let bits = self.bits();
        bits == SampleFormat::FLOAT_LE.bits() || bits == SampleFormat::FLOAT_BE.bits()
    }

    /// Bytes occupied by one frame of `channels` samples.
    #[inline]
    pub const fn frame_bytes(self, channels: u16) -> u32 {
        self.sample_bytes() * channels as u32
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        let bits = self.bits();
        if bits == Self::S8.bits() {
            "S8"
        } else if bits == Self::U8.bits() {
            "U8"
        } else if bits == Self::S16_LE.bits() {
            "S16_LE"
        } else if bits == Self::S16_BE.bits() {
            "S16_BE"
        } else if bits == Self::U16_LE.bits() {
            "U16_LE"
        } else if bits == Self::U16_BE.bits() {
            "U16_BE"
        } else if bits == Self::S24_LE.bits() {
            "S24_LE"
        } else if bits == Self::S24_BE.bits() {
            "S24_BE"
        } else if bits == Self::S24_3LE.bits() {
            "S24_3LE"
        } else if bits == Self::S24_3BE.bits() {
            "S24_3BE"
        } else if bits == Self::S32_LE.bits() {
            "S32_LE"
        } else if bits == Self::S32_BE.bits() {
            "S32_BE"
        } else if bits == Self::FLOAT_LE.bits() {
            "FLOAT_LE"
        } else if bits == Self::FLOAT_BE.bits() {
            "FLOAT_BE"
        } else {
            "UNKNOWN"
        }
    }
}

/// One PCM endpoint of a card, as reported by `AlsaPcmEnumerate`.
///
/// In ALSA a card is a device (an AC97 controller, an HDA codec) and a PCM is
/// one endpoint of it. Keeping the split means a card with four outputs
/// reports four `PcmInfo` rows that all share `card`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct PcmInfo {
    /// Card number, ALSA's `N` in `hw:N,M`.
    pub card: u32,
    /// PCM index within the card, ALSA's `M` in `hw:N,M`.
    pub device: u32,
    /// Sub-device index `P`; `0` for hardware without sub-devices.
    pub subdevice: u32,
    pub stream: PcmStream,
    /// Bitmask of `SampleFormat` values this endpoint accepts.
    pub formats: u32,
    /// Bitmask of `PcmAccess` values this endpoint supports.
    pub access: u32,
    /// Bitmask of channel positions (ALSA channel map bits).
    pub channels: u64,
    /// Lowest supported sample rate in Hz.
    pub rate_min: u32,
    /// Highest supported sample rate in Hz.
    pub rate_max: u32,
    /// Hard floor on the period size, in frames.
    pub periods_min: u32,
    /// Hard ceiling on the period size, in frames.
    pub periods_max: u32,
    /// Buffer size the endpoint prefers, in frames.
    pub buffer_size: u32,
}

/// The negotiated parameters of a prepared stream.
///
/// A ported `aplay` needs exactly these to compute transfer sizes, and they
/// match the fields it asks `snd_pcm_hw_params_get_*` for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct PcmParams {
    pub stream: PcmStream,
    pub access: u32,
    pub format: u32,
    /// Channel-position bitmask actually in use.
    pub channels: u64,
    pub rate: u32,
    /// Number of channels in use (not a mask, a count).
    pub channel_count: u16,
    pub _pad: u16,
    /// Period size in frames.
    pub period_size: u16,
    pub _pad2: u16,
    /// Buffer size in frames; always a whole number of periods.
    pub buffer_size: u32,
    pub _pad3: u32,
}

impl PcmParams {
    /// Bytes occupied by one frame under these parameters.
    #[inline]
    pub const fn frame_bytes(&self) -> u32 {
        SampleFormat::from_bits_retain(self.format).frame_bytes(self.channel_count)
    }

    /// Bytes occupied by `frames` frames under these parameters.
    #[inline]
    pub const fn bytes_for(&self, frames: u32) -> u32 {
        self.frame_bytes() * frames
    }
}

/// Request to open one PCM.
///
/// `flags` may carry `PcmOpenFlags::NO_AUTO_FORMAT` and friends; the driver
/// refuses an open it cannot satisfy instead of silently reconfiguring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct PcmOpenPayload {
    pub card: u32,
    pub device: u32,
    pub subdevice: u32,
    pub stream: PcmStream,
    pub access: u32,
    pub flags: u32,
}

bitflags! {
    /// Behaviour modifiers for a PCM open.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(C)]
    pub struct PcmOpenFlags: u32 {
        const NONE = 0;
        /// Do not insert format conversion layers; fail if the requested
        /// format is unsupported.
        const NO_AUTO_FORMAT = 1 << 0;
        /// Do not insert rate conversion.
        const NO_AUTO_RATE = 1 << 1;
        /// Do not insert channel remapping.
        const NO_AUTO_CHANNEL_MAP = 1 << 2;
    }
}

/// Query for the hardware parameters an endpoint advertises.
///
/// A `field` selects which interval the reply carries; the driver answers
/// with the full set of candidates, and the client narrows it down with
/// further queries, exactly like `snd_pcm_hw_params_any` -> `..._set_*` ->
/// `test_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct HwParamsQuery {
    /// PCM handle returned by `PcmOpen`.
    pub pcm: u32,
    /// One of `HwParamsQuery::FIELD_*`.
    pub field: u32,
    /// `1` to narrow the candidates with `value`, `0` to query.
    pub set: u32,
    /// Packed value: a rate, a format, an access mode or a channel count.
    pub value: u64,
}

impl HwParamsQuery {
    /// Number of distinct hw-params fields the framework understands.
    pub const FIELD_COUNT: u32 = 5;

    /// `ACCESS` - access model, `PcmAccess` in `value`.
    pub const FIELD_ACCESS: u32 = 0;
    /// `FORMAT` - sample format, `SampleFormat` bits in `value`.
    pub const FIELD_FORMAT: u32 = 1;
    /// `CHANNELS` - channel count, plain number in `value`.
    pub const FIELD_CHANNELS: u32 = 2;
    /// `RATE` - sample rate in Hz, plain number in `value`.
    pub const FIELD_RATE: u32 = 3;
    /// `PERIOD_SIZE` - period in frames, plain number in `value`.
    pub const FIELD_PERIOD_SIZE: u32 = 4;

    /// Build a query for one field.
    #[inline]
    pub const fn new(pcm: u32, field: u32) -> Self {
        Self { pcm, field, set: 0, value: 0 }
    }

    /// Build a query that narrows `field` to `value`.
    #[inline]
    pub const fn setting(pcm: u32, field: u32, value: u64) -> Self {
        Self { pcm, field, set: 1, value }
    }

    #[inline]
    pub const fn field_name(self) -> &'static str {
        match self.field {
            Self::FIELD_ACCESS => "access",
            Self::FIELD_FORMAT => "format",
            Self::FIELD_CHANNELS => "channels",
            Self::FIELD_RATE => "rate",
            Self::FIELD_PERIOD_SIZE => "period_size",
            _ => "unknown",
        }
    }
}

/// A candidate range, mirroring ALSA's `_snd_mask` / `_snd_interval`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Interval {
    pub min: u64,
    pub max: u64,
    /// Step between `min` and `max`; `0` means the value is not a range.
    pub step: u64,
}

impl Interval {
    pub const EMPTY: Self = Self { min: 0, max: 0, step: 0 };

    #[inline]
    pub const fn exact(value: u64) -> Self {
        Self { min: value, max: value, step: 0 }
    }

    #[inline]
    pub const fn range(min: u64, max: u64, step: u64) -> Self {
        Self { min, max, step }
    }

    #[inline]
    pub const fn contains(self, value: u64) -> bool {
        value >= self.min && value <= self.max
    }

    /// Whether the range describes exactly one value.
    #[inline]
    pub const fn is_single(self) -> bool {
        self.min == self.max
    }
}

/// Reply to a `HwParamsQuery` for one field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct HwParamsResult {
    /// Bitmask of candidate values, for the enumerated fields.
    pub values: u64,
    /// The value the driver would pick for a `set` request.
    pub chosen: u64,
    /// Bounds of the value range, for the numeric fields.
    pub interval: Interval,
}

/// Mixer control face, mirroring ALSA's `SND_CTL_ELEM_IFACE_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CtlInterface {
    /// Card-level mixer.
    Card = 0,
    /// A mixer routed to one PCM.
    Pcm = 1,
    /// Hardware-dependent metadata.
    Hwdep = 2,
    /// Jack / plug detection.
    Jack = 3,
    /// Raw device tree, as in `alsactl store`.
    Raw = 4,
    /// TLV metadata shared across interfaces.
    Tlv = 5,
}

impl CtlInterface {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Pcm,
            2 => Self::Hwdep,
            3 => Self::Jack,
            4 => Self::Raw,
            5 => Self::Tlv,
            _ => Self::Card,
        }
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Card => "card",
            Self::Pcm => "pcm",
            Self::Hwdep => "hwdep",
            Self::Jack => "jack",
            Self::Raw => "raw",
            Self::Tlv => "tlv",
        }
    }
}

/// Mixer element type, mirroring `SND_CTL_ELEM_TYPE_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CtlElemType {
    None = 0,
    Integer = 1,
    Enumerated = 2,
    Iec958 = 3,
    Tlv = 4,
}

impl CtlElemType {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Integer,
            2 => Self::Enumerated,
            3 => Self::Iec958,
            4 => Self::Tlv,
            _ => Self::None,
        }
    }
}

bitflags! {
    /// Read/write permissions of one control element.
    ///
    /// Mirrors ALSA's `SND_CTL_ELEM_ACCESS_*`; the driver rejects a write to
    /// an element that does not advertise `WRITE`, which is what makes a
    /// read-only hardware control safe to expose.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(C)]
    pub struct CtlAccess: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const RW = (1 << 0) | (1 << 1);
    }
}

/// One simple mixer element, as reported by `AlsaMixerEnumerate`.
///
/// This is the port of `snd_mixer_selem_id`: a name plus the coordinate that
/// identifies the element inside its card, so `amixer` can print and address
/// it exactly like the C tool does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct MixerSelem {
    pub interface: CtlInterface,
    pub elem_type: CtlElemType,
    pub access: CtlAccess,
    /// ALSA element index within the card.
    pub index: u32,
    /// Packed ALSA element name, zero-padded.
    pub name: [u8; 32],
    /// Playback / capture channel the element is bound to; `0xFFFF_FFFF` when
    /// the element is global.
    pub playback: u32,
    pub capture: u32,
}

impl MixerSelem {
    /// Number of bytes in [`Self::name`] that carry the name.
    pub const NAME_LEN: usize = 32;

    /// `playback` / `capture` sentinel meaning "not bound to a stream".
    pub const NO_CHANNEL: u32 = 0xFFFF_FFFF;

    /// Borrow the element name as `&str`, trimming the zero padding.
    #[inline]
    pub fn name_str(&self) -> &str {
        let end = self.name.iter().position(|&b| b == 0).unwrap_or(Self::NAME_LEN);
        core::str::from_utf8(&self.name[..end]).unwrap_or("")
    }

    /// Set the name from a `&str`, zero-padding the rest.
    #[inline]
    pub fn set_name(&mut self, value: &str) {
        self.name = [0u8; Self::NAME_LEN];
        let bytes = value.as_bytes();
        let len = bytes.len().min(Self::NAME_LEN);
        self.name[..len].copy_from_slice(&bytes[..len]);
    }

    /// Whether the element is bound to a specific playback stream.
    #[inline]
    pub const fn has_playback(&self) -> bool {
        self.playback != Self::NO_CHANNEL
    }

    /// Whether the element is bound to a specific capture stream.
    #[inline]
    pub const fn has_capture(&self) -> bool {
        self.capture != Self::NO_CHANNEL
    }

    /// Whether a write to this element is allowed at all.
    #[inline]
    pub const fn is_writable(&self) -> bool {
        self.access.contains(CtlAccess::WRITE)
    }
}

/// Current value of one mixer element.
///
/// The port of `snd_mixer_selem_get_*_dB` / `_volume`: raw integers in the
/// element's own unit, with the driver doing the dB conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct MixerValue {
    pub index: u32,
    pub channels: u32,
    /// Raw values, one per channel, in the element's own unit.
    pub values: [i32; 8],
}

impl MixerValue {
    /// Largest number of channels one element can carry on the wire.
    pub const MAX_CHANNELS: usize = 8;

    #[inline]
    pub const fn get(&self, channel: usize) -> Option<i32> {
        if channel >= self.channels as usize || channel >= Self::MAX_CHANNELS {
            None
        } else {
            Some(self.values[channel])
        }
    }

    #[inline]
    pub const fn set(&mut self, channel: usize, value: i32) -> bool {
        if channel >= self.channels as usize || channel >= Self::MAX_CHANNELS {
            false
        } else {
            self.values[channel] = value;
            true
        }
    }

    /// Lowest value across all channels, which is what a mono-style read
    /// reports: muting either channel should read as muted.
    pub fn min(&self) -> Option<i32> {
        let count = self.channels as usize;
        if count == 0 || count > Self::MAX_CHANNELS {
            return None;
        }
        let mut best = self.values[0];
        for channel in 1..count {
            if self.values[channel] < best {
                best = self.values[channel];
            }
        }
        Some(best)
    }

    /// Highest value across all channels, the mirror of [`Self::min`].
    pub fn max(&self) -> Option<i32> {
        let count = self.channels as usize;
        if count == 0 || count > Self::MAX_CHANNELS {
            return None;
        }
        let mut best = self.values[0];
        for channel in 1..count {
            if self.values[channel] > best {
                best = self.values[channel];
            }
        }
        Some(best)
    }
}

/// Jack / plug detection state, mirroring ALSA's switch API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct JackState {
    pub card: u32,
    pub index: u32,
    /// Non-zero when something is plugged in.
    pub present: u32,
    /// Bitmask of which function (headphone, mic, ...) is active.
    pub function: u32,
}

impl JackState {
    #[inline]
    pub const fn is_present(&self) -> bool {
        self.present != 0
    }
}

/// One hardware-dependent device, mirroring ALSA's hwdep interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct HwdepInfo {
    pub card: u32,
    pub device: u32,
    /// hwdep node index (`/dev/snd/hwdepC`).
    pub node: u32,
    /// Driver-private interface identifier.
    pub iface: u64,
    /// Vendor of the underlying device.
    pub vendor: u32,
    pub device_id: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_format_widths_match_the_c_layout() {
        assert_eq!(SampleFormat::S8.sample_bytes(), 1);
        assert_eq!(SampleFormat::U8.sample_bytes(), 1);
        assert_eq!(SampleFormat::S16_LE.sample_bytes(), 2);
        assert_eq!(SampleFormat::S16_BE.sample_bytes(), 2);
        assert_eq!(SampleFormat::U16_LE.sample_bytes(), 2);
        assert_eq!(SampleFormat::S24_3LE.sample_bytes(), 3);
        assert_eq!(SampleFormat::S24_LE.sample_bytes(), 4);
        assert_eq!(SampleFormat::S32_LE.sample_bytes(), 4);
        assert_eq!(SampleFormat::FLOAT_LE.sample_bytes(), 4);
    }

    #[test]
    fn sample_format_signedness_and_endianness() {
        assert!(SampleFormat::S16_LE.is_signed());
        assert!(!SampleFormat::U16_LE.is_signed());
        assert!(!SampleFormat::U8.is_signed());
        assert!(!SampleFormat::S16_LE.is_big_endian());
        assert!(SampleFormat::S16_BE.is_big_endian());
        assert!(SampleFormat::FLOAT_LE.is_float());
        assert!(!SampleFormat::S32_LE.is_float());
        assert_eq!(SampleFormat::S16_LE.as_str(), "S16_LE");
        assert_eq!(SampleFormat::empty().as_str(), "UNKNOWN");
    }

    #[test]
    fn frame_bytes_scales_with_channel_count() {
        assert_eq!(SampleFormat::S16_LE.frame_bytes(1), 2);
        assert_eq!(SampleFormat::S16_LE.frame_bytes(2), 4);
        assert_eq!(SampleFormat::S32_LE.frame_bytes(6), 24);
    }

    #[test]
    fn access_flags_answer_direction_questions() {
        assert!(PcmAccess::RW.is_readable());
        assert!(PcmAccess::RW.is_writable());
        assert!(!PcmAccess::RW.is_interleaved());
        assert!(PcmAccess::Interleaved.is_interleaved());
        assert!(!PcmAccess::W.is_readable());
    }

    #[test]
    fn mixer_name_is_trimmed_and_padded() {
        let mut selem = MixerSelem {
            interface: CtlInterface::Card,
            elem_type: CtlElemType::Integer,
            access: CtlAccess::RW,
            index: 0,
            name: [0u8; MixerSelem::NAME_LEN],
            playback: MixerSelem::NO_CHANNEL,
            capture: MixerSelem::NO_CHANNEL,
        };
        selem.set_name("Master");
        assert_eq!(selem.name_str(), "Master");
        assert_eq!(selem.name[6], 0);
        assert!(!selem.has_playback());
        assert!(selem.is_writable());
    }

    #[test]
    fn mixer_value_is_bounded_by_its_channel_count() {
        let mut value = MixerValue { index: 0, channels: 2, values: [10; 8] };
        assert!(value.set(0, 100));
        assert!(value.set(1, 50));
        assert!(!value.set(2, 5));
        assert_eq!(value.get(0), Some(100));
        assert_eq!(value.get(2), None);
        assert_eq!(value.min(), Some(50));
        assert_eq!(value.max(), Some(100));
    }

    #[test]
    fn mixer_value_rejects_nonsensical_channel_counts() {
        let empty = MixerValue { index: 0, channels: 0, values: [0; 8] };
        assert_eq!(empty.min(), None);
        assert_eq!(empty.max(), None);
        let over = MixerValue { index: 0, channels: 99, values: [0; 8] };
        assert_eq!(over.min(), None);
    }

    #[test]
    fn state_and_stream_words_round_trip() {
        for state in [PcmState::Open, PcmState::Running, PcmState::XRun, PcmState::Closed] {
            assert_eq!(PcmState::from_u32(state as u32), state);
        }
        assert!(PcmState::Running.is_running());
        assert!(PcmState::Draining.is_running());
        assert!(!PcmState::Paused.is_running());
        assert_eq!(PcmStream::from_u32(1), PcmStream::Capture);
        assert_eq!(PcmStream::Capture.as_str(), "capture");
    }

    #[test]
    fn hwparams_queries_are_named() {
        let query = HwParamsQuery::new(3, HwParamsQuery::FIELD_RATE);
        assert_eq!(query.field_name(), "rate");
        assert_eq!(query.set, 0);
        let set = HwParamsQuery::setting(3, HwParamsQuery::FIELD_FORMAT, 0x10);
        assert_eq!(set.set, 1);
        assert_eq!(set.value, 0x10);
        assert_eq!(HwParamsQuery::new(0, 99).field_name(), "unknown");
    }

    #[test]
    fn interval_contains_and_exact() {
        let interval = Interval::range(32, 4096, 32);
        assert!(interval.contains(64));
        assert!(interval.contains(32));
        assert!(!interval.contains(4097));
        assert!(!interval.is_single());
        assert!(Interval::exact(480).is_single());
        assert!(Interval::exact(480).contains(480));
        assert!(!Interval::exact(480).contains(481));
    }

    #[test]
    fn params_derive_buffer_arithmetic() {
        let params = PcmParams {
            stream: PcmStream::Playback,
            access: PcmAccess::Interleaved.bits(),
            format: SampleFormat::S16_LE.bits(),
            channels: 0b11,
            rate: 48000,
            channel_count: 2,
            _pad: 0,
            period_size: 1024,
            _pad2: 0,
            buffer_size: 4096,
            _pad3: 0,
        };
        assert_eq!(params.frame_bytes(), 4);
        assert_eq!(params.bytes_for(1024), 4096);
    }

    #[test]
    fn jack_presence_is_a_simple_predicate() {
        let jack = JackState { card: 0, index: 0, present: 1, function: 0 };
        assert!(jack.is_present());
        assert!(!JackState { present: 0, ..jack }.is_present());
    }
}
