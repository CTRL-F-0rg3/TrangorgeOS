//! Manager tests, over a machine that does not exist.
//!
//! The rules worth testing here are the ones a hardware-less machine is the
//! only way to test: who gets started, what they are given, and what they may
//! not reach. A fake platform describes a machine; the manager's behaviour on
//! it is the same behaviour it would have on silicon.

use ds_detect::manifest::{Manifest, ManifestEntry};
use ds_manager::resources::Region;
use ds_manager::{Manager, Platform, SpawnError};
use kapi_abi::{CapId, Handle};

/// An Arc A770, at 0x8000_0000 with a 2 MiB BAR0.
const INTEL_GPU: (u16, u16, u64, u64) = (0x8086, 0x56A0, 0x8000_0000, 0x20_0000);
/// A Radeon RX 7900.
const AMD_GPU: (u16, u16, u64, u64) = (0x1002, 0x744C, 0x9000_0000, 0x20_0000);

/// A machine described in software.
struct FakePlatform {
    manifest: Manifest,
    buses: [u8; 1],
    /// `(vendor, device, bar base, bar size)` triples the machine contains.
    devices: [(u16, u16, u64, u64); 2],
    device_count: usize,
    /// Refuse every spawn, to test that one failure does not stop the rest.
    refuse_spawn: bool,
    /// Every handle the manager asked to start.
    spawned: [Handle; 4],
    spawned_count: usize,
    /// Every handle the manager asked to stop.
    stopped: [Handle; 4],
    stopped_count: usize,
}

impl FakePlatform {
    fn new(manifest: Manifest) -> Self {
        Self {
            manifest,
            buses: [0],
            devices: [INTEL_GPU, AMD_GPU],
            device_count: 1,
            refuse_spawn: false,
            spawned: [Handle(0); 4],
            spawned_count: 0,
            stopped: [Handle(0); 4],
            stopped_count: 0,
        }
    }

    /// A machine whose only device has no usable BAR.
    fn with_broken_bars() -> Self {
        let mut platform = Self::new(Manifest::empty());
        platform.devices[0] = (INTEL_GPU.0, INTEL_GPU.1, 0, 0);
        platform
    }
}

impl Platform for FakePlatform {
    fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    fn read_pci(&self, _bus: u8, device: u8, function: u8, offset: u8) -> Option<u32> {
        if device as usize >= self.device_count || function != 0 {
            return None;
        }
        let (vendor, id, base, size) = self.devices[device as usize];
        Some(match offset {
            // The scanner reads the vendor id from the low half and the device
            // id from the high half, which is the PCI layout: 0x00 holds the
            // vendor, 0x02 the device.
            0x00 => (vendor as u32) | ((id as u32) << 16),
            // Class 03, subclass 00: display controller.
            0x08 => 0x0300_0000,
            0x0E => 0x00, // header type: not multifunction
            // BAR0 holds the register window, BAR2 the aperture. Both are
            // 32-bit memory BARs: `decode_bar` masks the low four bits off and
            // reads the base from what is left, so a raw value is just the base
            // with the type bits clear. Bit 0 clear means memory, bit 1 clear
            // means 32-bit, which keeps each base in one register.
            0x10 => (base as u32) & 0xFFFF_FFF0,
            0x14 => 0,
            0x18 => (size as u32) & 0xFFFF_FFF0,
            0x1C => 0,
            // BAR4 and BAR5 absent: a small fake GPU has two windows.
            _ => 0,
        })
    }

    fn bus_numbers(&self) -> &[u8] {
        &self.buses
    }

    fn spawn(&mut self, entry: &ManifestEntry) -> Result<Handle, SpawnError> {
        if self.refuse_spawn {
            return Err(SpawnError::PlatformRefused);
        }
        let handle = Handle((self.spawned_count + 1) as u32);
        if self.spawned_count < self.spawned.len() {
            self.spawned[self.spawned_count] = handle;
        }
        self.spawned_count += 1;
        let _ = entry;
        Ok(handle)
    }

    fn stop(&mut self, driver: Handle) {
        if self.stopped_count < self.stopped.len() {
            self.stopped[self.stopped_count] = driver;
        }
        self.stopped_count += 1;
    }
}

// ── what gets started ──────────────────────────────────────────────────────────

#[test]
fn a_manifest_with_no_entries_starts_nothing() {
    let mut manager = Manager::new(FakePlatform::new(Manifest::empty()));
    manager.boot(CapId::KERNEL_ALL, 1);
    assert_eq!(manager.start_count(), 0);
    assert!(manager.detection().is_some(), "but it still detected");
}

#[test]
fn a_matching_entry_starts_its_driver() {
    let mut manifest = Manifest::empty();
    manifest.push(ManifestEntry::exact("IntelGpu-TrangorgeOS", 1, INTEL_GPU.0, INTEL_GPU.1));

    let mut manager = Manager::new(FakePlatform::new(manifest));
    manager.boot(CapId::KERNEL_ALL, 1);

    let detection = manager.detection().expect("detection ran");
    assert_eq!(
        detection.devices.len(),
        1,
        "the scanner should have found the GPU: {:?}",
        detection.devices
    );
    assert_eq!(manager.start_count(), 1, "the Intel GPU on the machine got a driver");
    let started = manager.started().next().expect("one started");
    assert_eq!(started.entry_index, 0);
    assert!(started.leases > 0, "and it was given something to map");
}

#[test]
fn a_machine_with_no_matching_hardware_starts_nothing() {
    // An image carrying the Intel driver, booted on a machine with an AMD card
    // and no Intel entry: the driver must stay in the image and out of the
    // boot. This is the whole point of the manifest.
    let mut manifest = Manifest::empty();
    manifest.push(ManifestEntry::exact("IntelGpu-TrangorgeOS", 1, INTEL_GPU.0, INTEL_GPU.1));
    let mut platform = FakePlatform::new(manifest);
    platform.devices[0] = AMD_GPU;

    let mut manager = Manager::new(platform);
    manager.boot(CapId::KERNEL_ALL, 1);
    assert_eq!(manager.start_count(), 0, "the Intel driver has no hardware here");
}

#[test]
fn a_refused_spawn_does_not_stop_the_others() {
    // One driver failing to start must not take the boot with it. This is why
    // `start` records and continues rather than propagating.
    let mut manifest = Manifest::empty();
    manifest.push(ManifestEntry::exact("IntelGpu-TrangorgeOS", 1, INTEL_GPU.0, INTEL_GPU.1));
    let mut platform = FakePlatform::new(manifest);
    platform.refuse_spawn = true;

    let mut manager = Manager::new(platform);
    manager.boot(CapId::KERNEL_ALL, 1);
    assert_eq!(manager.start_count(), 0, "the spawn failed, so nothing started");
    // And the boot completed rather than panicking.
    assert!(manager.detection().is_some());
}

#[test]
fn a_device_with_no_usable_bars_is_not_started() {
    // Present in the configuration space, absent on the silicon. Calibration
    // refuses it, and a driver with nothing to map is worse than no driver.
    let mut manifest = Manifest::empty();
    manifest.push(ManifestEntry::exact("IntelGpu-TrangorgeOS", 1, INTEL_GPU.0, INTEL_GPU.1));
    let platform = FakePlatform::with_broken_bars();

    let mut manager = Manager::new(platform);
    manager.boot(CapId::KERNEL_ALL, 1);
    assert_eq!(manager.start_count(), 0, "a GPU with no BAR cannot be driven");
}

// ── what a driver is given ─────────────────────────────────────────────────────

#[test]
fn a_driver_gets_what_the_manifest_asked_for() {
    let mut manifest = Manifest::empty();
    let mut entry = ManifestEntry::exact("IntelGpu-TrangorgeOS", 1, INTEL_GPU.0, INTEL_GPU.1);
    entry.capabilities = CapId::GPU_ENUMERATE.union(CapId::GPU_DEVICE);
    manifest.push(entry);

    let mut manager = Manager::new(FakePlatform::new(manifest));
    manager.boot(CapId::KERNEL_ALL, 1);

    let started = manager.started().next().expect("one started");
    assert!(started.grant.permits(CapId::GPU_ENUMERATE));
    assert!(started.grant.permits(CapId::GPU_DEVICE));
    assert!(started.grant.withheld.is_empty());
}

#[test]
fn the_boot_policy_withholds_rather_than_refuses() {
    // A driver asking for everything, on a boot that only hands out the
    // read-only bit, must still start - and be told what it is missing. A
    // refusal would mean no driver at all, which is a worse failure than a
    // driver that cannot submit.
    let mut manifest = Manifest::empty();
    let mut entry = ManifestEntry::exact("IntelGpu-TrangorgeOS", 1, INTEL_GPU.0, INTEL_GPU.1);
    entry.capabilities = CapId::GPU_ENUMERATE.union(CapId::GPU_DEVICE);
    manifest.push(entry);

    let mut manager = Manager::new(FakePlatform::new(manifest));
    manager.boot(CapId::GPU_ENUMERATE, 1); // only the read-only bit

    let started = manager.started().next().expect("it still started");
    assert!(started.grant.permits(CapId::GPU_ENUMERATE), "and it can still look");
    assert!(!started.grant.permits(CapId::GPU_DEVICE), "but it may not drive");
    assert!(!started.grant.withheld.is_empty(), "and it knows what it lost");
}

#[test]
fn an_unknown_driver_is_granted_nothing() {
    // A handle the manager never issued may do nothing. This is what stops a
    // client inventing a driver identity and asking under it.
    let mut manager = Manager::new(FakePlatform::new(Manifest::empty()));
    manager.boot(CapId::KERNEL_ALL, 1);

    assert!(!manager.permits(Handle(99), CapId::KERNEL_ALL));
    assert!(!manager.permits(Handle(0), CapId::DEV_MMIO));
}

// ── what a driver may touch ────────────────────────────────────────────────────

#[test]
fn a_started_driver_may_touch_its_own_region() {
    let mut manifest = Manifest::empty();
    manifest.push(ManifestEntry::exact("IntelGpu-TrangorgeOS", 1, INTEL_GPU.0, INTEL_GPU.1));
    let mut manager = Manager::new(FakePlatform::new(manifest));
    manager.boot(CapId::KERNEL_ALL, 1);

    let handle = Handle(1);
    // The fake machine reports two memory BARs, so the driver holds two
    // leases: a real GPU has a register window and a large-memory window.
    assert_eq!(manager.resources().for_driver(handle), 2, "it holds both BARs");
    // The BAR0 base is at 0x8000_0000. `ds-detect` does not read BAR sizes, so
    // the manager falls back to a 1 GiB bound.
    assert!(manager.permits_access(handle, 0x8000_0000, 4), "a register read");
    assert!(manager.permits_access(Handle(1), 0x801F_FFF0, 4), "further in");
    // But not a gigabyte further, which is past the assumed bound and therefore
    // past any real BAR.
    assert!(!manager.permits_access(Handle(1), 0x8000_0000 + (1 << 30), 4));
    // And not a plausible other device.
    assert!(!manager.permits_access(Handle(1), 0xFEB0_0000, 4));
}

#[test]
fn a_driver_may_not_use_another_drivers_lease() {
    let mut manifest = Manifest::empty();
    manifest.push(ManifestEntry::exact("IntelGpu-TrangorgeOS", 1, INTEL_GPU.0, INTEL_GPU.1));
    manifest.push(ManifestEntry::exact("OtherGpu-TrangorgeOS", 2, AMD_GPU.0, AMD_GPU.1));
    let mut platform = FakePlatform::new(manifest);
    platform.device_count = 2;

    let mut manager = Manager::new(platform);
    manager.boot(CapId::KERNEL_ALL, 1);
    assert_eq!(manager.start_count(), 2);

    // Driver 1 may not use driver 2's BAR0, even though both have a lease and
    // both are the manager's own. Driver 1's BAR0 is at 0x8000_0000 and driver
    // 2's at 0x9000_0000.
    assert!(manager.permits_access(Handle(1), 0x8000_0000, 4));
    assert!(
        !manager.permits_access(Handle(2), 0x8000_0000, 4),
        "one driver's lease is not another's"
    );
    assert!(manager.permits_access(Handle(2), 0x9000_0000, 4), "but it has its own");
}

#[test]
fn a_region_of_zero_is_never_leased() {
    // A driver asking to be handed address zero is asking for whatever page
    // happens to be there.
    let mut manifest = Manifest::empty();
    manifest.push(ManifestEntry::exact("IntelGpu-TrangorgeOS", 1, INTEL_GPU.0, INTEL_GPU.1));
    let mut manager = Manager::new(FakePlatform::new(manifest));
    manager.boot(CapId::KERNEL_ALL, 1);

    assert!(!manager.permits_access(Handle(1), 0, 4));
    assert!(!manager.permits_access(Handle(1), 0x1000, 4));
}

// ── shutdown ───────────────────────────────────────────────────────────────────

#[test]
fn stopping_a_driver_releases_its_leases() {
    let mut manifest = Manifest::empty();
    manifest.push(ManifestEntry::exact("IntelGpu-TrangorgeOS", 1, INTEL_GPU.0, INTEL_GPU.1));
    let mut manager = Manager::new(FakePlatform::new(manifest));
    manager.boot(CapId::KERNEL_ALL, 1);
    assert!(manager.permits_access(Handle(1), 0x8000_0000, 4));

    manager.stop(Handle(1));

    // The driver is gone, so its address belongs to nobody. A stale mapping
    // must not outlive it.
    assert!(!manager.permits_access(Handle(1), 0x8000_0000, 4));
    assert!(!manager.permits(Handle(1), CapId::KERNEL_ALL));
    assert_eq!(manager.start_count(), 0);
}

#[test]
fn a_region_is_mappable_only_when_it_is_real() {
    // A zero base is the configuration space saying "unimplemented". That is
    // the one thing that disqualifies a region outright.
    let region = Region { base: 0x1000, size: 0x1000 };
    assert!(region.is_mappable());
    assert!(region.contains(0x1000, 4));
    assert!(region.contains(0x1FFC, 4));
    assert!(!region.contains(0x1FFC, 8), "the last four bytes are the end");
    assert!(!region.contains(0x2000, 4), "past the end");

    assert!(!(Region { base: 0, size: 0x1000 }).is_mappable());
    assert!(!(Region { base: 0, size: 0 }).is_mappable());
    assert!(!region.contains(0, 4), "and never at address zero");

    // A sized region uses its own size; an no_size one falls back to the bound.
    let sized = Region { base: 0x1000, size: 0x100 };
    assert!(sized.contains(0x1080, 4));
    assert!(!sized.contains(0x1200, 4), "past a size we actually know");

    let no_size = Region { base: 0x1000, size: 0 };
    assert!(no_size.is_mappable(), "an no_size BAR is still mappable");
    assert!(no_size.contains(0x1000 + no_size.limit_bytes() - 4, 4));
    assert!(!no_size.contains(0x1000 + no_size.limit_bytes(), 4));

    // An access whose end overflows is refused, not wrapped to something that
    // fits.
    assert!(!region.contains(u64::MAX, 8));
}
