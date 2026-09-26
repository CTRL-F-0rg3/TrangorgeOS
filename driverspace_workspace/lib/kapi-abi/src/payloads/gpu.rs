//! GPU / graphics IPC message payloads.
//!
//! These are the structures a GPU driver and its clients exchange: device
//! descriptors, mode sets, buffer and surface allocation, shader handles,
//! submissions and fences.
//!
//! The vocabulary comes from the graphics chain this OS already has
//! (`gfx_protocol_workspace`: `SurfaceId`, `BufferId`, `PixelFormat`, `GfxCmd`)
//! and from the Gallium model Mesa uses (`PIPE_CAP_*`, a context, a fence).
//! Keeping those names is what lets `Pipe-TrangorgeOS` be a port rather than a
//! redesign.
//!
//! ## What crosses the boundary and what does not
//!
//! Only *descriptors* cross: ids, sizes, flags and addresses. Register dumps,
//! command buffers and shader binaries stay on the driver side and are
//! referenced by handle. A GPU driver talks to hardware directly, so the more
//! it can do behind its own door, the less the system has to trust about it.
//!
//! All requests follow the convention used by the rest of `kapi-abi`:
//! `DsMsg::arg0` carries a payload pointer and `arg1` the length in bytes.

use bitflags::bitflags;

/// Which vendor's hardware a driver drives.
///
/// The two vendors are separate drivers by design, not by accident: their
/// register models, memory models and command encodings share nothing worth
/// abstracting, and a common layer that pretended otherwise would be a
/// lowest-common-denominator that fits neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum GpuVendor {
    Intel = 0,
    Amd = 1,
    /// The software `vgpu` used for headless and virtual setups.
    Virtual = 2,
    /// The default is `Unknown`, not `Intel`: a zeroed descriptor must never
    /// read as a real part, or a driver that forgot to fill a field would be
    /// taken for an Intel one.
    #[default]
    Unknown = 0xFFFF,
}

impl GpuVendor {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::Intel,
            1 => Self::Amd,
            2 => Self::Virtual,
            _ => Self::Unknown,
        }
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Intel => "intel",
            Self::Amd => "amd",
            Self::Virtual => "virtual",
            Self::Unknown => "unknown",
        }
    }
}

bitflags! {
    /// What a GPU can do, as a feature bitmask.
    ///
    /// The port of Gallium's `PIPE_CAP_*` reduced to the bits the OS itself
    /// has to act on. A client reads it to decide what to ask for; it is
    /// descriptive, never prescriptive.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    #[repr(C)]
    pub struct GpuCaps: u64 {
        const NONE = 0;
        /// Can present to a scanout engine without a copy.
        const DIRECT_SCANOUT = 1 << 0;
        /// Has a programmable command processor that takes ring buffers.
        const RING_SUBMIT = 1 << 1;
        /// Supports pinned, page-locked buffer memory.
        const PINNED_MEMORY = 1 << 2;
        /// Has its own video memory as well as system memory.
        const DEDICATED_VRAM = 1 << 3;
        /// Shares system memory with the graphics device.
        const UNIFIED_MEMORY = 1 << 4;
        /// Supports hardware semaphores / fences.
        const HW_FENCE = 1 << 5;
        /// Can be reset without a full device re-enumeration.
        const HW_RESET = 1 << 6;
        /// Requires the IOMMU to map its DMA memory.
        const REQUIRES_IOMMU = 1 << 7;
        /// Can present two surfaces at once (seamless flip).
        const FLIP = 1 << 8;
    }
}

/// How a pixel is laid out in memory.
///
/// The values are the ones already used by `gfx_protocol_workspace`, so a
/// surface created there and rendered here agree without a translation table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum GpuFormat {
    #[default]
    XRGB8888 = 0,
    ARGB8888 = 1,
    XRGB1555 = 2,
    RGB565 = 3,
    /// 4 bytes per pixel, hardware-native tiling left to the driver.
    Bgra8888 = 4,
    /// No format: a pure command or query target.
    None = 0xFFFF,
}

impl GpuFormat {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::ARGB8888,
            2 => Self::XRGB1555,
            3 => Self::RGB565,
            4 => Self::Bgra8888,
            0xFFFF => Self::None,
            _ => Self::XRGB8888,
        }
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::XRGB8888 => "XRGB8888",
            Self::ARGB8888 => "ARGB8888",
            Self::XRGB1555 => "XRGB1555",
            Self::RGB565 => "RGB565",
            Self::Bgra8888 => "BGRA8888",
            Self::None => "NONE",
        }
    }

    /// Bytes one pixel occupies; `0` for [`GpuFormat::None`].
    #[inline]
    pub const fn bytes_per_pixel(self) -> u32 {
        match self {
            Self::XRGB8888 | Self::ARGB8888 | Self::Bgra8888 => 4,
            Self::XRGB1555 | Self::RGB565 => 2,
            Self::None => 0,
        }
    }
}

/// Immutable description of one GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuInfo {
    /// Driver-private index, `0..gpu_count`.
    pub index: u32,
    pub vendor: GpuVendor,
    /// PCI vendor id.
    pub vendor_id: u16,
    /// PCI device id.
    pub device_id: u16,
    /// What the driver can do; see [`GpuCaps`].
    pub caps: u64,
    /// Size of the dedicated video memory, `0` for a unified-memory part.
    pub vram_size: u64,
    /// Size of the MMIO aperture the driver mapped.
    pub mmio_size: u64,
    /// The device's clock in MHz, as reported at probe time.
    pub clock_mhz: u32,
    /// Number of execution engines usable for rendering.
    pub engines: u32,
}

/// A display mode, the payload of `GfxModeSet` and `GfxModeGet`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuMode {
    pub width: u32,
    pub height: u32,
    pub format: GpuFormat,
    /// Bytes per scanline; `0` lets the driver pick.
    pub stride: u32,
    /// Refresh rate in mHz, so `60000` is 60 Hz.
    pub refresh_mhz: u32,
}

bitflags! {
    /// How a buffer will be used.
    ///
    /// The driver uses this to pick a memory class, and refuses an allocation
    /// it cannot honour rather than silently substituting a slower one.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    #[repr(C)]
    pub struct GpuBufferFlags: u32 {
        const NONE = 0;
        /// Usable as a render target.
        const RENDER_TARGET = 1 << 0;
        /// Usable as a texture.
        const TEXTURE = 1 << 1;
        /// Read by the command processor.
        const VERTEX = 1 << 2;
        /// Read by the rasterizer.
        const INDEX = 1 << 3;
        /// Read by the pixel shader.
        const UNIFORM = 1 << 4;
        /// Backed by device memory rather than system memory.
        const VRAM = 1 << 5;
        /// Mapped into the driver's address space once.
        const PINNED = 1 << 6;
        /// Shared between processes through a handle.
        const SHARED = 1 << 7;
    }
}

/// Request to allocate a GPU buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuBufferRequest {
    pub size: u64,
    pub flags: GpuBufferFlags,
    pub _pad: u32,
}

/// A buffer the driver has allocated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuBuffer {
    /// Handle the client uses for this buffer.
    pub handle: u32,
    pub _pad: u32,
    /// Physical address, `0` until the driver has placed it.
    pub phys: u64,
    pub size: u64,
    /// Virtual address in the driver's address space.
    pub virt: u64,
}

bitflags! {
    /// What a surface is for.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    #[repr(C)]
    pub struct GpuSurfaceFlags: u32 {
        const NONE = 0;
        /// Displayed when the compositor commits.
        const VISIBLE = 1 << 0;
        /// Covers the whole output.
        const FULLSCREEN = 1 << 1;
        /// Double-buffered, so the compositor never tears.
        const DOUBLE_BUFFERED = 1 << 2;
        /// Should be scanned out directly by the GPU.
        const DIRECT_SCANOUT = 1 << 3;
    }
}

/// A drawable surface owned by a client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuSurface {
    pub handle: u32,
    pub _pad: u32,
    /// Buffers the surface renders into, front then back.
    pub buffers: [u32; 2],
    pub width: u32,
    pub height: u32,
    pub format: GpuFormat,
    pub flags: GpuSurfaceFlags,
    /// Which buffer is being presented, 0 or 1.
    pub front: u32,
}

bitflags! {
    /// What kind of program a shader is.
    ///
    /// The bits deliberately start above [`GpuSurfaceFlags`]'s range. Both are
    /// `u32` bitflags in this ABI, so overlapping values would make a mixed-up
    /// field a silent permission change rather than a type error.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    #[repr(C)]
    pub struct GpuShaderFlags: u32 {
        const NONE = 0;
        const VERTEX = 1 << 8;
        const FRAGMENT = 1 << 9;
        const COMPUTE = 1 << 10;
        const GEOMETRY = 1 << 11;
    }
}

/// Request to create a context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuContextRequest {
    /// Driver-private device index.
    pub device: u32,
    /// Engine mask; `0` lets the driver choose.
    pub engines: u32,
    /// Shared memory base, for coherent upload buffers.
    pub shmem_base: u64,
    pub shmem_size: u64,
}

/// Reply to [`GpuContextRequest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuContextInfo {
    /// Handle the client uses for this context.
    pub handle: u32,
    /// Engine mask the driver actually assigned.
    pub engines: u32,
    /// Size of the driver's command ring, once it has sized it.
    pub ring_size: u64,
    /// How much of that ring the client may fill.
    pub ring_tail_usable: u64,
}

/// Request to compile a shader program.
///
/// The source follows this struct in the payload area; only its length
/// crosses here, because the binary the driver produces never leaves it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuShaderRequest {
    pub context: u32,
    pub flags: GpuShaderFlags,
    /// Byte length of the source that follows.
    pub source_len: u32,
    pub _pad: u32,
}

/// A compiled program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuShader {
    /// Handle the client passes back in a draw.
    pub handle: u32,
    /// Byte length of the driver's own binary, kept driver-side.
    pub binary_len: u32,
    /// A hash the client can use to skip recompiles.
    pub hash: u64,
}

/// One batch of work for the command processor.
///
/// A submission names a slice of the client's ring rather than carrying the
/// commands: they are already in memory the GPU can read, and copying them
/// through the IPC message would be both slower and a way for a client to lie
/// about what it is about to execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuSubmit {
    pub context: u32,
    /// Byte offset into the client's ring.
    pub offset: u32,
    /// Byte length of the command stream.
    pub length: u32,
    /// Ring this submission belongs to, for multi-engine work.
    pub ring: u32,
    /// Engine mask this batch targets.
    pub engines: u32,
    pub _pad: u32,
}

/// A fence the client waits on.
///
/// Fences are monotonic and per-context: comparing a new value against the
/// last one seen is enough to know a batch completed, which is what the
/// compositor needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuFence {
    /// Monotonically increasing per context; `0` means "no fence".
    pub value: u64,
    /// Context the fence belongs to.
    pub context: u32,
    /// Whether the GPU has signalled it.
    pub signalled: u32,
    pub _pad: u32,
}

/// Request to reset the GPU.
///
/// Reset is a blunt instrument and the driver refuses it while work is in
/// flight, because a half-submitted command stream that a reset discards is
/// how a display server ends up unrecoverable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct GpuResetRequest {
    pub device: u32,
    pub context: u32,
    /// `1` to allow a reset with work in flight.
    pub force: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendors_round_trip_and_have_names() {
        assert_eq!(GpuVendor::from_u32(0), GpuVendor::Intel);
        assert_eq!(GpuVendor::from_u32(1), GpuVendor::Amd);
        assert_eq!(GpuVendor::from_u32(99), GpuVendor::Unknown);
        assert_eq!(GpuVendor::Intel.as_str(), "intel");
        assert_eq!(GpuVendor::Amd.as_str(), "amd");
    }

    #[test]
    fn intel_and_amd_are_distinct_vendors() {
        // The two drivers are separate by design; the vocabulary must not
        // collapse them into one "gpu" value.
        assert_ne!(GpuVendor::Intel, GpuVendor::Amd);
        assert_eq!(GpuVendor::Intel as u32, 0);
        assert_eq!(GpuVendor::Amd as u32, 1);
    }

    #[test]
    fn format_widths_match_the_protocol_workspace() {
        assert_eq!(GpuFormat::XRGB8888.bytes_per_pixel(), 4);
        assert_eq!(GpuFormat::ARGB8888.bytes_per_pixel(), 4);
        assert_eq!(GpuFormat::RGB565.bytes_per_pixel(), 2);
        assert_eq!(GpuFormat::XRGB1555.bytes_per_pixel(), 2);
        assert_eq!(GpuFormat::None.bytes_per_pixel(), 0);
        assert_eq!(GpuFormat::from_u32(0), GpuFormat::XRGB8888);
        assert_eq!(GpuFormat::from_u32(0xFFFF), GpuFormat::None);
    }

    #[test]
    fn format_names_are_the_protocol_workspace_spelling() {
        assert_eq!(GpuFormat::XRGB8888.as_str(), "XRGB8888");
        assert_eq!(GpuFormat::RGB565.as_str(), "RGB565");
    }

    #[test]
    fn caps_compose_without_overlap() {
        let caps = GpuCaps::DIRECT_SCANOUT | GpuCaps::RING_SUBMIT;
        assert!(caps.contains(GpuCaps::DIRECT_SCANOUT));
        assert!(caps.contains(GpuCaps::RING_SUBMIT));
        assert!(!caps.contains(GpuCaps::HW_RESET));
        assert!(GpuCaps::NONE.is_empty());
    }

    #[test]
    fn buffer_flags_describe_the_standard_pipeline_stages() {
        let flags = GpuBufferFlags::VERTEX
            | GpuBufferFlags::INDEX
            | GpuBufferFlags::RENDER_TARGET;
        assert!(flags.contains(GpuBufferFlags::VERTEX));
        assert!(flags.contains(GpuBufferFlags::RENDER_TARGET));
        assert!(!flags.contains(GpuBufferFlags::VRAM));
    }

    #[test]
    fn surface_and_shader_flags_are_disjoint() {
        // Both are `u32` bitflags in this ABI, so a mix-up would be a silent
        // permission change rather than a type error. The whole surface range
        // must stay clear of the shader range.
        let surface_bits = GpuSurfaceFlags::VISIBLE.bits()
            | GpuSurfaceFlags::FULLSCREEN.bits()
            | GpuSurfaceFlags::DOUBLE_BUFFERED.bits()
            | GpuSurfaceFlags::DIRECT_SCANOUT.bits();
        let shader_bits = GpuShaderFlags::VERTEX.bits()
            | GpuShaderFlags::FRAGMENT.bits()
            | GpuShaderFlags::COMPUTE.bits()
            | GpuShaderFlags::GEOMETRY.bits();
        assert_eq!(surface_bits & shader_bits, 0, "surface and shader bits overlap");
        assert_ne!(GpuSurfaceFlags::VISIBLE.bits(), GpuShaderFlags::VERTEX.bits());
    }

    #[test]
    fn mode_refresh_is_expressed_in_millihertz() {
        let mode = GpuMode {
            width: 1920,
            height: 1080,
            format: GpuFormat::XRGB8888,
            stride: 1920 * 4,
            refresh_mhz: 60_000,
        };
        assert_eq!(mode.refresh_mhz / 1000, 60, "60 Hz expressed in mHz");
    }
}
