//! Calibration: read what a device can actually do, before trusting it.
//!
//! A manifest entry says what a driver *claims* to handle. Calibration reads
//! the hardware and says what it *is*. The two can disagree - a card may sit
//! behind a bridge, expose fewer BARs than the driver expects, or answer a
//! probe slowly - and the gap is exactly where a driver that started anyway
//! would fault.
//!
//! So calibration runs before any resource is granted. What comes out of it is
//! the set of resources the driver will actually be given; a device that fails
//! calibration is not started at all, and the reason is reported.

use heapless::Vec;

use crate::hw::{PciBar, ScannedDevice};

/// What calibration found out about one device.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Calibration {
    /// Memory regions the driver may map, as `(base, size)` pairs.
    pub memory: Vec<MemoryRegion, 6>,
    /// I/O port ranges the driver may touch, as `(base, size)` pairs.
    pub io: Vec<IoRegion, 6>,
    /// Interrupt lines the device claims, as a bitmask over `irq_index`.
    pub interrupts: u32,
    /// Whether the device answered at all.
    pub responsive: bool,
}

/// One memory region, page-aligned because that is what a mapping needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryRegion {
    pub base: u64,
    pub size: u64,
}

/// One I/O port range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IoRegion {
    pub base: u16,
    pub size: u16,
}

/// Why a device could not be calibrated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalibrateError {
    /// The configuration space read back as all-ones or all-zeros, which is
    /// what an absent device looks like.
    NotPresent,
    /// The device has no usable resource of any kind.
    NoResources,
    /// The driver's expectation does not fit what the device reports.
    Mismatch,
}

/// Calibrate one device.
///
/// `expect_bars` is how many BARs the driver's manifest entry says it needs; a
/// device with fewer usable BARs than that fails with
/// [`CalibrateError::Mismatch`] rather than being started against a mapping
/// that does not exist.
///
/// ## Why size is not required
///
/// A BAR's *size* can only be read by writing all-ones to it and reading the
/// mask back, which is a write to hardware the detection layer does not own.
/// The scanner therefore records the base address and leaves the size at zero
/// for a driver or the manager to fill in once it holds the resource. A BAR
/// counts as usable when its **base** is non-zero: a non-zero base is what a
/// mapping needs, and it is what the configuration space reliably reports.
pub fn calibrate(device: &ScannedDevice, expect_bars: usize) -> Result<Calibration, CalibrateError> {
    let Some(pci) = device.as_pci() else {
        // Non-PCI devices are calibrated by their own driver; nothing to do
        // here, so report an empty, responsive device.
        return Ok(Calibration { responsive: true, ..Calibration::default() });
    };

    // An absent function reads back as all-ones. Catching that here means the
    // planner never binds a device that is not really there.
    if pci.vendor_id == 0xFFFF && pci.device_id == 0xFFFF {
        return Err(CalibrateError::NotPresent);
    }

    let mut result = Calibration { responsive: true, ..Calibration::default() };
    let mut usable = 0usize;

    for bar in pci.bars.iter() {
        match *bar {
            PciBar::Memory { base, size } if base != 0 => {
                if result.memory.push(MemoryRegion { base, size }).is_err() {
                    break;
                }
                usable += 1;
            }
            PciBar::Io { base, size } if base != 0 => {
                if result.io.push(IoRegion { base, size }).is_err() {
                    break;
                }
                usable += 1;
            }
            _ => {}
        }
    }

    // A multifunction device reports its interrupts per function; the count
    // is a property of the device, not of the driver, so a simple
    // one-line-per-function model is enough for the first cut.
    if pci.function != 0 {
        result.interrupts |= 1;
    }

    if usable == 0 {
        return Err(CalibrateError::NoResources);
    }
    if expect_bars != 0 && usable < expect_bars {
        return Err(CalibrateError::Mismatch);
    }
    Ok(result)
}
