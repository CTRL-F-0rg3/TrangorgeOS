//! Which execution engines a device has, and what each one is for.
//!
//! # The real generation boundary
//!
//! The engine layout changes at Gen12, and it changes *differently* for the
//! discrete parts than for the integrated ones. That asymmetry is the reason
//! this module exists separately from [`crate::probe::GenKind`], and it is the
//! closest thing in Intel's line to the split a discrete card usually implies:
//!
//! * **Gen11** (Ice Lake) - one render engine, one video engine.
//! * **Gen12** (Tiger Lake) - render split into two, plus video.
//! * **Gen12.5 integrated** (Alder Lake) - a small and a large render engine.
//! * **Gen12.5 discrete** (Arc DG2) - many media engines, no large/small
//!   asymmetry: a symmetric design aimed at throughput.
//! * **Gen13** (Battlemage) - more compute units, more media engines.
//!
//! An Arc card and an Alder Lake iGPU share a generation and diverge exactly
//! where one would expect a discrete card to: the engine array.
//!
//! # Why the framework only sees a count
//!
//! `GpuInfo::engines` is a number. What an engine *is* is this driver's
//! business, because a client has nothing to do with it beyond "give me
//! something to draw on".

use crate::probe::GenKind;

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
    pub const fn for_generation(generation: GenKind) -> Self {
        let mut list = Self { engines: [None; 8], count: 0 };

        match generation {
            GenKind::Gen11 => {
                list.engines[0] = Some(Engine { kind: EngineKind::Render, mmio_offset: 0x2_0000, units: 512 });
                list.engines[1] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1A_0000, units: 2 });
                list.count = 2;
            }
            // Render split in two, plus video.
            GenKind::Gen12 | GenKind::Gen12_7 => {
                list.engines[0] = Some(Engine { kind: EngineKind::Render, mmio_offset: 0x2_0000, units: 512 });
                list.engines[1] = Some(Engine { kind: EngineKind::Render, mmio_offset: 0x4_0000, units: 512 });
                list.engines[2] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1A_0000, units: 2 });
                list.count = 3;
            }
            // A small and a large render engine: the integrated asymmetry.
            GenKind::Gen12_5 => {
                list.engines[0] = Some(Engine { kind: EngineKind::Render, mmio_offset: 0x2_0000, units: 128 });
                list.engines[1] = Some(Engine { kind: EngineKind::Render, mmio_offset: 0x4_0000, units: 512 });
                list.engines[2] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1A_0000, units: 2 });
                list.count = 3;
            }
            // Arc DG2 and Battlemage: symmetric, and media-heavy. This is the
            // shape that actually differs from the iGPU of the same
            // generation, and it is why one driver is not a compromise.
            GenKind::Gen13 => {
                list.engines[0] = Some(Engine { kind: EngineKind::Render, mmio_offset: 0x2_0000, units: 1024 });
                list.engines[1] = Some(Engine { kind: EngineKind::Render, mmio_offset: 0x4_0000, units: 1024 });
                list.engines[2] = Some(Engine { kind: EngineKind::Render, mmio_offset: 0x6_0000, units: 1024 });
                list.engines[3] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1A_0000, units: 4 });
                list.engines[4] = Some(Engine { kind: EngineKind::Media, mmio_offset: 0x1C_0000, units: 4 });
                list.engines[5] = Some(Engine { kind: EngineKind::Copy, mmio_offset: 0x1D_0000, units: 1 });
                list.count = 6;
            }
            // Old enough that this driver refuses them.
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
        self.iter().filter(|e| e.kind == EngineKind::Render).count()
    }

    /// The engine mask `GpuInfo` reports. A count, not a map.
    pub const fn mask(&self) -> u32 {
        self.count as u32
    }
}
