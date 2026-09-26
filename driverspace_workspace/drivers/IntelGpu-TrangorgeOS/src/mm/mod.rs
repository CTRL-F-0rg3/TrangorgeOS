//! Where a buffer lives, and how the GPU is told about it.
//!
//! # This is the integrated-versus-discrete split
//!
//! Intel's discrete and integrated parts share a register map, so the driver
//! is one. What genuinely differs is the *memory model*:
//!
//! * An integrated part has no memory of its own. Every buffer is system
//!   memory, and the GPU reaches it through the GTT - a page table the
//!   driver builds, because the GPU's aperture is far smaller than the
//!   address space it must address.
//! * A discrete part has local memory *and* can reach system memory. A
//!   texture the GPU samples often wants to be in local memory, while a
//!   buffer shared with the CPU has to stay in system memory.
//!
//! Getting this wrong is not a slowdown, it is a fault: an integrated part
//! asked to use local memory has none, and a discrete part asked to place a
//! render target in system memory will do it, slowly, over PCIe.
//!
//! # Why the driver decides, not the client
//!
//! A client says what a buffer is *for* (`GpuBufferFlags::RENDER_TARGET`,
//! `TEXTURE`, `VRAM`). It does not say where it goes. A client that guessed
//! wrong gets a buffer in the wrong memory class and finds out as a frame-rate
//! problem; a driver that guesses wrong produces a fault. So the placement
//! rule lives here and the client cannot override it.

use core::fmt;

/// How a device's memory is organised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryModel {
    /// No local memory. Everything is system memory, reached through the GTT.
    ///
    /// This is every integrated part: Broadwell through Arrow Lake.
    Shared {
        /// Bytes of system memory the device may address through its GTT.
        ///
        /// Smaller than physical RAM on purpose - the GTT is what makes the
        /// GPU's aperture cover more than it physically has.
        gtt_size: u64,
    },

    /// Local memory on the card, plus system memory for sharing.
    ///
    /// This is Arc: DG1, DG2, and Battlemage.
    LocalAndShared {
        /// Bytes of local (on-board) memory.
        vram_size: u64,
        /// Bytes of system memory the device may address through its GTT.
        gtt_size: u64,
    },
}

impl MemoryModel {
    /// Whether the device has memory of its own.
    pub const fn has_local_memory(self) -> bool {
        matches!(self, Self::LocalAndShared { .. })
    }

    /// Bytes of on-board memory; `0` for an integrated part.
    pub const fn vram_size(self) -> u64 {
        match self {
            Self::Shared { .. } => 0,
            Self::LocalAndShared { vram_size, .. } => vram_size,
        }
    }

    /// Bytes of system memory the device may address.
    pub const fn gtt_size(self) -> u64 {
        match self {
            Self::Shared { gtt_size } => gtt_size,
            Self::LocalAndShared { gtt_size, .. } => gtt_size,
        }
    }

    /// Where a buffer with these flags belongs.
    ///
    /// `GpuBufferFlags::VRAM` is a request, not a command: a part with no
    /// local memory cannot satisfy it, and pretending otherwise would hand
    /// the client an address in an aperture that does not exist.
    pub const fn placement(self, flags: u32) -> Placement {
        use kapi_abi::payloads::gpu::GpuBufferFlags;

        let wants_vram = flags & GpuBufferFlags::VRAM.bits() != 0;
        if wants_vram {
            return if self.has_local_memory() {
                Placement::Local
            } else {
                // An integrated part cannot honour a VRAM request. Reporting
                // this rather than silently substituting shared memory is the
                // whole point: the client asked for a class that does not
                // exist on this device, and that is worth knowing.
                Placement::Impossible
            };
        }

        // A render target on a discrete card is a local-memory placement
        // unless it is explicitly shared; the display engine and the CPU both
        // read shared buffers, so `SHARED` must be honoured.
        let shared = flags & GpuBufferFlags::SHARED.bits() != 0;
        if !shared && self.has_local_memory() && flags & GpuBufferFlags::RENDER_TARGET.bits() != 0 {
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
    /// System memory, reached through the GTT. The only place a CPU can see.
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
