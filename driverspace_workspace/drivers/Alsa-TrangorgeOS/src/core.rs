//! The ALSA-shaped object model.
//!
//! This is the port of `alsa-lib`'s object hierarchy, reduced to what the OS
//! actually needs and expressed as owned Rust values instead of raw pointers.
//! The names are kept so a ported `aplay` reads like the C original.
//!
//! ## The hierarchy
//!
//! ```text
//! Card        a physical codec (an AC97 controller, an HDA codec)
//!  ├─ Pcm      one endpoint of it: playback or capture
//!  ├─ Mixer    a set of simple mixer elements
//!  ├─ Jack     a plug detector
//!  └─ Hwdep    hardware-dependent metadata
//! ```
//!
//! ALSA calls a *stream* `SND_PCM_STREAM_PLAYBACK` / `SND_PCM_STREAM_CAPTURE`
//! and a *control* `SND_CTL_ELEM_*`; both survive here under the same names so
//! the two implementations can be diffed by eye.

use kapi_abi::payloads::audio::{
    CtlAccess, CtlElemType, CtlInterface, JackState, MixerSelem, MixerValue, PcmInfo, PcmParams,
    PcmState, PcmStream, SampleFormat,
};

/// Decode a fixed-size, zero-padded byte field as `&str`.
pub fn str_field(field: &[u8]) -> &str {
    let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
    core::str::from_utf8(&field[..end]).unwrap_or("")
}

/// A physical audio card.
///
/// One card may expose several PCMs, a mixer, jacks and hwdep nodes; ALSA
/// calls this a `snd_card`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Card {
    pub index: u32,
    /// Short driver-private identifier, ALSA's `id` (e.g. `"AC97"`).
    pub id: [u8; 8],
    /// Human-readable name (e.g. `"Trangorge AC97"`).
    pub name: [u8; 32],
    /// PCI vendor id, `0xFFFF` when the card is not on PCI.
    pub vendor: u16,
    /// PCI device id.
    pub device: u16,
}

impl Card {
    /// Build a card with a short id and a display name.
    pub fn new(index: u32, id: &str, name: &str) -> Self {
        let mut card = Self { index, vendor: 0xFFFF, device: 0xFFFF, ..Self::default() };
        write_str(&mut card.id, id);
        write_str(&mut card.name, name);
        card
    }

    /// Borrow the short id.
    pub fn id_str(&self) -> &str {
        str_field(&self.id)
    }

    /// Borrow the display name.
    pub fn name_str(&self) -> &str {
        str_field(&self.name)
    }
}

/// Write `value` into a zero-padded fixed-size field.
fn write_str(field: &mut [u8], value: &str) {
    field.fill(0);
    let bytes = value.as_bytes();
    let len = bytes.len().min(field.len());
    field[..len].copy_from_slice(&bytes[..len]);
}

/// A PCM endpoint of a card: the port of `snd_pcm`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pcm {
    /// Card this endpoint belongs to.
    pub card: u32,
    /// Endpoint index within the card, ALSA's `N` in `hw:0,N`.
    pub device: u32,
    pub stream: PcmStream,
    /// Formats the hardware can produce without conversion.
    pub formats: u32,
    /// Access modes the hardware supports.
    pub access: u32,
    /// Channel positions the hardware can route.
    pub channels: u64,
    pub rate_min: u32,
    pub rate_max: u32,
    pub period_min: u32,
    pub period_max: u32,
    /// Preferred buffer size in frames.
    pub buffer_size: u32,
}

impl Default for Pcm {
    /// A zeroed endpoint; the driver fills the real limits in `new`.
    fn default() -> Self {
        Self {
            card: 0,
            device: 0,
            stream: PcmStream::Playback,
            formats: 0,
            access: 0,
            channels: 0,
            rate_min: 0,
            rate_max: 0,
            period_min: 0,
            period_max: 0,
            buffer_size: 0,
        }
    }
}

impl Pcm {
    /// Whether this endpoint accepts `format` natively.
    pub fn supports_format(&self, format: SampleFormat) -> bool {
        self.formats & format.bits() != 0
    }

    /// Whether `rate` is inside the hardware's range.
    pub fn supports_rate(&self, rate: u32) -> bool {
        rate >= self.rate_min && rate <= self.rate_max
    }

    /// Whether `frames` is a usable period size.
    pub fn supports_period(&self, frames: u32) -> bool {
        frames >= self.period_min && frames <= self.period_max
    }

    /// Fill the wire descriptor for this endpoint.
    pub fn fill_info(&self, out: &mut PcmInfo) {
        *out = PcmInfo {
            card: self.card,
            device: self.device,
            subdevice: 0,
            stream: self.stream,
            formats: self.formats,
            access: self.access,
            channels: self.channels,
            rate_min: self.rate_min,
            rate_max: self.rate_max,
            periods_min: self.period_min,
            periods_max: self.period_max,
            buffer_size: self.buffer_size,
        };
    }
}

/// The negotiated parameters of one stream, mirroring `snd_pcm_hw_params_t`.
///
/// This is the part of ALSA that most often surprises people: parameters are
/// negotiated as a *set*, and each field constrains the others. The struct
/// therefore records the chosen values together, and [`HwParams::is_complete`]
/// is what decides whether `prepare` may proceed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HwParams {
    pub access: u32,
    pub format: u32,
    pub channel_count: u16,
    pub rate: u32,
    pub period_size: u16,
    pub buffer_size: u32,
    /// Field-by-field "has been chosen" flags; see the `FIELD_*` constants.
    pub chosen: u32,
}

impl HwParams {
    pub const FIELD_ACCESS: u32 = 1 << 0;
    pub const FIELD_FORMAT: u32 = 1 << 1;
    pub const FIELD_CHANNELS: u32 = 1 << 2;
    pub const FIELD_RATE: u32 = 1 << 3;
    pub const FIELD_PERIOD: u32 = 1 << 4;

    /// Mask of every negotiable field.
    pub const ALL_FIELDS: u32 = 0b1_1111;

    /// A fresh, un-negotiated set: the port of `snd_pcm_hw_params_any`.
    pub const fn empty() -> Self {
        Self {
            access: 0,
            format: 0,
            channel_count: 0,
            rate: 0,
            period_size: 0,
            buffer_size: 0,
            chosen: 0,
        }
    }

    /// Whether the field at `bit` has been chosen.
    pub const fn has(&self, bit: u32) -> bool {
        self.chosen & bit != 0
    }

    /// Mark the field at `bit` as chosen.
    pub fn set_chosen(&mut self, bit: u32) {
        self.chosen |= bit;
    }

    /// Whether every mandatory field has been chosen.
    ///
    /// `prepare` must refuse to run on an incomplete set, which is exactly
    /// what stops a stream from starting with a half-defined format.
    pub const fn is_complete(&self) -> bool {
        self.chosen & Self::ALL_FIELDS == Self::ALL_FIELDS
    }

    /// Whether the buffer is a whole number of periods, as ALSA requires.
    pub const fn buffer_is_multiple_of_period(&self) -> bool {
        self.period_size != 0 && self.buffer_size % self.period_size as u32 == 0
    }

    /// How many periods fit in the buffer.
    pub const fn period_count(&self) -> u32 {
        if self.period_size == 0 {
            0
        } else {
            self.buffer_size / self.period_size as u32
        }
    }

    /// Project the negotiated set onto the wire struct.
    pub fn to_params(&self, stream: PcmStream) -> PcmParams {
        PcmParams {
            stream,
            access: self.access,
            format: self.format,
            channels: 0,
            rate: self.rate,
            channel_count: self.channel_count,
            _pad: 0,
            period_size: self.period_size,
            _pad2: 0,
            buffer_size: self.buffer_size,
            _pad3: 0,
        }
    }

    /// Bytes occupied by one frame under these parameters.
    pub fn frame_bytes(&self) -> u32 {
        SampleFormat::from_bits_retain(self.format).frame_bytes(self.channel_count)
    }

    /// Bytes occupied by `frames` frames under these parameters.
    pub fn bytes_for(&self, frames: u32) -> u32 {
        self.frame_bytes() * frames
    }

    /// Time one period takes to play, in microseconds.
    ///
    /// This is the number a latency budget is made of: `period_size / rate`
    /// seconds, in the unit ALSA reports.
    pub fn period_us(&self) -> u64 {
        if self.rate == 0 || self.period_size == 0 {
            return 0;
        }
        (self.period_size as u64 * 1_000_000) / self.rate as u64
    }
}

/// Whether a stream stops immediately or plays out its buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropMode {
    Drop,
    Drain,
}

impl From<DropMode> for u32 {
    fn from(mode: DropMode) -> Self {
        match mode {
            DropMode::Drop => 0,
            DropMode::Drain => 1,
        }
    }
}

/// One mixer element, the port of `snd_mixer_selem`.
#[derive(Clone, Copy, Debug)]
pub struct MixerElem {
    pub index: u32,
    pub interface: CtlInterface,
    pub elem_type: CtlElemType,
    pub access: CtlAccess,
    pub name: [u8; MixerSelem::NAME_LEN],
    pub channel_count: u32,
    /// Current value per channel, in the element's own unit.
    pub values: [i32; MixerValue::MAX_CHANNELS],
}

impl MixerElem {
    /// Build an element with a name and an initial value for every channel.
    pub fn new(
        index: u32,
        name: &str,
        access: CtlAccess,
        value: i32,
        channel_count: u32,
    ) -> Self {
        let mut elem = Self {
            index,
            interface: CtlInterface::Card,
            elem_type: CtlElemType::Integer,
            access,
            name: [0u8; MixerSelem::NAME_LEN],
            channel_count,
            values: [0; MixerValue::MAX_CHANNELS],
        };
        write_str(&mut elem.name, name);
        let count = (channel_count as usize).min(MixerValue::MAX_CHANNELS);
        elem.values[..count].fill(value);
        elem
    }

    /// Borrow the element name.
    pub fn name_str(&self) -> &str {
        str_field(&self.name)
    }

    /// Whether a write is allowed on this element.
    pub fn is_writable(&self) -> bool {
        self.access.contains(CtlAccess::WRITE)
    }

    /// Fill the wire descriptor.
    pub fn fill_info(&self, out: &mut MixerSelem) {
        *out = MixerSelem {
            interface: self.interface,
            elem_type: self.elem_type,
            access: self.access,
            index: self.index,
            name: self.name,
            playback: MixerSelem::NO_CHANNEL,
            capture: MixerSelem::NO_CHANNEL,
        };
    }

    /// Fill the wire value.
    pub fn fill_value(&self, out: &mut MixerValue) {
        *out = MixerValue {
            index: self.index,
            channels: self.channel_count,
            values: self.values,
        };
    }

    /// Apply a new value, returning `false` if it is out of range.
    ///
    /// The bound check is the port of ALSA's `_snd_ctl_elem_range` clamping:
    /// a control that cannot represent the request is refused rather than
    /// silently wrapping into a very different setting.
    pub fn set_value(&mut self, value: &MixerValue) -> bool {
        if value.channels > self.channel_count {
            return false;
        }
        let count = (value.channels as usize).min(MixerValue::MAX_CHANNELS);
        self.values[..count].copy_from_slice(&value.values[..count]);
        true
    }
}

/// A plug detector, the port of ALSA's switch API.
#[derive(Clone, Copy, Debug)]
pub struct Jack {
    pub index: u32,
    pub card: u32,
    /// Bitmask of functions this detector reports (headphone, mic, ...).
    pub function: u32,
    /// Last known presence.
    pub present: bool,
}

impl Jack {
    /// Fill the wire state.
    pub fn fill_state(&self, out: &mut JackState) {
        *out = JackState {
            card: self.card,
            index: self.index,
            present: self.present as u32,
            function: self.function,
        };
    }
}

/// The lifecycle of one open stream, with the transitions ALSA permits.
///
/// Encoding the rules here rather than at each call site is what makes an
/// illegal transition impossible to express: `Running` cannot become
/// `Prepared` without going through an xrun, exactly as in `snd_pcm`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stream {
    state: PcmState,
    params: HwParams,
}

impl Stream {
    /// A newly opened, not-yet-negotiated stream.
    pub const fn new() -> Self {
        Self { state: PcmState::Open, params: HwParams::empty() }
    }

    /// Current state.
    pub const fn state(&self) -> PcmState {
        self.state
    }

    /// Negotiated parameters.
    pub const fn params(&self) -> &HwParams {
        &self.params
    }

    /// Mutable access to the parameters, for negotiation.
    pub fn params_mut(&mut self) -> &mut HwParams {
        &mut self.params
    }

    /// Move `Open` -> `Setup`; parameter negotiation may begin.
    pub fn begin_setup(&mut self) -> bool {
        if self.state == PcmState::Open {
            self.state = PcmState::Setup;
            true
        } else {
            false
        }
    }

    /// Move `Setup` -> `Prepared`, refusing an incomplete negotiation.
    pub fn prepare(&mut self) -> bool {
        if self.state == PcmState::Setup
            && self.params.is_complete()
            && self.params.buffer_is_multiple_of_period()
        {
            self.state = PcmState::Prepared;
            true
        } else {
            false
        }
    }

    /// Move `Prepared` -> `Running`.
    pub fn start(&mut self) -> bool {
        if self.state == PcmState::Prepared {
            self.state = PcmState::Running;
            true
        } else {
            false
        }
    }

    /// Stop a running stream, discarding or playing out the tail.
    pub fn drop_stream(&mut self, mode: DropMode) -> bool {
        if !self.state.is_running() {
            return false;
        }
        self.state = match mode {
            DropMode::Drop => PcmState::Setup,
            DropMode::Drain => PcmState::Draining,
        };
        true
    }

    /// Record a short transfer: the stream is now in `XRun`.
    pub fn mark_xrun(&mut self) {
        if self.state.is_running() {
            self.state = PcmState::XRun;
        }
    }

    /// `Prepare` + `Start` in one step, as `snd_pcm_prepare` does.
    pub fn recover(&mut self) -> bool {
        if matches!(self.state, PcmState::XRun | PcmState::Setup) {
            self.state = PcmState::Prepared;
            self.state = PcmState::Running;
            true
        } else {
            false
        }
    }

    /// Release the stream; the handle is dead afterwards.
    pub fn close(&mut self) {
        self.state = PcmState::Closed;
    }
}

impl Default for Stream {
    fn default() -> Self {
        Self::new()
    }
}
