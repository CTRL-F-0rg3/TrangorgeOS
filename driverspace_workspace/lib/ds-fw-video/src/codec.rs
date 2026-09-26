//! Wire encoding and decoding for video payloads.
//!
//! Every request and reply uses one convention - the same one as
//! `kapi_abi::payloads::sys::LogPayload` and the ALSA payloads: `DsMsg::arg0`
//! carries the payload pointer and `DsMsg::arg1` the length.
//!
//! Serialization goes through the `Encoder` / `Decoder` provided by
//! `kapi_abi::wire`, so byte order and bounds checking match the rest of the
//! system and the framework never grows a second implementation.

use kapi_abi::{
    payloads::video::{
        EngineInfo, FrameFormat, FrameInfo, SessionRequest, SessionState, SinkRequest,
        SubmitRequest,
    },
    wire::{Decoder, Encoder},
};

/// Worst-case encoded length of a single video payload, in bytes.
///
/// `FrameInfo` is the largest: a fourcc, a stride, two dimensions and two
/// timestamps, with the buffer handle and the ticket. The figure is the field
/// sum rounded up, and it is what the driver sizes scratch buffers with.
pub const MAX_PAYLOAD_LEN: usize = 64;

/// Encode an `EngineInfo`.
pub fn encode_engine_info(enc: &mut Encoder<'_>, value: &EngineInfo) -> bool {
    enc.write_u32(value.index)
        && enc.write_u32(value.caps)
        && enc.write_u32(value.max_width)
        && enc.write_u32(value.max_height)
        && enc.write_u32(value.max_surfaces)
        && enc.write_u64(value.dedicated_memory)
        && enc.write_u16(value.vendor_id)
        && enc.write_u16(value.device_id)
}

/// Decode an `EngineInfo`.
pub fn decode_engine_info(dec: &mut Decoder<'_>) -> Option<EngineInfo> {
    Some(EngineInfo {
        index: dec.read_u32()?,
        caps: dec.read_u32()?,
        max_width: dec.read_u32()?,
        max_height: dec.read_u32()?,
        max_surfaces: dec.read_u32()?,
        dedicated_memory: dec.read_u64()?,
        vendor_id: dec.read_u16()?,
        device_id: dec.read_u16()?,
    })
}

/// Encode a `SessionRequest`.
pub fn encode_session_request(enc: &mut Encoder<'_>, value: &SessionRequest) -> bool {
    enc.write_u32(value.engine)
        && enc.write_u32(value.codec)
        && enc.write_u32(value.out_format)
        && enc.write_u32(value.surfaces)
        && enc.write_u32(value.encode)
        && enc.write_u32(value.low_latency)
}

/// Decode a `SessionRequest`.
pub fn decode_session_request(dec: &mut Decoder<'_>) -> Option<SessionRequest> {
    Some(SessionRequest {
        engine: dec.read_u32()?,
        codec: dec.read_u32()?,
        out_format: dec.read_u32()?,
        surfaces: dec.read_u32()?,
        encode: dec.read_u32()?,
        low_latency: dec.read_u32()?,
    })
}

/// Encode a `SubmitRequest`.
///
/// The compressed bytes are *not* in the payload: this describes where they
/// live. They are in shared memory, and a client hands the driver a physical
/// address so the engine can DMA them straight out of a mapped ring rather than
/// through a bounce buffer in the middle.
pub fn encode_submit_request(enc: &mut Encoder<'_>, value: &SubmitRequest) -> bool {
    enc.write_u32(value.session)
        && enc.write_u32(value.size)
        && enc.write_u64(value.src_phys)
        && enc.write_u64(value.src_virt)
        && enc.write_u64(value.pts_ns)
        && enc.write_u32(value.keyframe)
}

/// Decode a `SubmitRequest`.
pub fn decode_submit_request(dec: &mut Decoder<'_>) -> Option<SubmitRequest> {
    Some(SubmitRequest {
        session: dec.read_u32()?,
        size: dec.read_u32()?,
        src_phys: dec.read_u64()?,
        src_virt: dec.read_u64()?,
        pts_ns: dec.read_u64()?,
        keyframe: dec.read_u32()?,
    })
}

/// Encode a `FrameInfo`.
///
/// The `present` flag is the first field on the wire, so a client reading the
/// first four bytes of a reply can tell "there is a frame" from "there is not"
/// without decoding the rest. That is the whole reason for the flag.
pub fn encode_frame_info(enc: &mut Encoder<'_>, value: &FrameInfo) -> bool {
    enc.write_u32(value.present)
        && enc.write_u32(value.buffer)
        && enc.write_u32(value.ticket)
        && encode_format(enc, &value.format)
        && enc.write_u64(value.pts_ns)
        && enc.write_u64(value.dts_ns)
        && enc.write_u32(value.late)
}

/// Decode a `FrameInfo`.
pub fn decode_frame_info(dec: &mut Decoder<'_>) -> Option<FrameInfo> {
    Some(FrameInfo {
        present: dec.read_u32()?,
        buffer: dec.read_u32()?,
        ticket: dec.read_u32()?,
        format: decode_format(dec)?,
        pts_ns: dec.read_u64()?,
        dts_ns: dec.read_u64()?,
        late: dec.read_u32()?,
    })
}

/// Encode a `FrameFormat`.
pub fn encode_format(enc: &mut Encoder<'_>, value: &FrameFormat) -> bool {
    enc.write_u32(value.format.0)
        && enc.write_u32(value.stride)
        && enc.write_u32(value.width)
        && enc.write_u32(value.height)
}

/// Decode a `FrameFormat`.
pub fn decode_format(dec: &mut Decoder<'_>) -> Option<FrameFormat> {
    Some(FrameFormat {
        format: kapi_abi::payloads::video::Fourcc(dec.read_u32()?),
        stride: dec.read_u32()?,
        width: dec.read_u32()?,
        height: dec.read_u32()?,
    })
}

/// Encode a `SessionState` as a single `u32`.
pub fn encode_session_state(enc: &mut Encoder<'_>, value: &SessionState) -> bool {
    enc.write_u32(*value as u32)
}

/// Decode a `SessionState`.
pub fn decode_session_state(dec: &mut Decoder<'_>) -> Option<SessionState> {
    Some(SessionState::from_u32(dec.read_u32()?))
}

/// Encode a `SinkRequest`.
pub fn encode_sink_request(enc: &mut Encoder<'_>, value: &SinkRequest) -> bool {
    enc.write_u32(value.purpose) && enc.write_u32(value.push) && enc.write_u32(value.depth)
}

/// Decode a `SinkRequest`.
pub fn decode_sink_request(dec: &mut Decoder<'_>) -> Option<SinkRequest> {
    Some(SinkRequest {
        purpose: dec.read_u32()?,
        push: dec.read_u32()?,
        depth: dec.read_u32()?,
    })
}
