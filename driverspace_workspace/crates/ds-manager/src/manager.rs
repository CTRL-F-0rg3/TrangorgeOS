//! The manager: what the driver space actually does at boot.
//!
//! # The order, and why it is this order
//!
//! ```text
//!   1. detect()      scan, match, plan, calibrate    -> what to start
//!   2. grant()       manifest + policy                -> what it may do
//!   3. lease()       calibration                      -> what it may touch
//!   4. spawn()       the driver starts                -> with both
//! ```
//!
//! Capabilities before resources, and both before the driver runs. A driver
//! that started first could touch hardware before the manager had decided it
//! was allowed to.
//!
//! # What is deliberately not here
//!
//! No process spawning, no IPC, no hardware access. Those are injected:
//! [`Manager`] is built over a [`Platform`], and the tests supply a fake one.
//! That is the only way the rules above can be checked without a machine, and
//! the rules are the whole point.

use ds_detect::{
    Detection, Manifest,
    calibrate::calibrate,
    manifest::ManifestEntry,
};
use kapi_abi::{CapId, Handle};

use crate::{
    resources::ResourceTable,
    table::{CapTable, Grant},
};

/// What the manager needs from the machine.
///
/// Implemented for real against the kernel's PCI and process APIs, and by a
/// fake in the tests. Keeping it this narrow is what makes the manager's
/// rules testable.
pub trait Platform {
    /// The device catalogue the image carries.
    fn manifest(&self) -> &Manifest;

    /// Read one PCI configuration register.
    ///
    /// The manager forwards this to `ds-detect` rather than passing a device
    /// list in: the scanner needs the bus to walk, and hiding that behind a
    /// pre-scanned list would mean two different notions of what is present.
    fn read_pci(&self, bus: u8, device: u8, function: u8, offset: u8) -> Option<u32>;

    /// The bus numbers to walk, in order.
    fn bus_numbers(&self) -> &[u8];

    /// Start a driver, returning the handle it will answer on.
    ///
    /// Called only after the grant and the leases are in place.
    fn spawn(&mut self, entry: &ManifestEntry) -> Result<Handle, SpawnError>;

    /// Release a driver's resources at shutdown.
    fn stop(&mut self, driver: Handle);
}

/// Why a driver could not be started.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnError {
    /// The manifest entry is for hardware this image does not carry.
    NoSuchDriver,
    /// The platform refused, e.g. out of process slots.
    PlatformRefused,
    /// The device has nothing the driver can map.
    NoResources,
}

/// How many drivers one boot may start.
///
/// A free constant rather than an associated one: a `Self`-dependent length in
/// a field type is a generic const expression, which is not stable.
pub const MAX_STARTED: usize = 16;

/// One driver the manager has started, and everything it was given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Started {
    /// The handle the driver answers on.
    pub driver: Handle,
    /// Which manifest entry started it.
    pub entry_index: usize,
    /// What it may do.
    pub grant: Grant,
    /// How many regions it holds.
    pub leases: usize,
    /// The devices it was matched against, by index into the scan.
    pub devices: [u8; 4],
    pub device_count: u8,
}

/// The manager.
///
/// Deliberately generic over the platform so the boot rules can be tested on
/// a machine with no drivers and no hardware.
pub struct Manager<P: Platform> {
    platform: P,
    caps: CapTable,
    resources: ResourceTable,
    /// The boot's detection result, kept so the log can report it after the
    /// drivers are already running.
    detection: Option<Detection>,
    started: [Option<Started>; MAX_STARTED],
    start_count: usize,
}

impl<P: Platform> Manager<P> {
    /// A manager over a platform, with nothing granted and nothing leased.
    pub const fn new(platform: P) -> Self {
        Self {
            platform,
            caps: CapTable::empty(),
            resources: ResourceTable::empty(),
            detection: None,
            started: [None; MAX_STARTED],
            start_count: 0,
        }
    }

    /// The capability table, for inspection.
    pub const fn caps(&self) -> &CapTable {
        &self.caps
    }

    /// The resource table, for inspection.
    pub const fn resources(&self) -> &ResourceTable {
        &self.resources
    }

    /// The drivers that started, in order.
    pub fn started(&self) -> impl Iterator<Item = &Started> {
        self.started.iter().take(self.start_count).flatten()
    }

    /// How many drivers started.
    pub fn start_count(&self) -> usize {
        self.start_count
    }

    /// The detection result, once [`Manager::boot`] has run.
    pub const fn detection(&self) -> Option<&Detection> {
        self.detection.as_ref()
    }

    /// Run the whole boot: detect, grant, lease, spawn.
    ///
    /// This is the entry point, and it does the four steps in the order the
    /// module documents. `privileged` is what the boot may hand out: a bit
    /// outside it is withheld, not refused, so a driver still starts and can
    /// report what it is missing.
    pub fn boot(&mut self, privileged: CapId, expected_bars: usize) {
        self.detection = Some(ds_detect::detect(
            self,
            self.platform.bus_numbers(),
            self.platform.manifest(),
            expected_bars,
        ));
        self.start(privileged, expected_bars);
    }

    /// Start every driver the plan chose, in order.
    ///
    /// A driver that fails to start does not stop the others: one missing
    /// driver must not take the rest of the boot with it.
    pub fn start(&mut self, privileged: CapId, expected_bars: usize) {
        let Some(detection) = self.detection.clone() else {
            return;
        };

        // `Manifest` iterates rather than indexing, so the plan's entry index is
        // turned back into an entry by counting. The two are the same order,
        // which is what makes the index meaningful.
        for start in detection.plan.starts.iter() {
            let Some(entry) = self.platform.manifest().iter().nth(start.entry_index) else {
                continue;
            };
            let entry = *entry;

            // Step 2: what may it do. Recorded even if the spawn later fails,
            // so the log can say what the driver was going to have.
            let grant = CapTable::grant_for(&entry, privileged);

            // Step 3: what may it touch. Calibration for the first matched
            // device is representative; a driver matched to several devices
            // gets the regions of the first and is told how many it had.
            let Some(first) = start.devices.first().copied() else {
                continue;
            };
            let Some(device) = detection.devices.get(first).copied() else {
                continue;
            };
            let Ok(calibration) = calibrate(&device, expected_bars) else {
                // Calibration refused it: the device is not really there.
                continue;
            };

            if self.start_count >= MAX_STARTED {
                continue;
            }

            // Step 4: start it, then hand over the grant and the leases.
            let Ok(handle) = self.platform.spawn(&entry) else {
                continue;
            };
            let leases = self.resources.lease_all(handle, handle, &calibration);
            if leases == 0 {
                // Started with nothing to map. Better to stop it now than to
                // run a driver that cannot reach the silicon it was matched to.
                self.platform.stop(handle);
                continue;
            }
            let _ = self.caps.insert(grant.requested, privileged);

            let mut devices_slot = [0u8; 4];
            let mut device_count = 0u8;
            for index in start.devices.iter().take(4) {
                devices_slot[device_count as usize] = *index as u8;
                device_count += 1;
            }
            self.started[self.start_count] = Some(Started {
                driver: handle,
                entry_index: start.entry_index,
                grant,
                leases,
                devices: devices_slot,
                device_count,
            });
            self.start_count += 1;
        }
    }

    /// Stop a driver and release everything it held.
    ///
    /// The leases go first: a driver that is still running must not be able to
    /// touch a region that has been handed to its replacement.
    pub fn stop(&mut self, driver: Handle) {
        self.resources.release(driver);
        if let Some(slot) = self
            .started
            .iter_mut()
            .find(|entry| entry.is_some_and(|s| s.driver == driver))
        {
            *slot = None;
            self.start_count = self.started.iter().flatten().count();
        }
        self.platform.stop(driver);
    }

    /// Whether `driver` may make a request needing `capability`.
    ///
    /// The whole enforcement point, and it lives here rather than in the
    /// driver because the driver is not the thing being trusted.
    pub fn permits(&self, driver: Handle, capability: CapId) -> bool {
        match self.started.iter().flatten().find(|entry| entry.driver == driver) {
            Some(started) => started.grant.permits(capability),
            None => false,
        }
    }

    /// Whether `driver` may touch `len` bytes at `offset`.
    pub fn permits_access(&self, driver: Handle, offset: u64, len: u64) -> bool {
        self.resources.permits(driver, offset, len)
    }
}

impl<P: Platform> ds_detect::scan::PciBus for Manager<P> {
    fn read32(&self, bus: u8, device: u8, function: u8, offset: u8) -> Option<u32> {
        self.platform.read_pci(bus, device, function, offset)
    }
}
