//! Which execution engines a device has, and what each one is for.
//!
//! # The engine model changed three times
//!
//! AMD did not keep one compute-unit design and scale it. The changes are
//! structural, which is why this module exists separately from
//! [`crate::probe::Arch`]:
//!
//! * **GCN** (2011-2019) - SIMD32 inside a 64-wide wave, one compute
//!   dispatcher, and a separate UVD block for video.
//! * **Vega** - a compute unit gained its own fast memory, and the display
//!   engine moved onto the card proper rather than sitting beside it.
//! * **RDNA** (2019-) - wave32 instead of wave64, a redesigned compute
//!   pipeline, and a hardware queue that replaced the packet scheduler.
//!
//! The wave width is the part a driver has to get *right*, not merely fast: a
//! dispatch sized for 64 running on a 32-wide wave is a correctness bug on the
//! GPU, not a slow path. [`Arch::wave_size`] carries it.
//!
//! # Why the framework only sees a count
//!
//! `GpuInfo::engines` is a number. What an engine *is* is this driver's
//! business, because a client has nothing to do with it beyond "give me
//! something to draw on".

use crate::probe::Arch;

/// What one engine does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineKind {
    /// Draws triangles. What a renderer wants.
    Render,
    /// Blits and fills. Can stand in for render on a small part.
    Copy,
    /// Video encode and decode.
    Media,
    /// General compute without graphics.
    Compute,
}

/// One execution engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Engine {
    /// Its class, which selects a register block.
    pub kind: EngineKind,
    /// Where its registers are, as an offset into the MMIO aperture.
    pub mmio_offset: u32,
    /// How many execution units it has. For a render engine this is the EU
    /// count a client actually cares about.
    pub units: u16,
}

/// The engine array for a generation.
///
/// A fixed-size array rather than a `Vec`, because the number of engines is
/// known from the generation and a driver with an allocator is a driver with a
/// way to fail at the worst moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineList {
    engines: [Option<Engine>; 8],
    count: u8,
}

impl EngineList {
    /// The engines a device of this generation has.
    ///
    /// Counts are architectural, not per-part: a 4-part DG2 really does have
    /// more media engines than a 2-part one, but the driver reads the real
    /// counts from the `GT_*` registers at attach and narrows this.
    pub const fn for_arch(arch: Arch) -> Self {
        let mut list = Self { engines: [None; 8], count: 0 };

        match arch {
            // GCN: one compute array, the UVD video block, and a copy engine.
            Arch::Gcn1 | Arch::Gcn4 => {
                list.engines[0] = Some(Engine { kind: EngineKind::Compute, mmio_offset: 0x1A_0000, units: 8 });
                list.engines[1] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1B_8000, units: 1 });
                list.engines[2] = Some(Engine { kind: EngineKind::Copy, mmio_offset: 0x1A_2000, units: 1 });
                list.count = 3;
            }
            // GCN 5, "Polaris": same structure, a narrower array.
            Arch::Gcn5 => {
                list.engines[0] = Some(Engine { kind: EngineKind::Compute, mmio_offset: 0x1A_0000, units: 4 });
                list.engines[1] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1B_8000, units: 1 });
                list.engines[2] = Some(Engine { kind: EngineKind::Copy, mmio_offset: 0x1A_2000, units: 1 });
                list.count = 3;
            }
            // Vega: the display engine is on the card and the array grew. This
            // is the first part where discrete is a physical fact rather than
            // an APU sharing silicon with the CPU.
            Arch::Vega => {
                list.engines[0] = Some(Engine { kind: EngineKind::Compute, mmio_offset: 0x1A_0000, units: 16 });
                list.engines[1] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1B_8000, units: 1 });
                list.engines[2] = Some(Engine { kind: EngineKind::Copy, mmio_offset: 0x1A_2000, units: 1 });
                list.engines[3] = Some(Engine { kind: EngineKind::Copy, mmio_offset: 0x1A_3000, units: 1 });
                list.count = 4;
            }
            // RDNA 1: the wave halved and the video block became VCN.
            Arch::Rdna1 => {
                list.engines[0] = Some(Engine { kind: EngineKind::Compute, mmio_offset: 0x1A_0000, units: 20 });
                list.engines[1] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1B_8000, units: 1 });
                list.engines[2] = Some(Engine { kind: EngineKind::Copy, mmio_offset: 0x1A_2000, units: 1 });
                list.engines[3] = Some(Engine { kind: EngineKind::Copy, mmio_offset: 0x1A_3000, units: 1 });
                list.count = 4;
            }
            // RDNA 2: two compute partitions and a second media block.
            Arch::Rdna2 => {
                list.engines[0] = Some(Engine { kind: EngineKind::Compute, mmio_offset: 0x1A_0000, units: 20 });
                list.engines[1] = Some(Engine { kind: EngineKind::Compute, mmio_offset: 0x1A_2000, units: 20 });
                list.engines[2] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1B_8000, units: 1 });
                list.engines[3] = Some(Engine { kind: EngineKind::Copy, mmio_offset: 0x1A_3000, units: 1 });
                list.count = 4;
            }
            // RDNA 3 and 4: more compute, several media blocks, and an
            // infinity cache the older parts do not have.
            Arch::Rdna3 | Arch::Rdna4 => {
                list.engines[0] = Some(Engine { kind: EngineKind::Compute, mmio_offset: 0x1A_0000, units: 40 });
                list.engines[1] = Some(Engine { kind: EngineKind::Compute, mmio_offset: 0x1A_2000, units: 40 });
                list.engines[2] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1B_8000, units: 2 });
                list.engines[3] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1B_9000, units: 2 });
                list.engines[4] = Some(Engine { kind: EngineKind::Copy, mmio_offset: 0x1A_3000, units: 1 });
                list.count = 5;
            }
            // Legacy and unknown are refused by the caller.
            _ => {}
        }

        list
    }

    /// How many engines there are.
    pub const fn count(&self) -> usize {
        self.count as usize
    }

    /// One engine, if it exists.
    pub const fn get(&self, index: usize) -> Option<Engine> {
        if index >= self.count as usize {
            return None;
        }
        self.engines[index]
    }

    /// Iterate the engines in order.
    pub fn iter(&self) -> impl Iterator<Item = &Engine> {
        self.engines.iter().take(self.count as usize).filter_map(|e| e.as_ref())
    }

    /// How many render engines: what a renderer can spread across.
    pub fn render_count(&self) -> usize {
        self.iter().filter(|e| e.kind == EngineKind::Compute).count()
    }

    /// The engine mask `GpuInfo` reports. A count, not a map.
    pub const fn mask(&self) -> u32 {
        self.count as u32
    }
}
