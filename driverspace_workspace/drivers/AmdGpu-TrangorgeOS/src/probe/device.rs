//! Reading one AMD GPU out of the PCI bus.
//!
//! # What probe decides
//!
//! Given a bus, function, vendor and device id, probe answers three questions
//! and refuses a fourth:
//!
//! 1. Is this an AMD display device at all?
//! 2. Which [`Arch`] and which [`MemoryModel`] is it?
//! 3. Is it an APU or a discrete card?
//! 4. Can this driver drive it? A `no` is a refusal, not a warning.
//!
//! # Why probe takes ids, not a live BAR
//!
//! So the classification is pure and testable without hardware, and the rules
//! can be checked against every id in the table on a machine that has none of
//! them. Only after classification does anything map a register.
//!
//! # What is deliberately not decided here
//!
//! The Instinct MI-series accelerators share AMD's vendor id, and some of them
//! share a *device id* with a retail part as well: `0x66A1` is both a Vega 20
//! and an Instinct MI50, because they are the same die. `pci.ids` records that
//! in the subsystem row, not the device row, and a subsystem row is not
//! available to a device-class driver.
//!
//! So this layer cannot refuse an MI50 by id, and does not pretend to. What
//! it can do is classify the id, and the driver refuses an accelerator later,
//! when it has read the silicon and found there is no display engine. The
//! alternative - guessing from the marketing name - would either reject retail
//! cards or accept compute-only ones, and both are worse than a refusal made
//! with real information.

use crate::{
    mm::MemoryModel,
    probe::{arch::Arch, names::name_for},
};

/// AMD's PCI vendor id. ATI used the same one, so it covers both lineages.
pub const AMD_VENDOR_ID: u16 = 0x1002;

/// What probe found out about one device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmdDevice {
    pub vendor_id: u16,
    pub device_id: u16,
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    /// The architecture, which picks the register map.
    pub arch: Arch,
    /// How its memory is organised: the APU-versus-discrete distinction.
    pub memory: MemoryModel,
    /// A human-readable name from the PCI id table.
    pub name: &'static str,
}

impl AmdDevice {
    /// Whether this device is a discrete card with its own memory.
    pub const fn is_discrete(&self) -> bool {
        self.memory.has_local_memory()
    }

    /// `GpuCaps` bits implied by the hardware alone.
    pub const fn caps(&self) -> u64 {
        use kapi_abi::payloads::gpu::GpuCaps;

        let mut caps = GpuCaps::RING_SUBMIT.bits()
            | GpuCaps::PINNED_MEMORY.bits()
            | GpuCaps::HW_FENCE.bits()
            | GpuCaps::HW_RESET.bits()
            | GpuCaps::FLIP.bits();

        if self.memory.has_local_memory() {
            caps |= GpuCaps::DEDICATED_VRAM.bits();
        } else {
            caps |= GpuCaps::UNIFIED_MEMORY.bits();
        }
        caps
    }
}

/// Classify a device from its PCI ids.
pub fn probe(vendor_id: u16, device_id: u16, bus: u8, slot: u8, function: u8) -> Option<AmdDevice> {
    if vendor_id != AMD_VENDOR_ID {
        return None;
    }
    let name = name_for(device_id);
    // An id the table does not know is refused rather than guessed at.
    if name == GENERIC_NAME {
        return None;
    }
    let family = family_of(device_id)?;
    let arch = Arch::from_family(family);
    let memory = memory_model(arch, device_id)?;
    Some(AmdDevice {
        vendor_id,
        device_id,
        bus,
        slot,
        function,
        arch,
        memory,
        name,
    })
}

/// The name `name_for` returns for an id that is not in the table.
const GENERIC_NAME: &str = "AMD Radeon";

/// The family a device id belongs to.
const fn family_of(device_id: u16) -> Option<&'static str> {
    crate::probe::families::family_for(device_id)
}

/// Whether a device id is one this driver recognises.
pub const fn is_supported(vendor_id: u16, device_id: u16) -> bool {
    if vendor_id != AMD_VENDOR_ID {
        return false;
    }
    match family_of(device_id) {
        Some(family) => Arch::from_family(family).is_supported(),
        None => false,
    }
}

/// The APU families, by name.
///
/// An APU shares system memory with the CPU, so its memory model is
/// [`MemoryModel::Shared`] regardless of how new the silicon is. This list is
/// what makes that a decision rather than a guess - and the test that a 2023
/// APU is treated differently from a 2019 discrete card is the one that
/// matters.
pub const APU_FAMILIES: &[&str] = &[
    "Kaveri", "Kabini", "Mullins", "Topaz", "Carrizo", "Stoney", "Barts",
    "Aruba", "Lesotho", "Sienna", "Renoir", "Cezanne", "Lucienne", "Raphael",
    "Barcelo", "Dali", "Mendocino", "YellowCarp", "Sian", "Aquaria",
    "Phoenix", "Phoenix1", "Phoenix2", "Rembrandt", "Hawk", "HawkPoint1",
    "HawkPoint2", "Strix", "Krackan", "Krackan2", "VanGogh", "Gorgon",
    "Picasso/Raven", "Raven", "GraniteRidge",
];

/// The default aperture an AMD part is given.
///
/// A 64-bit IOVA is the architectural limit on the parts this driver drives,
/// and asking for more would mean mapping a page table nobody can fill.
pub const DEFAULT_APERTURE: u64 = 1 << 47;

/// The memory model for a known device id.
///
/// `None` for an id the family table did not recognise, so a caller cannot
/// accidentally treat an unknown card as an APU.
pub fn memory_model(arch: Arch, device_id: u16) -> Option<MemoryModel> {
    let family = family_of(device_id)?;
    if APU_FAMILIES.contains(&family) {
        return Some(MemoryModel::Shared { aperture: DEFAULT_APERTURE });
    }
    Some(match arch {
        // A discrete part's memory size is a property of the board, not the id:
        // one Navi 21 id covers a 6 GB card and a 20 GB one. The driver reads
        // the real size from `VRAM_SIZE` at attach; this is the floor.
        Arch::Vega => MemoryModel::HbmAndShared { vram_size: 2 << 30, aperture: DEFAULT_APERTURE },
        Arch::Gcn1 | Arch::Gcn4 | Arch::Gcn5 | Arch::Rdna1 | Arch::Rdna2
        | Arch::Rdna3 | Arch::Rdna4 => {
            MemoryModel::LocalAndShared { vram_size: 4 << 30, aperture: DEFAULT_APERTURE }
        }
        // Legacy and unknown are refused by the caller; the model here is only
        // so the probe can return a value rather than a partial one.
        _ => MemoryModel::Shared { aperture: DEFAULT_APERTURE },
    })
}
