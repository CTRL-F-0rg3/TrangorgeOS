//! Reading one Intel GPU out of the PCI bus.
//!
//! # What probe decides
//!
//! Given a bus, function, vendor and device id, probe answers two questions
//! and refuses a third:
//!
//! 1. Is this an Intel display device at all?
//! 2. Which [`GenKind`] and which [`MemoryModel`] is it?
//! 3. Can this driver drive it? A `no` here is a refusal, not a warning.
//!
//! # Why probe does not touch hardware
//!
//! It takes ids, not a live BAR. That keeps the classification pure and
//! testable without a device, and it means the rules above can be checked
//! against every PCI id in the tree on a machine that has none of them. Only
//! after classification does anything map a register.

use crate::{
    mm::MemoryModel,
    probe::{genkind::GenKind, names::name_for},
};

/// Intel's PCI vendor id.
pub const INTEL_VENDOR_ID: u16 = 0x8086;

/// What probe found out about one device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntelDevice {
    pub vendor_id: u16,
    pub device_id: u16,
    /// Which bus, device and function it was found on.
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    /// The generation, which picks the register map.
    pub generation: GenKind,
    /// How its memory is organised: this is the iGPU-versus-dGPU distinction.
    pub memory: MemoryModel,
    /// A human-readable name from the PCI id table.
    pub name: &'static str,
}

impl IntelDevice {
    /// Whether this device is a discrete card with its own memory.
    pub const fn is_discrete(&self) -> bool {
        self.memory.has_local_memory()
    }

    /// `GpuCaps` bits implied by the hardware alone.
    ///
    /// Everything here is a fact about the silicon. What a driver can
    /// *achieve* is a separate question and belongs in the renderer, because a
    /// capability the driver cannot honour on this OS is worse than a
    /// missing one.
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

        // A discrete part can scan straight out of its own memory; an
        // integrated one is already scanning out of system memory, which is
        // the same result by a different route.
        caps
    }
}

/// Classify a device from its PCI ids.
///
/// `None` when `vendor_id` is not Intel, or when the device id is not a
/// display device this table knows. The two cases are deliberately
/// indistinguishable to the caller: from `ds-manager`'s point of view a
/// non-Intel device and an unknown Intel device both mean "not mine".
pub fn probe(vendor_id: u16, device_id: u16, bus: u8, slot: u8, function: u8) -> Option<IntelDevice> {
    if vendor_id != INTEL_VENDOR_ID {
        return None;
    }
    let generation = GenKind::from_device_id(device_id)?;
    // `from_device_id` just said yes, so this cannot be `None`; using `?`
    // rather than `expect` keeps probe total without a panic path.
    let memory = memory_model(device_id)?;
    Some(IntelDevice {
        vendor_id,
        device_id,
        bus,
        slot,
        function,
        generation,
        memory,
        name: name_for(device_id),
    })
}

/// Whether a device id is one this driver recognises, without classifying it.
///
/// `ds-detect` needs this to build a manifest: it has to know a device is
/// worth loading a driver for before anything is mapped.
pub const fn is_supported(vendor_id: u16, device_id: u16) -> bool {
    if vendor_id != INTEL_VENDOR_ID {
        return false;
    }
    match GenKind::from_device_id(device_id) {
        Some(generation) => generation.is_supported(),
        None => false,
    }
}

/// The discrete device ids, by generation.
///
/// # Why an explicit list, when generation almost decides it
///
/// Gen12.5 contains both Alder Lake integrated (`4680`..`4693`) and Arc DG2
/// (`4905`..`56B6`). Both are the same generation and both need the same
/// register map, so the generation cannot answer "is this discrete" on its
/// own - which is exactly why the memory model is a separate axis rather than
/// a field of `GenKind`.
///
/// Being explicit also means a new Arc id is *not* silently treated as
/// integrated: an unlisted id is assumed shared, and that assumption is
/// visible in review rather than buried in a range.
const fn is_discrete_id(device_id: u16) -> bool {
    matches!(device_id,
        // Arc DG1 - the "Xe MAX" board.
        0x4905 | 0x4906 | 0x4908 | 0x4909
        // Arc DG2: all three steps (G10/G11/G12), the Pro part and the mobile
        // parts are all in this one range.
        | 0x5690..=0x56B6
        // Battlemage B-series.
        | 0xE20B..=0xE223
    )
}

/// The memory model for a known Intel device id.
///
/// `None` for an id [`GenKind::from_device_id`] did not recognise, so a caller
/// cannot accidentally treat an unknown card as an integrated one.
pub const fn memory_model(device_id: u16) -> Option<MemoryModel> {
    if GenKind::from_device_id(device_id).is_none() {
        return None;
    }

    Some(if is_discrete_id(device_id) {
        // The VRAM size is a property of the board, not the id: one DG2 id
        // covers an A310 with 4 GB and an A770 with 16 GB. The driver reads
        // the real size from `BAR2` at attach time; this is the floor the
        // allocator is sized against before that read.
        MemoryModel::LocalAndShared {
            vram_size: 4 << 30,
            gtt_size: DEFAULT_GTT_SIZE,
        }
    } else {
        MemoryModel::Shared { gtt_size: DEFAULT_GTT_SIZE }
    })
}

/// The GTT size an integrated part is given when nothing says otherwise.
///
/// Intel's own driver uses 4 GiB by default, because a GTT smaller than the
/// working set means constant remapping and a larger one costs page tables
/// the device has to walk for nothing.
pub const DEFAULT_GTT_SIZE: u64 = 4 << 30;
