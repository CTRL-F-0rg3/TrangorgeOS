//! Where a buffer lives, and how the GPU is told about it.
//!
//! # This is the integrated-versus-discrete split, and it is real here
//!
//! Unlike Intel - where a discrete Arc card and an integrated Iris Xe share a
//! register map - AMD's split is genuine and appears in three places:
//!
//! * an APU has **no memory of its own**; the GPU reaches system memory
//!   directly and the CPU and GPU share the same pages with no copy;
//! * a discrete card has **local memory** plus the ability to reach system
//!   memory, and the choice matters more than it does at Intel, because the
//!   local memory sits behind a much wider interconnect;
//! * the **display engine** differs: an APU scans out of system memory, a
//!   discrete card scans out of a dedicated framebuffer or of a VRAM surface.
//!
//! # Why the driver decides, not the client
//!
//! A client says what a buffer is *for*. It does not say where it goes. A
//! client that guessed wrong gets a buffer in the wrong memory class and finds
//! out as a frame-rate problem; a driver that guessed wrong produces a fault or
//! a display that reads garbage.

use core::fmt;

/// How a device's memory is organised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryModel {
    /// An APU: no local memory, one shared address space.
    ///
    /// Not merely "system memory" as on Intel. The GPU and the CPU see the
    /// *same* pages, which is why an APU can be faster than a discrete card
    /// on a workload that would otherwise round-trip through VRAM.
    Shared {
        /// Bytes of system memory the graphics block may address.
        ///
        /// AMD's aperture is a 48-bit IOVA on the 64-bit parts and narrower
        /// below that; the driver sizes this from the register it reads at
        /// attach.
        aperture: u64,
    },

    /// A discrete card: local memory on the board, plus system memory.
    ///
    /// The two are *not* interchangeable. A texture the GPU samples wants to
    /// be local, while anything the CPU or the display engine reads has to be
    /// in system memory - which on AMD is a PCIe or Infinity Fabric
    /// round-trip.
    LocalAndShared {
        /// Bytes of local (on-board) memory.
        vram_size: u64,
        /// Bytes of system memory the card may address.
        aperture: u64,
    },

    /// Vega on HBM, where local memory is not ordinary GDDR.
    ///
    /// Separate because the access path differs enough that a driver treating
    /// it as generic local memory will place buffers it should not.
    HbmAndShared {
        vram_size: u64,
        aperture: u64,
    },
}

impl MemoryModel {
    /// Whether the device has memory of its own.
    pub const fn has_local_memory(self) -> bool {
        matches!(self, Self::LocalAndShared { .. } | Self::HbmAndShared { .. })
    }

    /// Bytes of on-board memory; `0` for an APU.
    pub const fn vram_size(self) -> u64 {
        match self {
            Self::Shared { .. } => 0,
            Self::LocalAndShared { vram_size, .. } | Self::HbmAndShared { vram_size, .. } => vram_size,
        }
    }

    /// Bytes of system memory the device may address.
    pub const fn aperture(self) -> u64 {
        match self {
            Self::Shared { aperture }
            | Self::LocalAndShared { aperture, .. }
            | Self::HbmAndShared { aperture, .. } => aperture,
        }
    }

    /// Where a buffer with these flags belongs.
    ///
    /// `GpuBufferFlags::VRAM` is a request, not a command: a part with no local
    /// memory cannot satisfy it, and pretending otherwise would hand the
    /// client an address in an aperture that does not exist.
    pub const fn placement(self, flags: u32) -> Placement {
        use kapi_abi::payloads::gpu::GpuBufferFlags;

        if flags & GpuBufferFlags::VRAM.bits() != 0 {
            return if self.has_local_memory() {
                Placement::Local
            } else {
                // An APU has no such memory. Saying so is the point: silently
                // substituting shared memory would work, be correct, and be
                // indistinguishable from a driver that honoured the request.
                Placement::Impossible
            };
        }

        let shared = flags & GpuBufferFlags::SHARED.bits() != 0;
        let render_target = flags & GpuBufferFlags::RENDER_TARGET.bits() != 0;

        if !shared && render_target && self.has_local_memory() {
            Placement::Local
        } else {
            Placement::Shared
        }
    }
}

/// Where one buffer belongs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Local memory on the card. Fast for the GPU, invisible to the CPU.
    Local,
    /// System memory. The only place a CPU, or an APU's display engine, can read.
    Shared,
    /// The request cannot be satisfied on this device.
    Impossible,
}

impl fmt::Display for Placement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Local => "local",
            Self::Shared => "shared",
            Self::Impossible => "unavailable",
        })
    }
}
