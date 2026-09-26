//! `ds-detect` - the detection layer: find hardware, decide what to start.
//!
//! ## The problem this solves
//!
//! A driver image has to carry every driver the machine *might* need, because
//! at build time nobody knows which machine it will boot on. Starting them all
//! is the obvious answer and the wrong one: each one probes registers, claims
//! resources, and a driver with no hardware usually ends up either inert or,
//! worse, actively confused by a device that is not its own.
//!
//! So the image carries everything and the system starts what it finds:
//!
//! ```text
//!   image (all drivers + manifest)
//!        │
//!   scan   read PCI configuration space           -> what hardware exists
//!        │
//!   match  manifest entry vs (vendor, device)      -> what could drive it
//!        │
//!   plan   one driver per device, honour policy     -> what to start, in what order
//!        │
//!   calibrate  read BARs, check they are real      -> what to grant
//!        │
//!   start  load, grant, run
//! ```
//!
//! The stages are separate types, not one function, because each has a
//! different failure mode: a scan can miss a device, a match can be too broad,
//! a plan can drop a driver, and a calibration can find a device that is
//! present in the configuration space and absent on the silicon. Keeping them
//! apart is what makes each of those diagnosable from a boot log.
//!
//! ## Why it is `no_std`
//!
//! It lives in `crates/`, which the workspace rules hold to `no_std`, and it
//! has to run before anything is loaded. There is no allocator and no
//! filesystem: the manifest is a fixed-size table carried in the image.

#![no_std]

pub mod calibrate;
pub mod hw;
pub mod manifest;
pub mod plan;
pub mod scan;

use heapless::Vec;

pub use calibrate::{CalibrateError, Calibration};
pub use hw::{PciBar, PciDevice, ScannedDevice};
pub use manifest::{LoadPolicy, Manifest, ManifestEntry};
pub use plan::{BootPlan, StartReason};
pub use scan::{PciBus, MAX_DEVICES};

/// How many buses a scan will walk.
///
/// 256 is the architectural limit for the x86 bus number; the scanner takes a
/// slice so a caller can hand it a shorter list when the firmware says how
/// many it installed.
pub const MAX_BUSES: usize = 256;

/// What the detection layer decided, ready for the loader to act on.
///
/// Deliberately `Clone` but not `Copy`: the three vectors are `heapless::Vec`
/// over inline storage, and copying them per device would defeat the point of
/// the fixed-size tables. A `Detection` is built once, at boot, and consumed.
#[derive(Clone, Debug)]
pub struct Detection {
    /// Everything the scan found, whether or not it is driven.
    pub devices: Vec<ScannedDevice, MAX_DEVICES>,
    /// The plan, naming manifest indices for the drivers to start.
    pub plan: BootPlan,
    /// Manifest entries that matched a device but failed calibration, with the
    /// reason. A driver in here is *not* started.
    pub rejected: Vec<(usize, CalibrateError), MAX_DEVICES>,
}

impl Detection {
    /// The number of drivers that will actually run.
    pub fn start_count(&self) -> usize {
        self.plan.start_count()
    }

    /// A one-line summary for the boot log: how many devices, how many drivers.
    pub fn summary(&self) -> heapless::String<64> {
        let mut text = heapless::String::new();
        let _ = core::fmt::Write::write_fmt(
            &mut text,
            format_args!(
                "detected {} device(s), starting {} driver(s)",
                self.devices.len(),
                self.plan.start_count()
            ),
        );
        text
    }
}

/// Run the whole layer: scan, match, plan, calibrate.
///
/// `expected_bars` says how many usable BARs each driver's entry needs; pass
/// `1` for a driver that only needs a single register window, which is the
/// common case. A device that cannot satisfy its entry is recorded in
/// [`Detection::rejected`] rather than started against a mapping that does
/// not exist.
pub fn detect<B: PciBus>(
    bus: &B,
    bus_numbers: &[u8],
    manifest: &Manifest,
    expected_bars: usize,
) -> Detection {
    let devices = scan::scan_pci(bus, bus_numbers);
    let plan = plan::plan(manifest, &devices);
    let mut rejected: Vec<(usize, CalibrateError), MAX_DEVICES> = Vec::new();

    // Calibrate only the drivers that actually matched something: reading a
    // device no driver will claim is wasted work on the boot path.
    for start in plan.starts.iter() {
        if !start.calibrate {
            continue;
        }
        for device_index in start.devices.iter() {
            let Some(device) = devices.get(*device_index) else {
                continue;
            };
            if let Err(error) = calibrate::calibrate(device, expected_bars) {
                let _ = rejected.push((start.entry_index, error));
            }
        }
    }

    Detection { devices, plan, rejected }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ManifestEntry;
    use crate::scan::PciBus;

    // The crate is `no_std`, so the test harness needs an explicit `extern
    // crate std;` before it can use the standard collections.
    extern crate std;
    use std::collections::BTreeMap;

    /// A bus described by a table, so a whole machine can be set up in a test.
    struct FakeBus {
        registers: BTreeMap<(u8, u8, u8, u8), u32>,
    }

    impl FakeBus {
        fn new() -> Self {
            Self { registers: BTreeMap::new() }
        }

        /// Write a function's identifying registers, as hardware stores them.
        fn describe(&mut self, bus: u8, slot: u8, vendor: u16, device: u16, class: u32) {
            self.registers
                .insert((bus, slot, 0, 0x00), vendor as u32 | ((device as u32) << 16));
            // Register 0x08: sub-class in 15:8, class in 23:16.
            self.registers
                .insert((bus, slot, 0, 0x08), ((class >> 8) & 0xFF) << 16 | (class & 0xFF) << 8);
            self.registers.insert((bus, slot, 0, 0x0E), 0);
        }

        /// A device with one usable memory BAR.
        fn device(&mut self, bus: u8, slot: u8, vendor: u16, device: u16, class: u32) {
            self.describe(bus, slot, vendor, device, class);
            self.registers.insert((bus, slot, 0, 0x10), 0xFEB0_0004);
        }

        /// A device with no usable BAR, as a bus-bridge stub looks.
        fn stub(&mut self, bus: u8, slot: u8, vendor: u16, device: u16, class: u32) {
            self.describe(bus, slot, vendor, device, class);
        }
    }

    impl PciBus for FakeBus {
        fn read32(&self, bus: u8, device: u8, function: u8, offset: u8) -> Option<u32> {
            self.registers.get(&(bus, device, function, offset)).copied()
        }
    }

    /// A manifest carrying more drivers than any test machine has hardware for.
    fn crowded_manifest() -> Manifest {
        let mut manifest = Manifest::empty();
        manifest.push(ManifestEntry::exact("Alsa-TrangorgeOS", 4, 0x8086, 0x2415));
        manifest.push(ManifestEntry::by_class("NetTrangorgeOS", 5, "02"));
        manifest.push(ManifestEntry::by_class("GfxTrangorgeOS", 6, "03"));
        manifest.push(ManifestEntry::by_class("InputTrangorgeOS", 7, "09"));
        manifest.push(ManifestEntry::by_class("UsbTrangorgeOS", 8, "0C"));
        manifest.push(ManifestEntry::by_class("SataTrangorgeOS", 9, "01"));
        manifest.push(ManifestEntry::by_class("VgaTrangorgeOS", 10, "0300"));
        manifest
    }

    #[test]
    fn a_machine_with_one_card_starts_one_driver_out_of_seven() {
        // The machine has an audio controller and nothing else.
        let mut bus = FakeBus::new();
        bus.device(0, 0x1f, 0x8086, 0x2415, 0x0401);

        let manifest = crowded_manifest();
        let result = detect(&bus, &[0], &manifest, 1);

        assert_eq!(result.devices.len(), 1, "one device on the bus");
        assert_eq!(result.start_count(), 1, "and one driver started out of seven");
        assert!(result.rejected.is_empty());
    }

    #[test]
    fn only_the_matching_driver_runs_even_when_the_class_also_matches() {
        // The audio device matches both the exact entry and a class-04 entry.
        let mut bus = FakeBus::new();
        bus.device(0, 0x1f, 0x8086, 0x2415, 0x0401);

        let mut manifest = Manifest::empty();
        manifest.push(ManifestEntry::exact("Alsa-TrangorgeOS", 4, 0x8086, 0x2415));
        manifest.push(ManifestEntry::by_class("GenericAudio", 10, "04"));

        let result = detect(&bus, &[0], &manifest, 1);
        // One device, so exactly one driver: the first entry that matched.
        assert_eq!(result.start_count(), 1);
        assert_eq!(result.plan.starts[0].entry_index, 0);
        assert_eq!(result.plan.unbound.len(), 0);
    }

    #[test]
    fn hardware_the_image_does_not_carry_stays_unbound() {
        // An audio and an ethernet card, on an image that only knows audio.
        let mut bus = FakeBus::new();
        bus.device(0, 0x1f, 0x8086, 0x2415, 0x0401);
        bus.device(0, 0x02, 0x8086, 0x100e, 0x0200);

        let mut manifest = Manifest::empty();
        manifest.push(ManifestEntry::exact("Alsa-TrangorgeOS", 4, 0x8086, 0x2415));

        let result = detect(&bus, &[0], &manifest, 1);
        assert_eq!(result.devices.len(), 2, "both cards are visible");
        assert_eq!(result.start_count(), 1, "but only the audio driver runs");
        assert_eq!(result.plan.unbound.len(), 1, "the ethernet card is unbound");
    }

    #[test]
    fn a_device_that_fails_calibration_is_not_started() {
        // A device that answers in the configuration space but has no BAR.
        let mut bus = FakeBus::new();
        bus.stub(0, 0x1f, 0x8086, 0x2415, 0x0401);

        let mut manifest = Manifest::empty();
        manifest.push(ManifestEntry::exact("Alsa-TrangorgeOS", 4, 0x8086, 0x2415));

        let result = detect(&bus, &[0], &manifest, 1);
        assert_eq!(result.devices.len(), 1, "it was seen");
        assert_eq!(result.rejected.len(), 1, "but refused calibration");
        assert_eq!(result.rejected[0].1, CalibrateError::NoResources);
    }

    #[test]
    fn a_driver_that_needs_more_bars_than_exist_is_rejected() {
        let mut bus = FakeBus::new();
        bus.device(0, 0x1f, 0x8086, 0x2415, 0x0401);

        let mut manifest = Manifest::empty();
        manifest.push(ManifestEntry::exact("Alsa-TrangorgeOS", 4, 0x8086, 0x2415));

        // The device offers one BAR; the driver was built expecting three.
        let result = detect(&bus, &[0], &manifest, 3);
        assert_eq!(result.rejected.len(), 1);
        assert_eq!(result.rejected[0].1, CalibrateError::Mismatch);
    }

    #[test]
    fn one_driver_never_claims_two_devices() {
        // Two identical audio cards, one driver.
        let mut bus = FakeBus::new();
        bus.device(0, 0x1f, 0x8086, 0x2415, 0x0401);
        bus.device(0, 0x1e, 0x8086, 0x2415, 0x0401);

        let mut manifest = Manifest::empty();
        manifest.push(ManifestEntry::exact("Alsa-TrangorgeOS", 4, 0x8086, 0x2415));

        let result = detect(&bus, &[0], &manifest, 1);
        assert_eq!(result.start_count(), 1);
        // Both cards are reported to the one driver that started.
        assert_eq!(result.plan.starts[0].devices.len(), 2);
        assert_eq!(result.plan.unbound.len(), 0);
    }

    #[test]
    fn the_summary_reports_both_numbers() {
        let mut bus = FakeBus::new();
        bus.device(0, 0x1f, 0x8086, 0x2415, 0x0401);
        bus.device(0, 0x02, 0x8086, 0x100e, 0x0200);

        let mut manifest = Manifest::empty();
        manifest.push(ManifestEntry::exact("Alsa-TrangorgeOS", 4, 0x8086, 0x2415));

        let result = detect(&bus, &[0], &manifest, 1);
        let summary = result.summary();
        assert!(summary.contains("2 device"), "got {summary}");
        assert!(summary.contains("1 driver"), "got {summary}");
    }
}
