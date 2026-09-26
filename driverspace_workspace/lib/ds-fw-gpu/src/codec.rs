//! Payload encoding for the GPU opcodes.
//!
//! ## Why this is hand-written
//!
//! A GPU payload is a fixed-layout `#[repr(C)]` struct, so on the target it
//! could be transmuted straight into the message. This module does it by hand
//! anyway, for three reasons that each have bitten a driver before:
//!
//! 1. The host build that runs these tests is not the target, so a
//!    `transmute` would be tested against the wrong layout and break on the
//!    device that matters.
//! 2. A decode must reject a short or over-long payload. Reading a `#[repr(C)]`
//!    struct out of a slice is a read past its end; returning an error is not.
//! 3. Shader source is variable-length. A fixed-struct encode cannot express
//!    "header plus source", and that opcode has to.
//!
//! Every function is total: it returns `Result` rather than panicking, because
//! the input is a message off a socket and a malformed one is routine.
//!
//! Layout is little-endian throughout, matching the target and `DsMsg` itself.

use alloc::vec::Vec;

use kapi_abi::{
    DsError,
    payloads::gpu::{
        GpuBuffer, GpuBufferFlags, GpuBufferRequest, GpuContextInfo, GpuContextRequest, GpuFence,
        GpuInfo, GpuMode, GpuShader, GpuShaderFlags, GpuShaderRequest, GpuSubmit, GpuVendor,
    },
    wire::Decoder,
};

/// The largest payload a GPU opcode will carry.
///
/// Generous enough for a `GpuInfo` plus a large shader header, small enough
/// that a hostile `source_len` cannot make the driver allocate without bound.
pub const MAX_PAYLOAD_LEN: usize = 1 << 20;

/// The largest shader source accepted, 64 KiB.
///
/// A real vertex or fragment program is orders of magnitude smaller; anything
/// past this is a client that is not going to compile, and refusing it here is
/// cheaper than failing inside a compiler.
pub const MAX_SHADER_SOURCE: usize = 64 * 1024;

/// Decode `bytes` as a `GpuInfo`.
pub fn decode_device(bytes: &[u8]) -> Result<GpuInfo, DsError> {
    if bytes.len() < 44 {
        return Err(DsError::InvalidMessage);
    }
    let mut d = Decoder::new(bytes);
    let index = d.read_u32().ok_or(DsError::InvalidMessage)?;
    let vendor = d.read_u32().ok_or(DsError::InvalidMessage)?;
    let vendor_id = d.read_u16().ok_or(DsError::InvalidMessage)?;
    let device_id = d.read_u16().ok_or(DsError::InvalidMessage)?;
    let caps = d.read_u64().ok_or(DsError::InvalidMessage)?;
    let vram_size = d.read_u64().ok_or(DsError::InvalidMessage)?;
    let mmio_size = d.read_u64().ok_or(DsError::InvalidMessage)?;
    let clock_mhz = d.read_u32().ok_or(DsError::InvalidMessage)?;
    let engines = d.read_u32().ok_or(DsError::InvalidMessage)?;
    Ok(GpuInfo {
        index,
        vendor: GpuVendor::from_u32(vendor),
        vendor_id,
        device_id,
        caps,
        vram_size,
        mmio_size,
        clock_mhz,
        engines,
    })
}

/// Encode a `GpuInfo` into `out`, returning its length.
pub fn encode_device(info: &GpuInfo, out: &mut Vec<u8>) -> Result<usize, DsError> {
    push_u32(out, info.index);
    push_u32(out, info.vendor as u32);
    push_u16(out, info.vendor_id);
    push_u16(out, info.device_id);
    push_u64(out, info.caps);
    push_u64(out, info.vram_size);
    push_u64(out, info.mmio_size);
    push_u32(out, info.clock_mhz);
    push_u32(out, info.engines);
    finish(out)
}

/// Encode a `GpuMode`.
pub fn encode_mode(mode: &GpuMode, out: &mut Vec<u8>) -> Result<usize, DsError> {
    push_u32(out, mode.width);
    push_u32(out, mode.height);
    push_u32(out, mode.format as u32);
    push_u32(out, mode.stride);
    push_u32(out, mode.refresh_mhz);
    finish(out)
}

/// Decode a `GpuMode`.
pub fn decode_mode(bytes: &[u8]) -> Result<GpuMode, DsError> {
    if bytes.len() < 20 {
        return Err(DsError::InvalidMessage);
    }
    let mut d = Decoder::new(bytes);
    let width = d.read_u32().ok_or(DsError::InvalidMessage)?;
    let height = d.read_u32().ok_or(DsError::InvalidMessage)?;
    let format = d.read_u32().ok_or(DsError::InvalidMessage)?;
    let stride = d.read_u32().ok_or(DsError::InvalidMessage)?;
    let refresh_mhz = d.read_u32().ok_or(DsError::InvalidMessage)?;
    Ok(GpuMode { width, height, format: kapi_abi::payloads::gpu::GpuFormat::from_u32(format), stride, refresh_mhz })
}

/// Encode a `GpuContextInfo`.
pub fn encode_context_info(info: &GpuContextInfo, out: &mut Vec<u8>) -> Result<usize, DsError> {
    push_u32(out, info.handle);
    push_u32(out, info.engines);
    push_u64(out, info.ring_size);
    push_u64(out, info.ring_tail_usable);
    finish(out)
}

/// Decode a `GpuContextRequest`.
pub fn decode_context_request(bytes: &[u8]) -> Result<GpuContextRequest, DsError> {
    if bytes.len() < 24 {
        return Err(DsError::InvalidMessage);
    }
    let mut d = Decoder::new(bytes);
    Ok(GpuContextRequest {
        device: d.read_u32().ok_or(DsError::InvalidMessage)?,
        engines: d.read_u32().ok_or(DsError::InvalidMessage)?,
        shmem_base: d.read_u64().ok_or(DsError::InvalidMessage)?,
        shmem_size: d.read_u64().ok_or(DsError::InvalidMessage)?,
    })
}

/// Decode a `GpuBufferRequest`.
pub fn decode_buffer_request(bytes: &[u8]) -> Result<GpuBufferRequest, DsError> {
    if bytes.len() < 16 {
        return Err(DsError::InvalidMessage);
    }
    let mut d = Decoder::new(bytes);
    let size = d.read_u64().ok_or(DsError::InvalidMessage)?;
    let flags = d.read_u32().ok_or(DsError::InvalidMessage)?;
    Ok(GpuBufferRequest { size, flags: GpuBufferFlags::from_bits_truncate(flags), _pad: 0 })
}

/// Encode a `GpuBuffer`.
pub fn encode_buffer(buffer: &GpuBuffer, out: &mut Vec<u8>) -> Result<usize, DsError> {
    push_u32(out, buffer.handle);
    push_u32(out, 0);
    push_u64(out, buffer.phys);
    push_u64(out, buffer.size);
    push_u64(out, buffer.virt);
    finish(out)
}

/// Decode a `GpuSubmit`.
///
/// The command bytes are *not* here: a submission names a range of the
/// client's ring that the GPU can read directly.
pub fn decode_submit(bytes: &[u8]) -> Result<GpuSubmit, DsError> {
    if bytes.len() < 24 {
        return Err(DsError::InvalidMessage);
    }
    let mut d = Decoder::new(bytes);
    Ok(GpuSubmit {
        context: d.read_u32().ok_or(DsError::InvalidMessage)?,
        offset: d.read_u32().ok_or(DsError::InvalidMessage)?,
        length: d.read_u32().ok_or(DsError::InvalidMessage)?,
        ring: d.read_u32().ok_or(DsError::InvalidMessage)?,
        engines: d.read_u32().ok_or(DsError::InvalidMessage)?,
        _pad: 0,
    })
}

/// Encode a `GpuFence`.
pub fn encode_fence(fence: &GpuFence, out: &mut Vec<u8>) -> Result<usize, DsError> {
    push_u64(out, fence.value);
    push_u32(out, fence.context);
    push_u32(out, fence.signalled);
    push_u32(out, 0);
    finish(out)
}

/// Encode a `GpuShader`.
pub fn encode_shader(shader: &GpuShader, out: &mut Vec<u8>) -> Result<usize, DsError> {
    push_u32(out, shader.handle);
    push_u32(out, shader.binary_len);
    push_u64(out, shader.hash);
    finish(out)
}

/// Decode a `GpuShaderRequest` and the source that follows it.
///
/// The source length is checked twice: once against what is actually present
/// and once against [`MAX_SHADER_SOURCE`]. A client that claims a gigabyte of
/// shader gets an error rather than an allocation.
pub fn decode_shader_request(bytes: &[u8]) -> Result<(GpuShaderRequest, &[u8]), DsError> {
    if bytes.len() < 16 {
        return Err(DsError::InvalidMessage);
    }
    let mut d = Decoder::new(bytes);
    let context = d.read_u32().ok_or(DsError::InvalidMessage)?;
    let flags = d.read_u32().ok_or(DsError::InvalidMessage)?;
    let source_len = d.read_u32().ok_or(DsError::InvalidMessage)?;
    let _pad = d.read_u32().ok_or(DsError::InvalidMessage)?;
    if source_len as usize > MAX_SHADER_SOURCE {
        return Err(DsError::InvalidMessage);
    }
    // Take exactly `source_len` bytes, and only if they are really there.
    let source = bytes.get(16..).ok_or(DsError::InvalidMessage)?;
    if source.len() < source_len as usize {
        return Err(DsError::InvalidMessage);
    }
    Ok((
        GpuShaderRequest {
            context,
            flags: GpuShaderFlags::from_bits_truncate(flags),
            source_len,
            _pad: 0,
        },
        &source[..source_len as usize],
    ))
}

/// Decode a `GpuContextInfo`.
pub fn decode_context_info(bytes: &[u8]) -> Result<GpuContextInfo, DsError> {
    if bytes.len() < 24 {
        return Err(DsError::InvalidMessage);
    }
    let mut d = Decoder::new(bytes);
    Ok(GpuContextInfo {
        handle: d.read_u32().ok_or(DsError::InvalidMessage)?,
        engines: d.read_u32().ok_or(DsError::InvalidMessage)?,
        ring_size: d.read_u64().ok_or(DsError::InvalidMessage)?,
        ring_tail_usable: d.read_u64().ok_or(DsError::InvalidMessage)?,
    })
}

/// Decode a `GpuBuffer`.
pub fn decode_buffer(bytes: &[u8]) -> Result<GpuBuffer, DsError> {
    if bytes.len() < 32 {
        return Err(DsError::InvalidMessage);
    }
    let mut d = Decoder::new(bytes);
    Ok(GpuBuffer {
        handle: d.read_u32().ok_or(DsError::InvalidMessage)?,
        _pad: d.read_u32().ok_or(DsError::InvalidMessage)?,
        phys: d.read_u64().ok_or(DsError::InvalidMessage)?,
        size: d.read_u64().ok_or(DsError::InvalidMessage)?,
        virt: d.read_u64().ok_or(DsError::InvalidMessage)?,
    })
}

/// Decode a `GpuShader`.
pub fn decode_shader(bytes: &[u8]) -> Result<GpuShader, DsError> {
    if bytes.len() < 16 {
        return Err(DsError::InvalidMessage);
    }
    let mut d = Decoder::new(bytes);
    Ok(GpuShader {
        handle: d.read_u32().ok_or(DsError::InvalidMessage)?,
        binary_len: d.read_u32().ok_or(DsError::InvalidMessage)?,
        hash: d.read_u64().ok_or(DsError::InvalidMessage)?,
    })
}

/// Length of the encoded form of one payload, without building it.
fn finish(out: &Vec<u8>) -> Result<usize, DsError> {
    if out.len() > MAX_PAYLOAD_LEN {
        return Err(DsError::InvalidMessage);
    }
    Ok(out.len())
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}
