//! Video / Video-TrangorgeOS IPC message payloads.
//!
//! The vocabulary is deliberately vendor-neutral. QuickSync, NVDEC and VCN all
//! describe themselves in terms of "codecs this engine can decode, up to this
//! many surfaces, no wider than this", and that intersection is the only thing
//! the framework and its clients can rely on. Anything vendor-specific belongs
//! in a capability bit the caller may ignore, not in the descriptor.
//!
//! Every struct is `#[repr(C)]`, because these are interpreted by Rust, C and
//! Odin alike in shared memory. Requests follow the `DsMsg::arg0` = pointer,
//! `arg1` = length convention the rest of the system uses.
//!
//! # Nothing here names a vendor
//!
//! That is the point of the file, and it is worth stating because the obvious
//! alternative - one payload per vendor - is what makes a video stack
//! unmaintainable. A capture server and an NPU want the same thing: frames, in
//! a known layout, with a timestamp. Which silicon produced them is a
//! diagnostic, not part of the interface.

use bitflags::bitflags;

/// A compressed video codec, named the way every API names them.
///
/// FourCC, not an enum index: the numeric values are not ours to assign, and a
/// client that sends `VP9` needs the same four bytes the hardware does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Fourcc(pub u32);

impl Fourcc {
    /// `H.264 / AVC`.
    pub const H264: Self = Self::from_bytes(*b"avc1");
    /// `HEVC`.
    pub const HEVC: Self = Self::from_bytes(*b"hvc1");
    /// `VP9`.
    pub const VP9: Self = Self::from_bytes(*b"VP90");
    /// `AV1`.
    pub const AV1: Self = Self::from_bytes(*b"av01");
    /// `MPEG-2`.
    pub const MPEG2: Self = Self::from_bytes(*b"mpg2");
    /// `Motion JPEG`.
    pub const MJPEG: Self = Self::from_bytes(*b"MJpg");
    /// `VC-1`.
    pub const VC1: Self = Self::from_bytes(*b"VC-1");

    /// Build a FourCC from bytes in the usual order (`'a','v','c','1'`).
    #[inline]
    pub const fn from_bytes(b: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(b))
    }

    /// The four bytes, in the usual order.
    #[inline]
    pub const fn to_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    /// A short name for logs, if this is a codec we know.
    ///
    /// Falls back to the four characters themselves rather than `"unknown"`,
    /// because an unrecognised four-character code is far more useful in a log
    /// line than a word that says nothing.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::H264 => "H.264",
            Self::HEVC => "HEVC",
            Self::VP9 => "VP9",
            Self::AV1 => "AV1",
            Self::MPEG2 => "MPEG-2",
            Self::MJPEG => "MJPEG",
            Self::VC1 => "VC-1",
            _ => "????",
        }
    }

    /// Is this a codec the framework has a name for?
    ///
    /// Not `const`: comparing strings in a const context is not stable, and
    /// this is called from descriptors, not from a `const` item.
    #[inline]
    pub fn is_known(self) -> bool {
        self.as_str() != "????"
    }
}

/// A decoded surface layout.
///
/// Modelled on `DRM_FORMAT_MODIFIER`-free plain fourcc surfaces: the engine
/// hands back linear buffers with a stride, and a consumer either uses that or
/// asks for a conversion. A tiled or compressed surface would need a modifier
/// here, and none is defined because no backend produces one yet - see the
/// driver README rather than adding a field nothing sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct FrameFormat {
    /// FourCC of the decoded pixel format, e.g. `NV12`.
    pub format: Fourcc,
    /// Bytes per row. Never assumed from the width: a frame the hardware
    /// padded is a frame the consumer will read out of bounds if it assumes.
    pub stride: u32,
    /// Visible width in pixels.
    pub width: u32,
    /// Visible height in pixels.
    pub height: u32,
}

impl FrameFormat {
    /// Build a format descriptor.
    #[inline]
    pub const fn new(format: Fourcc, width: u32, height: u32, stride: u32) -> Self {
        Self { format, stride, width, height }
    }

    /// Bytes the visible plane occupies, which is `stride * height` and not
    /// `width * height`.
    #[inline]
    pub const fn plane_bytes(&self) -> u64 {
        (self.stride as u64) * (self.height as u64)
    }

    /// `NV12`: 4:2:0 with a full-resolution luma plane and a half-resolution
    /// interleaved chroma plane. What almost every hardware decoder outputs and
    /// what almost every NPU wants, which is why it is spelled out.
    #[inline]
    pub const fn nv12(width: u32, height: u32) -> Self {
        // Luma row is width bytes; chroma is half-width interleaved, which is
        // already the same byte count per row for 4:2:0 8-bit. Stride is rounded
        // up to 64 bytes, the alignment every engine here requires.
        let stride = ((width + 63) / 64) * 64;
        Self { format: Fourcc::from_bytes(*b"NV12"), stride, width, height }
    }

    /// Total bytes a surface of this format occupies, chroma included.
    #[inline]
    pub const fn total_bytes(&self) -> u64 {
        match self.format.to_bytes() {
            // 4:2:0: luma plus half as much again.
            [b'N', b'V', b'1', b'2'] => self.plane_bytes() + self.plane_bytes() / 2,
            // Packed 32-bit: one word per pixel.
            [b'R', b'G', b'B', b'X'] => self.plane_bytes() * 4,
            _ => self.plane_bytes(),
        }
    }
}

// A bitmask, not a set of queries, so a client can test support before it
// builds a pipeline - and so an engine that grows a new capability does not
// change the shape of the descriptor.
//
// Note: no doc comment on the invocation. rustdoc does not generate
// documentation for a macro invocation, and attaching one is a warning.
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[repr(transparent)]
    pub struct EngineCaps: u32 {
        /// Can take compressed frames in and produce decoded surfaces.
        const DECODE = 1 << 0;
        /// Can take decoded surfaces in and produce compressed frames.
        const ENCODE = 1 << 1;
        /// Can transcode: decode and encode in one session.
        const TRANSCODE = 1 << 2;
        /// Decoded surfaces are usable as-is, with no copy to system memory.
        ///
        /// This is the bit that decides whether a graphics server or an NPU can
        /// consume a frame for free. When it is clear, every frame crosses the
        /// bus on its way to the consumer, and a pipeline that assumed
        /// otherwise is silently three orders of magnitude slower.
        const ZERO_COPY = 1 << 3;
        /// Reports picture timing (colour description, interlacing).
        const HDR = 1 << 4;
        /// Can scale on the way out.
        const SCALE = 1 << 5;
    }
}

/// The descriptor of one video engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct EngineInfo {
    /// Index within the driver's engine list, and the handle for `VideoSubmit`.
    pub index: u32,
    /// What the engine can do.
    pub caps: u32,
    /// Highest resolution the engine accepts, as width and height.
    pub max_width: u32,
    /// See `max_width`.
    pub max_height: u32,
    /// How many frames may be in flight at once.
    ///
    /// A hard limit, not advice: exceeding it is what turns a decode into a
    /// stall, and a client that cannot see the limit will find out by hanging.
    pub max_surfaces: u32,
    /// Bytes of device memory the engine holds. Zero for a software engine.
    pub dedicated_memory: u64,
    /// PCI vendor, or `0` for a software engine. A diagnostic, never a
    /// dispatch key - see the module documentation.
    pub vendor_id: u16,
    /// PCI device.
    pub device_id: u16,
}

impl EngineInfo {
    /// The capability set, decoded.
    #[inline]
    pub fn cap_set(&self) -> EngineCaps {
        EngineCaps::from_bits_truncate(self.caps)
    }

    /// Can this engine decode?
    #[inline]
    pub const fn can_decode(&self) -> bool {
        self.caps & (EngineCaps::DECODE.bits() | EngineCaps::TRANSCODE.bits()) != 0
    }

    /// Does it hand out surfaces a consumer can read without a copy?
    #[inline]
    pub const fn is_zero_copy(&self) -> bool {
        self.caps & EngineCaps::ZERO_COPY.bits() != 0
    }
}

/// Lifecycle of one decoding or encoding session.
///
/// The same shape as ALSA's `snd_pcm_state`, and for the same reason: a caller
/// has to be able to ask "what is this doing" and get an answer that is not a
/// guess. The transitions are the same too, and a session that has been
/// `Reset` is always safe to reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SessionState {
    /// Created, nothing submitted yet.
    Idle = 0,
    /// Frames are in flight; a ticket may become ready at any moment.
    Running = 1,
    /// A frame is ready and a consumer is holding it.
    Draining = 2,
    /// The engine reported an unrecoverable error and the session needs a
    /// `Reset`.
    Error = 3,
}

impl SessionState {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Running,
            2 => Self::Draining,
            3 => Self::Error,
            _ => Self::Idle,
        }
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Draining => "draining",
            Self::Error => "error",
        }
    }
}

/// A request to open a session on an engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct SessionRequest {
    /// Engine index, from `EngineInfo::index`.
    pub engine: u32,
    /// Codec of the incoming bitstream.
    pub codec: u32,
    /// Pixel format the caller wants out.
    pub out_format: u32,
    /// Frames the caller intends to keep in flight. Clamped by the engine's
    /// `max_surfaces` rather than rejected, because a caller asking for more
    /// than the hardware has is asking for a lower bound, not an error.
    pub surfaces: u32,
    /// `0` to decode, non-zero to encode.
    pub encode: u32,
    /// Non-zero to receive every decoded frame, even one a consumer has not
    /// asked for. Off by default: an unsolicited frame is a frame somebody has
    /// to free.
    pub low_latency: u32,
}

/// A decoded or encoded frame, as handed to a consumer.
///
/// `buffer` is a *handle*, not a pointer. The memory belongs to the driver and
/// its address is not stable across a resize, so putting a pointer on the wire
/// would create a use-after-free that only shows up under load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct FrameInfo {
    /// `0` means "not ready"; every other field is then meaningless.
    ///
    /// A presence flag rather than an optional payload, so a client can poll
    /// with a fixed-size reply buffer and no allocation.
    pub present: u32,
    /// Driver handle for the surface, to be passed to `VideoRelease`.
    pub buffer: u32,
    /// The ticket `VideoSubmit` returned, echoed back.
    pub ticket: u32,
    /// The decoded layout.
    pub format: FrameFormat,
    /// Presentation timestamp, in nanoseconds on the engine's own clock.
    pub pts_ns: u64,
    /// Decode completion timestamp, same clock.
    pub dts_ns: u64,
    /// Whether this frame had to be dropped to keep up.
    ///
    /// Set on a frame that is delivered late rather than dropped, so a consumer
    /// can tell "you are seeing this frame late" from "you are not seeing it".
    pub late: u32,
}

impl FrameInfo {
    /// The "not ready" reply, which is a success with nothing in it.
    #[inline]
    pub const fn absent() -> Self {
        Self { present: 0, ..Self::ZERO }
    }

    const ZERO: Self = Self {
        present: 0,
        buffer: 0,
        ticket: 0,
        format: FrameFormat { format: Fourcc(0), stride: 0, width: 0, height: 0 },
        pts_ns: 0,
        dts_ns: 0,
        late: 0,
    };

    /// Is a frame actually here?
    #[inline]
    pub const fn is_present(&self) -> bool {
        self.present != 0
    }

    /// How late the frame was, in nanoseconds. Zero when it was on time.
    #[inline]
    pub const fn lateness_ns(&self) -> u64 {
        self.pts_ns.saturating_sub(self.dts_ns)
    }
}

/// A request to submit one compressed frame.
///
/// The bytes are not in this struct. They are in shared memory, and this
/// describes the shared memory - which is what lets a client hand the driver a
/// pointer into a DMA-mapped ring instead of a copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct SubmitRequest {
    /// Session handle, or `0` to use the session the service has open.
    pub session: u32,
    /// Byte length of the compressed frame.
    pub size: u32,
    /// Physical address of the compressed bytes.
    pub src_phys: u64,
    /// Virtual address the driver should read them through.
    pub src_virt: u64,
    /// Presentation timestamp, nanoseconds.
    pub pts_ns: u64,
    /// `1` when this is a keyframe, `0` when it depends on the previous frame.
    ///
    /// The engine needs this to bound its reference set. Getting it wrong is
    /// not a picture glitch, it is a decode failure, so it is explicit rather
    /// than inferred.
    pub keyframe: u32,
}

/// What a consumer wants from the driver, when it registers as a sink.
///
/// This is the whole of the video-to-graphics-or-NPU interface, and it is
/// deliberately three fields. Anything richer would force one consumer's needs
/// onto every other consumer, which is how a video stack ends up unable to
/// serve anybody.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct SinkRequest {
    /// What this consumer will do with the frames. A diagnostic for the
    /// driver's own logging, and nothing more.
    pub purpose: u32,
    /// `1` to receive frames without asking. `0` for pull semantics, where the
    /// consumer calls `VideoAcquire`.
    pub push: u32,
    /// How many frames the consumer can hold at once.
    ///
    /// The driver uses this as backpressure. A push consumer that says `1` and
    /// then takes 40 ms to finish a frame will be told to drop, not allowed to
    /// stall the decode - a slow consumer must never be able to stop the
    /// decoder, because the decoder is shared with every other session.
    pub depth: u32,
}

/// Values for `SinkRequest::purpose`.
///
/// Numbers, not a Rust enum, because the sink may be registered by something
/// that is not this framework - a graphics server speaking the wire protocol
/// directly.
pub mod sink_purpose {
    /// The frames are going to be displayed.
    pub const COMPOSITOR: u32 = 1;
    /// The frames are going to a neural accelerator for analysis.
    pub const ANALYTICS: u32 = 2;
    /// The frames are being written to storage.
    pub const CAPTURE: u32 = 3;
    /// The frames are being re-encoded.
    pub const TRANSCODE: u32 = 4;
}

impl SinkRequest {
    /// A pull-mode capture consumer, which is the safe default: nothing is
    /// pushed to a consumer that has not asked for frames.
    #[inline]
    pub const fn pull() -> Self {
        Self { purpose: sink_purpose::CAPTURE, push: 0, depth: 2 }
    }

    /// A push-mode compositor consumer.
    #[inline]
    pub const fn compositor() -> Self {
        Self { purpose: sink_purpose::COMPOSITOR, push: 1, depth: 3 }
    }

    /// Is this consumer push-mode?
    #[inline]
    pub const fn is_push(&self) -> bool {
        self.push != 0
    }

    /// The queue depth, forced to at least one.
    ///
    /// A depth of zero would mean "hold no frames", which for a push consumer
    /// means "drop everything" - almost certainly a caller mistake rather than
    /// an intent, so it is clamped and the frame is still delivered.
    #[inline]
    pub const fn effective_depth(&self) -> u32 {
        if self.depth == 0 {
            1
        } else {
            self.depth
        }
    }
}
