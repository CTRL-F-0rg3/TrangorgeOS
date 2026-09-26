//! The ported user-space tools: `aplay` and `amixer`.
//!
//! The parsing and formatting logic lives here rather than in the binaries so
//! it can be unit-tested without an IPC transport. The binaries are thin
//! wrappers that parse `argv`, call into this module, and report the outcome.
//!
//! Keeping ALSA's own vocabulary is the point: `aplay -D hw:0 -d 0 -f S16_LE
//! -r 48000 -c 2` means the same thing here as it does on Linux, so existing
//! scripts and documentation carry over.

use kapi_abi::payloads::audio::{SampleFormat, PcmStream};
use core::str::FromStr;

/// The device-name prefix, i.e. which PCM plugin the caller asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceKind {
    /// Direct hardware access, no conversion.
    Hardware,
    /// The plugger may insert format and rate conversion.
    Plug,
    /// Whatever the manager considers default.
    Default,
}

/// Which device a tool should talk to.
///
/// The port of ALSA's `snd_pcm_name_parse`: `hw:0,0`, `plughw:0`, `default`.
/// A ported tool accepts the same spellings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceName {
    /// Which plugin layer the caller named.
    pub kind: DeviceKind,
    pub card: u32,
    pub device: u32,
    pub subdevice: u32,
}

impl DeviceName {
    /// Parse an ALSA PCM name.
    ///
    /// Accepted forms, matching `snd_pcm_name_parse`:
    /// `default`, `hw`, `hw:N`, `hw:N,M`, `hw:N,M,P`, `plughw:...`.
    /// Returns `None` for anything else, so a typo is rejected rather than
    /// silently resolved to some other device.
    pub fn parse(name: &str) -> Option<Self> {
        if name.is_empty() {
            return None;
        }
        let (kind, rest) = if let Some(rest) = name.strip_prefix("plughw:") {
            (DeviceKind::Plug, rest)
        } else if let Some(rest) = name.strip_prefix("plug:") {
            (DeviceKind::Plug, rest)
        } else if let Some(rest) = name.strip_prefix("hw:") {
            (DeviceKind::Hardware, rest)
        } else if name == "default" {
            return Some(Self {
                kind: DeviceKind::Default,
                card: 0,
                device: 0,
                subdevice: 0,
            });
        } else if name == "hw" {
            (DeviceKind::Hardware, "")
        } else if name == "plug" || name == "plughw" {
            (DeviceKind::Plug, "")
        } else {
            return None;
        };

        let mut parts = rest.split(',');
        let card = match parts.next() {
            None | Some("") => 0,
            Some(value) => parse_index(value)?,
        };
        let device = match parts.next() {
            None | Some("") => 0,
            Some(value) => parse_index(value)?,
        };
        let subdevice = match parts.next() {
            None | Some("") => 0,
            Some(value) => parse_index(value)?,
        };
        if parts.next().is_some() {
            // A fourth component means the caller typed a name we do not model.
            return None;
        }
        Some(Self { kind, card, device, subdevice })
    }

    /// Render the name back to ALSA's spelling.
    ///
    /// The return type is `heapless::String` because the crate is `no_std`
    /// and must not pull in an allocator just to format a device name.
    pub fn as_string(self) -> heapless::String<32> {
        let prefix = match self.kind {
            DeviceKind::Hardware => "hw",
            DeviceKind::Plug => "plughw",
            DeviceKind::Default => "default",
        };
        match self.kind {
            DeviceKind::Default => heapless::String::from_str("default").unwrap_or_default(),
            _ => {
                let mut out = heapless::String::new();
                // `card` and `device` are u32, so at most 10 digits each; the
                // buffer is sized so this cannot fail.
                let _ = core::fmt::Write::write_fmt(
                    &mut out,
                    format_args!("{prefix}:{},{}", self.card, self.device),
                );
                out
            }
        }
    }
}

/// Parse one decimal component of a device name.
fn parse_index(value: &str) -> Option<u32> {
    if value.is_empty() {
        return Some(0);
    }
    let mut result: u32 = 0;
    for byte in value.bytes() {
        if !byte.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((byte - b'0') as u32)?;
    }
    Some(result)
}

/// Whether a parsed name may convert format, rate or channels.
///
/// `plughw` may; `hw` may not, which is the whole point of the distinction in
/// ALSA: `hw` surfaces exactly what the hardware does.
pub fn allows_conversion(device: DeviceName) -> bool {
    matches!(device.kind, DeviceKind::Plug | DeviceKind::Default)
}

/// The direction a tool plays in, as ALSA's `--playback` / `--capture`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Playback,
    Capture,
}

impl Direction {
    #[inline]
    pub const fn stream(self) -> PcmStream {
        match self {
            Self::Playback => PcmStream::Playback,
            Self::Capture => PcmStream::Capture,
        }
    }

    /// ALSA's access bit for this direction.
    pub const fn access_bit(self) -> u32 {
        match self {
            Self::Playback => 0x0002, // SND_PCM_ACCESS_RWWRITE, write side
            Self::Capture => 0x0001,  // read side
        }
    }
}

/// The fully negotiated parameter set an `aplay` invocation asks for.
///
/// This is `aplay`'s whole configuration in one value: it is filled from the
/// command line, checked against what the hardware reports, and then applied
/// field by field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaybackRequest {
    pub device: DeviceName,
    pub direction: Direction,
    pub format: SampleFormat,
    pub channels: u16,
    pub rate: u32,
    pub period_size: u16,
    /// `None` lets the driver pick.
    pub buffer_size: Option<u32>,
    /// Recover automatically on an xrun, as `aplay -X` does by default.
    pub recover_on_xrun: bool,
}

impl Default for PlaybackRequest {
    /// The defaults `aplay` itself uses: 48 kHz stereo 16-bit, 1024-frame
    /// periods, four-period buffer, xrun recovery on.
    fn default() -> Self {
        Self {
            device: DeviceName {
                kind: DeviceKind::Default,
                card: 0,
                device: 0,
                subdevice: 0,
            },
            direction: Direction::Playback,
            format: SampleFormat::S16_LE,
            channels: 2,
            rate: 48000,
            period_size: 1024,
            buffer_size: None,
            recover_on_xrun: true,
        }
    }
}

impl PlaybackRequest {
    /// Bytes one period of audio occupies, the number `aplay` reports.
    pub const fn period_bytes(&self) -> u32 {
        self.format.frame_bytes(self.channels) * self.period_size as u32
    }

    /// Duration of one period, in microseconds.
    ///
    /// `aplay` prints this in its setup summary; it is also the latency the
    /// user actually feels.
    pub const fn period_us(&self) -> u64 {
        if self.rate == 0 {
            0
        } else {
            (self.period_size as u64 * 1_000_000) / self.rate as u64
        }
    }

    /// The buffer size to use, defaulting to four periods as ALSA does.
    pub const fn effective_buffer_size(&self) -> u32 {
        match self.buffer_size {
            Some(size) => size,
            None => self.period_size as u32 * 4,
        }
    }
}

/// What a mixer command asked for: `amixer set Master 80%` and friends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MixerCommand {
    /// Print every element, as `amixer` with no arguments.
    List,
    /// Set one element to a percentage of full scale.
    SetPercent { name: [u8; 32], percent: i32 },
    /// Set one element to mute or unmute.
    SetSwitch { name: [u8; 32], on: bool },
    /// Read one element.
    Get { name: [u8; 32] },
}

impl MixerCommand {
    /// Build a fixed-size, zero-padded element name.
    pub fn name_field(name: &str) -> [u8; 32] {
        let mut field = [0u8; 32];
        let bytes = name.as_bytes();
        let len = bytes.len().min(32);
        field[..len].copy_from_slice(&bytes[..len]);
        field
    }

    /// Whether the command modifies state, and so needs write permission.
    pub const fn is_write(&self) -> bool {
        matches!(self, Self::SetPercent { .. } | Self::SetSwitch { .. })
    }
}

/// Clamp a requested percentage into the 0..=100 range `amixer` accepts.
pub const fn clamp_percent(percent: i32) -> i32 {
    if percent < 0 {
        0
    } else if percent > 100 {
        100
    } else {
        percent
    }
}

/// What `aplay` should do once its options are parsed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AplayAction {
    /// Print the usage text and exit successfully.
    Help,
    /// List the PCM endpoints, as `aplay -l` does.
    ListPcms,
    /// Print the parameters that would be negotiated, as `--dump-hw-params` does.
    DumpParams(PlaybackRequest),
    /// Play the named file with these parameters.
    Play(PlaybackRequest),
    /// The options were not understood; report `message` and fail.
    Invalid { option: [u8; 48] },
}

/// A borrowed C string, for reading the argument vector the loader published.
pub type ArgRef<'a> = &'a str;

/// Parse `aplay`'s arguments into an [`AplayAction`].
///
/// `args` excludes the program name, exactly like the C tool's `argv[1..]`.
pub fn parse_aplay(args: &[ArgRef<'_>]) -> AplayAction {
    let mut request = PlaybackRequest::default();
    let mut file = heapless::String::<64>::new();
    let mut list = false;
    let mut dump = false;

    let mut index = 0;
    while index < args.len() {
        let arg = args[index];
        index += 1;

        // `aplay` accepts `-D hw:0,0`, `-Dhw:0,0` and `-D=hw:0,0`. `read_value`
        // resolves all three, consuming the following argument when needed.
        if let Some(value) = read_value(args, &mut index, arg, "-D") {
            match DeviceName::parse(value) {
                Some(device) => request.device = device,
                None => return invalid(arg),
            }
        } else if let Some(value) = read_value(args, &mut index, arg, "-d") {
            match parse_u32(value) {
                Some(device) => request.device.device = device,
                None => return invalid(arg),
            }
        } else if let Some(value) = read_value(args, &mut index, arg, "-f") {
            match parse_format(value) {
                Some(format) => request.format = format,
                None => return invalid(arg),
            }
        } else if let Some(value) = read_value(args, &mut index, arg, "-r") {
            match parse_u32(value) {
                Some(rate) => request.rate = rate,
                None => return invalid(arg),
            }
        } else if let Some(value) = read_value(args, &mut index, arg, "-c") {
            match parse_u32(value) {
                Some(channels) => request.channels = channels as u16,
                None => return invalid(arg),
            }
        } else if let Some(value) = read_value(args, &mut index, arg, "-p") {
            match parse_u32(value) {
                Some(period) => request.period_size = period as u16,
                None => return invalid(arg),
            }
        } else if let Some(value) = read_value(args, &mut index, arg, "-b") {
            match parse_u32(value) {
                Some(buffer) => request.buffer_size = Some(buffer),
                None => return invalid(arg),
            }
        } else if arg == "-l" || arg == "--list" {
            list = true;
        } else if arg == "--dump-hw-params" {
            dump = true;
        } else if arg == "--capture" {
            request.direction = Direction::Capture;
        } else if arg == "--playback" {
            request.direction = Direction::Playback;
        } else if arg == "-h" || arg == "--help" {
            return AplayAction::Help;
        } else if arg.starts_with('-') {
            return invalid(arg);
        } else if file.is_empty() {
            let _ = file.push_str(arg);
        }
    }

    if list {
        return AplayAction::ListPcms;
    }
    if dump {
        return AplayAction::DumpParams(request);
    }
    if file.is_empty() {
        // A bare `aplay` with nothing to play is a usage error in the C tool
        // too, so it is reported rather than silently listing devices.
        return invalid("aplay: no input file");
    }
    AplayAction::Play(request)
}

/// Build the rejection action for a bad option.
fn invalid(option: &str) -> AplayAction {
    let mut field = [0u8; 48];
    let bytes = option.as_bytes();
    let len = bytes.len().min(48);
    field[..len].copy_from_slice(&bytes[..len]);
    AplayAction::Invalid { option: field }
}

/// Read the value of option `name` from `arg`, in any spelling `aplay` accepts.
///
/// Three forms work, and the C tool accepts all three:
///
/// * `-Dvalue`   - value attached
/// * `-D=value`  - value after an equals sign
/// * `-D value`  - value in the next argument, which this consumes
///
/// A bare flag (`-l`) yields `None` and consumes nothing, so the next
/// argument is left for the loop. `index` is advanced only when an argument
/// is actually taken.
fn read_value<'a>(
    args: &[ArgRef<'a>],
    index: &mut usize,
    arg: &'a str,
    name: &str,
) -> Option<&'a str> {
    let rest = arg.strip_prefix(name)?;
    if !rest.is_empty() {
        return Some(rest.strip_prefix('=').unwrap_or(rest));
    }
    // Bare option: the value, if any, is the next argument.
    let next = args.get(*index)?;
    *index += 1;
    Some(*next)
}

/// Parse a decimal `u32`, rejecting anything else.
fn parse_u32(value: &str) -> Option<u32> {
    if value.is_empty() {
        return None;
    }
    let mut result: u32 = 0;
    for byte in value.bytes() {
        if !byte.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((byte - b'0') as u32)?;
    }
    Some(result)
}

/// What `amixer` should do once its subcommand is parsed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmixerAction {
    /// Print the usage text and exit successfully.
    Help,
    /// List every simple element.
    List,
    /// Read one element by index.
    Get { index: u32 },
    /// Set one element to a level, already clamped to 0..=100.
    Set { index: u32, percent: i32 },
    /// The command was not understood.
    Invalid { message: [u8; 48] },
}

/// Parse `amixer`'s arguments into an [`AmixerAction`].
///
/// Accepted forms, matching the C tool:
///
/// ```text
/// amixer
/// amixer get <element>
/// amixer set <element> <percent>%
/// amixer set <element> mute|unmute
/// ```
///
/// Elements are addressed by their index in enumeration order, which is what
/// the driver reports; the C tool's name lookup is a convenience layer on top
/// of the same index and is not part of the wire contract.
pub fn parse_amixer(args: &[ArgRef<'_>]) -> AmixerAction {
    let Some(first) = args.first() else {
        return AmixerAction::List;
    };
    let first: &str = first;
    if first == "-h" || first == "--help" {
        return AmixerAction::Help;
    }
    if first != "get" && first != "set" {
        return amixer_invalid(first);
    }

    // `get <index>` or `set <index> <value>`.
    let Some(index_text) = args.get(1) else {
        return amixer_invalid(first);
    };
    let Some(index) = parse_u32(index_text) else {
        return amixer_invalid(index_text);
    };

    if first == "get" {
        return AmixerAction::Get { index };
    }

    let Some(value_text) = args.get(2) else {
        return amixer_invalid(index_text);
    };
    // `80%` and a bare `80` mean the same, as in the C tool.
    let numeric: &str = value_text.strip_suffix('%').unwrap_or(value_text);
    match parse_u32(numeric) {
        Some(percent) => {
            AmixerAction::Set { index, percent: clamp_percent(percent as i32) }
        }
        // `mute` / `unmute` map onto the ends of the range, which is what the
        // hardware switch does.
        None if numeric == "mute" => AmixerAction::Set { index, percent: 0 },
        None if numeric == "unmute" => AmixerAction::Set { index, percent: 100 },
        None => amixer_invalid(value_text),
    }
}

/// Build the rejection action with a message.
fn amixer_invalid(text: &str) -> AmixerAction {
    let mut field = [0u8; 48];
    let bytes = text.as_bytes();
    let len = bytes.len().min(48);
    field[..len].copy_from_slice(&bytes[..len]);
    AmixerAction::Invalid { message: field }
}

/// Parse an ALSA sample-format name such as `S16_LE`.
pub fn parse_format(name: &str) -> Option<SampleFormat> {
    let candidates = [
        ("S8", SampleFormat::S8),
        ("U8", SampleFormat::U8),
        ("S16_LE", SampleFormat::S16_LE),
        ("S16_BE", SampleFormat::S16_BE),
        ("U16_LE", SampleFormat::U16_LE),
        ("U16_BE", SampleFormat::U16_BE),
        ("S24_LE", SampleFormat::S24_LE),
        ("S24_BE", SampleFormat::S24_BE),
        ("S24_3LE", SampleFormat::S24_3LE),
        ("S24_3BE", SampleFormat::S24_3BE),
        ("S32_LE", SampleFormat::S32_LE),
        ("S32_BE", SampleFormat::S32_BE),
        ("FLOAT_LE", SampleFormat::FLOAT_LE),
        ("FLOAT_BE", SampleFormat::FLOAT_BE),
    ];
    candidates
        .iter()
        .find(|(text, _)| *text == name)
        .map(|(_, value)| *value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_names_parse_like_alsa() {
        assert_eq!(DeviceName::parse("default").unwrap().kind, DeviceKind::Default);
        assert_eq!(DeviceName::parse("hw:0,0").unwrap().card, 0);
        assert_eq!(DeviceName::parse("hw:1,2").unwrap().device, 2);
        assert_eq!(DeviceName::parse("plughw:3,1").unwrap().kind, DeviceKind::Plug);
        // Bare and partial forms default the missing components to zero.
        assert_eq!(DeviceName::parse("hw").unwrap().card, 0);
        assert_eq!(DeviceName::parse("hw:2").unwrap().device, 0);
        assert_eq!(DeviceName::parse("hw:0,0,1").unwrap().subdevice, 1);
    }

    #[test]
    fn bad_device_names_are_rejected() {
        assert!(DeviceName::parse("").is_none());
        assert!(DeviceName::parse("nonsense").is_none());
        assert!(DeviceName::parse("hw:a").is_none());
        assert!(DeviceName::parse("hw:0,0,0,0").is_none());
        assert!(DeviceName::parse("hw:99999999999").is_none());
    }

    #[test]
    fn device_names_round_trip() {
        let name = DeviceName::parse("hw:1,2").unwrap();
        assert_eq!(name.as_string().as_str(), "hw:1,2");
        assert_eq!(DeviceName::parse("default").unwrap().as_string().as_str(), "default");
    }

    #[test]
    fn only_plug_may_convert() {
        assert!(allows_conversion(DeviceName::parse("plughw:0,0").unwrap()));
        assert!(allows_conversion(DeviceName::parse("default").unwrap()));
        assert!(!allows_conversion(DeviceName::parse("hw:0,0").unwrap()));
    }

    #[test]
    fn direction_maps_to_alsa_access_bits() {
        assert_eq!(Direction::Playback.stream(), PcmStream::Playback);
        assert_eq!(Direction::Capture.stream(), PcmStream::Capture);
        assert_eq!(Direction::Playback.access_bit(), 0x0002);
        assert_eq!(Direction::Capture.access_bit(), 0x0001);
    }

    #[test]
    fn playback_defaults_match_aplay() {
        let request = PlaybackRequest::default();
        assert_eq!(request.rate, 48000);
        assert_eq!(request.channels, 2);
        assert_eq!(request.format, SampleFormat::S16_LE);
        assert_eq!(request.period_size, 1024);
        assert!(request.recover_on_xrun);
        assert_eq!(request.effective_buffer_size(), 4096);
        // 1024 frames of stereo S16 is 4096 bytes, ~21.3 ms at 48 kHz.
        assert_eq!(request.period_bytes(), 4096);
        assert_eq!(request.period_us(), 21333);
    }

    #[test]
    fn period_math_scales_with_format_and_channels() {
        let mut request = PlaybackRequest {
            format: SampleFormat::S32_LE,
            channels: 2,
            period_size: 512,
            ..PlaybackRequest::default()
        };
        assert_eq!(request.period_bytes(), 4096);
        request.channels = 6;
        assert_eq!(request.period_bytes(), 12288);
    }

    #[test]
    fn mixer_commands_classify_reads_and_writes() {
        let name = MixerCommand::name_field("Master");
        assert_eq!(core::str::from_utf8(&name[..6]).unwrap(), "Master");
        assert!(MixerCommand::SetPercent { name, percent: 50 }.is_write());
        assert!(MixerCommand::SetSwitch { name, on: true }.is_write());
        assert!(!MixerCommand::Get { name }.is_write());
        assert!(!MixerCommand::List.is_write());
    }

    #[test]
    fn percentages_are_clamped() {
        assert_eq!(clamp_percent(-10), 0);
        assert_eq!(clamp_percent(0), 0);
        assert_eq!(clamp_percent(50), 50);
        assert_eq!(clamp_percent(100), 100);
        assert_eq!(clamp_percent(200), 100);
    }

    #[test]
    fn format_names_parse_like_aplay() {
        assert_eq!(parse_format("S16_LE"), Some(SampleFormat::S16_LE));
        assert_eq!(parse_format("FLOAT_BE"), Some(SampleFormat::FLOAT_BE));
        assert_eq!(parse_format("s16_le"), None, "ALSA names are case-sensitive");
        assert_eq!(parse_format("MP3"), None);
    }

    #[test]
    fn aplay_parses_a_full_command_line() {
        let args = ["-D", "hw:0,0", "-d", "1", "-f", "S32_LE", "-r", "44100", "-c", "2", "song.raw"];
        match parse_aplay(&args) {
            AplayAction::Play(request) => {
                assert_eq!(request.device.card, 0);
                // `-d 1` overrides the device from the name, as the C tool does.
                assert_eq!(request.device.device, 1);
                assert_eq!(request.format, SampleFormat::S32_LE);
                assert_eq!(request.rate, 44100);
                assert_eq!(request.channels, 2);
            }
            other => panic!("expected Play, got {other:?}"),
        }
    }

    #[test]
    fn aplay_accepts_both_option_spellings() {
        let joined = ["-Dhw:1,2", "-r=48000", "-c=2", "a.raw"];
        match parse_aplay(&joined) {
            AplayAction::Play(request) => {
                assert_eq!(request.device.card, 1);
                assert_eq!(request.device.device, 2);
                assert_eq!(request.rate, 48000);
            }
            other => panic!("expected Play, got {other:?}"),
        }
    }

    #[test]
    fn aplay_flags_are_not_mistaken_for_values() {
        // `-l` must not swallow the following file name.
        match parse_aplay(&["-l"]) {
            AplayAction::ListPcms => {}
            other => panic!("expected ListPcms, got {other:?}"),
        }
    }

    #[test]
    fn aplay_reports_bad_options() {
        assert!(matches!(parse_aplay(&["--nonsense"]), AplayAction::Invalid { .. }));
        assert!(matches!(parse_aplay(&["-f", "MP3", "a.raw"]), AplayAction::Invalid { .. }));
        assert!(matches!(parse_aplay(&["-r", "fast", "a.raw"]), AplayAction::Invalid { .. }));
        assert!(matches!(parse_aplay(&["-D", "bogus", "a.raw"]), AplayAction::Invalid { .. }));
        // No file at all is a usage error, not a silent no-op.
        assert!(matches!(parse_aplay(&[]), AplayAction::Invalid { .. }));
    }

    #[test]
    fn aplay_help_and_dump_are_distinct_actions() {
        assert_eq!(parse_aplay(&["-h"]), AplayAction::Help);
        assert_eq!(parse_aplay(&["--help"]), AplayAction::Help);
        match parse_aplay(&["--dump-hw-params"]) {
            AplayAction::DumpParams(request) => assert_eq!(request.rate, 48000),
            other => panic!("expected DumpParams, got {other:?}"),
        }
    }

    #[test]
    fn amixer_with_no_arguments_lists() {
        assert_eq!(parse_amixer(&[]), AmixerAction::List);
    }

    #[test]
    fn amixer_get_and_set_parse() {
        assert_eq!(parse_amixer(&["get", "0"]), AmixerAction::Get { index: 0 });
        assert_eq!(
            parse_amixer(&["set", "1", "80%"]),
            AmixerAction::Set { index: 1, percent: 80 }
        );
        // A bare number means the same as one with a percent sign.
        assert_eq!(
            parse_amixer(&["set", "1", "80"]),
            AmixerAction::Set { index: 1, percent: 80 }
        );
    }

    #[test]
    fn amixer_mute_maps_onto_the_range_ends() {
        assert_eq!(
            parse_amixer(&["set", "0", "mute"]),
            AmixerAction::Set { index: 0, percent: 0 }
        );
        assert_eq!(
            parse_amixer(&["set", "0", "unmute"]),
            AmixerAction::Set { index: 0, percent: 100 }
        );
    }

    #[test]
    fn amixer_clamps_and_rejects() {
        // Out-of-range levels clamp rather than wrap.
        assert_eq!(
            parse_amixer(&["set", "0", "250"]),
            AmixerAction::Set { index: 0, percent: 100 }
        );
        assert!(matches!(parse_amixer(&["frobnicate"]), AmixerAction::Invalid { .. }));
        assert!(matches!(parse_amixer(&["get"]), AmixerAction::Invalid { .. }));
        assert!(matches!(parse_amixer(&["set", "0"]), AmixerAction::Invalid { .. }));
        assert!(matches!(parse_amixer(&["set", "x", "10"]), AmixerAction::Invalid { .. }));
        assert_eq!(parse_amixer(&["--help"]), AmixerAction::Help);
    }
}
