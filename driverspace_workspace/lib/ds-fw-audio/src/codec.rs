//! Wire encoding and decoding for audio payloads.
//!
//! Every ALSA request and reply uses one convention (the same one as
//! `kapi_abi::payloads::sys::LogPayload` and the IOMMU payloads): `DsMsg::arg0`
//! carries the payload pointer and `DsMsg::arg1` the length.
//!
//! Serialization goes through the `Encoder` / `Decoder` provided by
//! `kapi_abi::wire`, so byte order and bounds checking match the rest of the
//! system and the framework and the driver never grow a second implementation.

use kapi_abi::{
    payloads::audio::{
        HwParamsResult, Interval, JackState, MixerSelem, MixerValue, PcmInfo, PcmOpenPayload,
        PcmParams,
    },
    wire::{Decoder, Encoder},
};

/// Worst-case encoded length of a single audio payload, in bytes.
///
/// `MixerSelem` is the largest: it carries a 32-byte name. The figure is the
/// field sum rounded up, and it is what the driver uses to size scratch
/// buffers.
pub const MAX_PAYLOAD_LEN: usize = 64;

/// Encode a `PcmInfo`.
pub fn encode_pcm_info(enc: &mut Encoder<'_>, value: &PcmInfo) -> bool {
    enc.write_u32(value.card)
        && enc.write_u32(value.device)
        && enc.write_u32(value.subdevice)
        && enc.write_u32(value.stream as u32)
        && enc.write_u32(value.formats)
        && enc.write_u32(value.access)
        && enc.write_u64(value.channels)
        && enc.write_u32(value.rate_min)
        && enc.write_u32(value.rate_max)
        && enc.write_u32(value.periods_min)
        && enc.write_u32(value.periods_max)
        && enc.write_u32(value.buffer_size)
}

/// Decode a `PcmInfo`.
pub fn decode_pcm_info(dec: &mut Decoder<'_>) -> Option<PcmInfo> {
    Some(PcmInfo {
        card: dec.read_u32()?,
        device: dec.read_u32()?,
        subdevice: dec.read_u32()?,
        stream: kapi_abi::payloads::audio::PcmStream::from_u32(dec.read_u32()?),
        formats: dec.read_u32()?,
        access: dec.read_u32()?,
        channels: dec.read_u64()?,
        rate_min: dec.read_u32()?,
        rate_max: dec.read_u32()?,
        periods_min: dec.read_u32()?,
        periods_max: dec.read_u32()?,
        buffer_size: dec.read_u32()?,
    })
}

/// Encode a `PcmOpenPayload`.
pub fn encode_open(enc: &mut Encoder<'_>, value: &PcmOpenPayload) -> bool {
    enc.write_u32(value.card)
        && enc.write_u32(value.device)
        && enc.write_u32(value.subdevice)
        && enc.write_u32(value.stream as u32)
        && enc.write_u32(value.access)
        && enc.write_u32(value.flags)
}

/// Decode a `PcmOpenPayload`.
pub fn decode_open(dec: &mut Decoder<'_>) -> Option<PcmOpenPayload> {
    Some(PcmOpenPayload {
        card: dec.read_u32()?,
        device: dec.read_u32()?,
        subdevice: dec.read_u32()?,
        stream: kapi_abi::payloads::audio::PcmStream::from_u32(dec.read_u32()?),
        access: dec.read_u32()?,
        flags: dec.read_u32()?,
    })
}

/// Encode a `PcmParams`.
pub fn encode_params(enc: &mut Encoder<'_>, value: &PcmParams) -> bool {
    enc.write_u32(value.stream as u32)
        && enc.write_u32(value.access)
        && enc.write_u32(value.format)
        && enc.write_u64(value.channels)
        && enc.write_u32(value.rate)
        && enc.write_u16(value.channel_count)
        && enc.write_u16(value.period_size)
        && enc.write_u32(value.buffer_size)
}

/// Decode a `PcmParams`.
pub fn decode_params(dec: &mut Decoder<'_>) -> Option<PcmParams> {
    Some(PcmParams {
        stream: kapi_abi::payloads::audio::PcmStream::from_u32(dec.read_u32()?),
        access: dec.read_u32()?,
        format: dec.read_u32()?,
        channels: dec.read_u64()?,
        rate: dec.read_u32()?,
        channel_count: dec.read_u16()?,
        _pad: 0,
        period_size: dec.read_u16()?,
        _pad2: 0,
        buffer_size: dec.read_u32()?,
        _pad3: 0,
    })
}

/// Encode a `HwParamsResult`.
pub fn encode_hw_params_result(enc: &mut Encoder<'_>, value: &HwParamsResult) -> bool {
    enc.write_u64(value.values)
        && enc.write_u64(value.chosen)
        && enc.write_u64(value.interval.min)
        && enc.write_u64(value.interval.max)
        && enc.write_u64(value.interval.step)
}

/// Decode a `HwParamsResult`.
pub fn decode_hw_params_result(dec: &mut Decoder<'_>) -> Option<HwParamsResult> {
    Some(HwParamsResult {
        values: dec.read_u64()?,
        chosen: dec.read_u64()?,
        interval: Interval {
            min: dec.read_u64()?,
            max: dec.read_u64()?,
            step: dec.read_u64()?,
        },
    })
}

/// Encode a `MixerSelem`.
pub fn encode_mixer_selem(enc: &mut Encoder<'_>, value: &MixerSelem) -> bool {
    enc.write_u32(value.interface as u32)
        && enc.write_u32(value.elem_type as u32)
        && enc.write_u32(value.access.bits())
        && enc.write_u32(value.index)
        && enc.write_bytes(&value.name)
        && enc.write_u32(value.playback)
        && enc.write_u32(value.capture)
}

/// Decode a `MixerSelem`.
///
/// Fields are read in exactly the order `encode_mixer_selem` writes them:
/// interface, type, access, index, the 32-byte name, then the two channel
/// coordinates. Reading them out of order would silently mis-assign fields
/// that share a width.
pub fn decode_mixer_selem(dec: &mut Decoder<'_>) -> Option<MixerSelem> {
    use kapi_abi::payloads::audio::{CtlElemType, CtlInterface};
    let interface = CtlInterface::from_u32(dec.read_u32()?);
    let elem_type = CtlElemType::from_u32(dec.read_u32()?);
    let access = kapi_abi::payloads::audio::CtlAccess::from_bits_truncate(dec.read_u32()?);
    let index = dec.read_u32()?;
    let name = dec.read_bytes(MixerSelem::NAME_LEN)?;
    let playback = dec.read_u32()?;
    let capture = dec.read_u32()?;

    let mut selem = MixerSelem {
        interface,
        elem_type,
        access,
        index,
        name: [0u8; MixerSelem::NAME_LEN],
        playback,
        capture,
    };
    selem.name.copy_from_slice(name);
    Some(selem)
}

/// Encode a `MixerValue`.
pub fn encode_mixer_value(enc: &mut Encoder<'_>, value: &MixerValue) -> bool {
    enc.write_u32(value.index) && enc.write_u32(value.channels) && {
        let mut ok = true;
        for item in value.values.iter() {
            ok &= enc.write_u32(*item as u32);
        }
        ok
    }
}

/// Decode a `MixerValue`.
pub fn decode_mixer_value(dec: &mut Decoder<'_>) -> Option<MixerValue> {
    let index = dec.read_u32()?;
    let channels = dec.read_u32()?;
    let mut values = [0i32; MixerValue::MAX_CHANNELS];
    for slot in values.iter_mut() {
        *slot = dec.read_u32()? as i32;
    }
    Some(MixerValue { index, channels, values })
}

/// Encode a `JackState`.
pub fn encode_jack_state(enc: &mut Encoder<'_>, value: &JackState) -> bool {
    enc.write_u32(value.card)
        && enc.write_u32(value.index)
        && enc.write_u32(value.present)
        && enc.write_u32(value.function)
}

/// Decode a `JackState`.
pub fn decode_jack_state(dec: &mut Decoder<'_>) -> Option<JackState> {
    Some(JackState {
        card: dec.read_u32()?,
        index: dec.read_u32()?,
        present: dec.read_u32()?,
        function: dec.read_u32()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kapi_abi::payloads::audio::{CtlAccess, CtlElemType, CtlInterface, PcmStream, SampleFormat};

    /// Encode with `f` into a full-size scratch buffer, returning the bytes
    /// written, or `None` when the payload did not fit.
    fn encoded(f: impl FnOnce(&mut Encoder<'_>) -> bool) -> (usize, [u8; MAX_PAYLOAD_LEN]) {
        let mut buf = [0u8; MAX_PAYLOAD_LEN];
        let mut enc = Encoder::new(&mut buf);
        assert!(f(&mut enc), "payload exceeded MAX_PAYLOAD_LEN");
        (enc.pos(), buf)
    }

    fn pcm_info() -> PcmInfo {
        PcmInfo {
            card: 1,
            device: 2,
            subdevice: 3,
            stream: PcmStream::Playback,
            formats: SampleFormat::S32_LE.bits(),
            access: 0x101,
            channels: 0x33,
            rate_min: 44100,
            rate_max: 96000,
            periods_min: 128,
            periods_max: 2048,
            buffer_size: 8192,
        }
    }

    fn selem() -> MixerSelem {
        let mut selem = MixerSelem {
            interface: CtlInterface::Pcm,
            elem_type: CtlElemType::Integer,
            access: CtlAccess::READ,
            index: 9,
            name: [0u8; MixerSelem::NAME_LEN],
            playback: 0,
            capture: MixerSelem::NO_CHANNEL,
        };
        selem.set_name("Headphone");
        selem
    }

    #[test]
    fn pcm_info_round_trips() {
        let info = pcm_info();
        let (len, buf) = encoded(|e| encode_pcm_info(e, &info));
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_pcm_info(&mut dec).unwrap();
        assert_eq!(back.card, info.card);
        assert_eq!(back.device, info.device);
        assert_eq!(back.subdevice, info.subdevice);
        assert_eq!(back.stream, info.stream);
        assert_eq!(back.formats, info.formats);
        assert_eq!(back.channels, info.channels);
        assert_eq!(back.rate_max, info.rate_max);
        assert_eq!(back.periods_min, info.periods_min);
        assert_eq!(back.buffer_size, info.buffer_size);
    }

    #[test]
    fn mixer_selem_round_trips_including_the_name() {
        let (len, buf) = encoded(|e| encode_mixer_selem(e, &selem()));
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_mixer_selem(&mut dec).unwrap();
        assert_eq!(back.interface, CtlInterface::Pcm);
        assert_eq!(back.elem_type, CtlElemType::Integer);
        assert_eq!(back.index, 9);
        assert_eq!(back.name_str(), "Headphone");
        assert_eq!(back.playback, 0);
        assert_eq!(back.capture, MixerSelem::NO_CHANNEL);
        assert!(!back.is_writable());
    }

    #[test]
    fn mixer_value_round_trips_negative_numbers() {
        let value = MixerValue { index: 2, channels: 2, values: [-40, 40, 0, 0, 0, 0, 0, 0] };
        let (len, buf) = encoded(|e| encode_mixer_value(e, &value));
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_mixer_value(&mut dec).unwrap();
        assert_eq!(back.index, 2);
        assert_eq!(back.channels, 2);
        assert_eq!(back.get(0), Some(-40));
        assert_eq!(back.get(1), Some(40));
        assert_eq!(back.get(2), None);
    }

    #[test]
    fn hw_params_result_round_trips() {
        let result = HwParamsResult {
            values: 0xFFFF,
            chosen: 48000,
            interval: Interval::range(8000, 48000, 100),
        };
        let (len, buf) = encoded(|e| encode_hw_params_result(e, &result));
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_hw_params_result(&mut dec).unwrap();
        assert_eq!(back.values, 0xFFFF);
        assert_eq!(back.chosen, 48000);
        assert_eq!(back.interval.min, 8000);
        assert_eq!(back.interval.max, 48000);
        assert_eq!(back.interval.step, 100);
    }

    #[test]
    fn params_round_trip() {
        let params = PcmParams {
            stream: PcmStream::Capture,
            access: 0x100,
            format: SampleFormat::S16_LE.bits(),
            channels: 0b11,
            rate: 48000,
            channel_count: 2,
            _pad: 0,
            period_size: 512,
            _pad2: 0,
            buffer_size: 2048,
            _pad3: 0,
        };
        let (len, buf) = encoded(|e| encode_params(e, &params));
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_params(&mut dec).unwrap();
        assert_eq!(back.stream, PcmStream::Capture);
        assert_eq!(back.rate, 48000);
        assert_eq!(back.channel_count, 2);
        assert_eq!(back.period_size, 512);
        assert_eq!(back.buffer_size, 2048);
        assert_eq!(back.frame_bytes(), 4);
    }

    #[test]
    fn open_and_jack_round_trip() {
        let open = PcmOpenPayload {
            card: 0,
            device: 1,
            subdevice: 0,
            stream: PcmStream::Playback,
            access: 0x103,
            flags: 0,
        };
        let (len, buf) = encoded(|e| encode_open(e, &open));
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_open(&mut dec).unwrap();
        assert_eq!(back.device, 1);
        assert_eq!(back.stream, PcmStream::Playback);
        assert_eq!(back.access, 0x103);

        let jack = JackState { card: 0, index: 2, present: 1, function: 0b101 };
        let (len, buf) = encoded(|e| encode_jack_state(e, &jack));
        let mut dec = Decoder::new(&buf[..len]);
        let back = decode_jack_state(&mut dec).unwrap();
        assert_eq!(back.index, 2);
        assert!(back.is_present());
        assert_eq!(back.function, 0b101);
    }

    #[test]
    fn truncated_payloads_are_rejected() {
        let (len, buf) = encoded(|e| encode_jack_state(e, &JackState {
            card: 0,
            index: 1,
            present: 1,
            function: 0,
        }));
        // A full buffer decodes; a short one must yield `None`, never a panic.
        assert!(decode_jack_state(&mut Decoder::new(&buf[..len])).is_some());
        assert!(decode_jack_state(&mut Decoder::new(&buf[..2])).is_none());
    }

    #[test]
    fn encoding_into_a_tiny_buffer_fails_cleanly() {
        // An undersized buffer must report failure instead of writing past it.
        let mut small = [0u8; 2];
        let mut enc = Encoder::new(&mut small);
        assert!(!encode_pcm_info(&mut enc, &pcm_info()));
    }
}
