//! Which driver may touch which physical resource.
//!
//! # The rule
//!
//! A driver does not own a BAR, an interrupt or a DMA buffer. The manager
//! holds them and hands out a lease. A lease says: *this driver, on this
//! device, may map this range* - and nothing else.
//!
//! This is what makes the manager more than a launcher. Without it, a driver
//! that guesses a BAR index writes wherever the hardware happens to decode it,
//! and a driver that maps the wrong range corrupts a device belonging to
//! someone else.
//!
//! # Why a lease rather than a flag
//!
//! A lease is `(driver, region)`, and every access is checked against it. A
//! driver naming an address outside its own BAR is refused, because "a bit
//! further" is how an MMIO bug becomes a machine reset.

use ds_detect::calibrate::Calibration;
use kapi_abi::{CapId, Handle};

/// One mappable region, as the manager recorded it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub base: u64,
    pub size: u64,
}

impl Region {
    /// Whether the range is usable for a mapping.
    ///
    /// A zero base is not a region: it is the configuration space saying "this
    /// BAR is unimplemented", and mapping address zero is how a driver ends up
    /// writing to whatever physical page happens to be there.
    ///
    /// A zero *size* is not disqualifying. `ds-detect` leaves a BAR's size at
    /// zero on purpose, because reading it needs a write of all-ones to
    /// hardware the detection layer does not own. So a region with a base and
    /// no size is the normal case at this point, not a broken one.
    pub const fn is_mappable(&self) -> bool {
        self.base != 0
    }

    /// Whether `offset..offset + len` lies inside the region.
    ///
    /// A zero size means the size was never read, which `ds-detect` cannot do.
    /// In that case the check falls back to `limit_bytes`: a BAR of unknown
    /// size is still bounded, and an unbounded one would let a driver walk the
    /// whole physical space. Refusing an access that *might* be outside is the
    /// right way round for hardware.
    pub fn contains(&self, offset: u64, len: u64) -> bool {
        if self.base == 0 {
            return false;
        }
        let Some(end) = offset.checked_add(len) else {
            return false;
        };
        if offset < self.base {
            return false;
        }
        if self.size != 0 {
            return end <= self.base.saturating_add(self.size);
        }
        end <= self.base.saturating_add(self.limit_bytes())
    }

    /// How large an unsized region is assumed to be.
    ///
    /// A PCI BAR is at most 32 MiB for 32-bit addressing and 1 GiB for the
    /// 64-bit memory window most GPUs use. One gigabyte is the safe bound: it
    /// covers every real BAR, and it is small enough that a driver reaching
    /// past its own BAR is caught rather than allowed into the next device.
    pub const fn limit_bytes(&self) -> u64 {
        1 << 30
    }
}

/// A lease on one region's worth of resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lease {
    /// Which driver holds it.
    pub driver: Handle,
    /// Which device it belongs to.
    pub device: Handle,
    /// The region mapped.
    pub region: Region,
    /// The capability the lease carries, e.g. `DEV_MMIO`.
    pub kind: CapId,
}

impl Lease {
    /// Whether this lease permits an access of `len` bytes at `offset`.
    ///
    /// Not `const`: `Handle`'s `PartialEq` is derived rather than `const`, and
    /// a constant comparison is not worth making it one.
    pub fn permits(&self, driver: Handle, offset: u64, len: u64) -> bool {
        self.driver == driver && self.region.contains(offset, len)
    }
}

/// How many leases one table holds.
///
/// Sixteen drivers times one lease each, which is the common case: a GPU
/// driver leases its BARs once, at attach, and keeps them.
pub const MAX_LEASES: usize = 32;

/// What the manager is holding, and who may use it.
#[derive(Clone, Debug, Default)]
pub struct ResourceTable {
    leases: [Option<Lease>; MAX_LEASES],
    count: usize,
}

impl ResourceTable {
    /// A table holding nothing.
    pub const fn empty() -> Self {
        Self { leases: [None; MAX_LEASES], count: 0 }
    }

    /// Lease a region to a driver.
    ///
    /// `region` is rejected when it is not mappable, so a driver cannot be
    /// given a lease on address zero by asking for one.
    pub fn lease(&mut self, driver: Handle, device: Handle, region: Region, kind: CapId) -> bool {
        if self.count >= MAX_LEASES || !region.is_mappable() {
            return false;
        }
        self.leases[self.count] = Some(Lease { driver, device, region, kind });
        self.count += 1;
        true
    }

    /// Lease every usable region a calibration found, for one driver.
    ///
    /// Returns how many leases were made. A calibration with no usable region
    /// yields zero, which is how a device that is present in the configuration
    /// space and absent on the silicon is caught.
    pub fn lease_all(
        &mut self,
        driver: Handle,
        device: Handle,
        calibration: &Calibration,
    ) -> usize {
        let mut count = 0;
        for region in calibration.memory.iter() {
            let region = Region { base: region.base, size: region.size };
            if !region.is_mappable() {
                continue;
            }
            if self.lease(driver, device, region, CapId::MEM_MAP) {
                count += 1;
            }
        }
        count
    }

    /// Whether `driver` may touch `len` bytes at `offset`.
    ///
    /// The check every mapped access goes through. It is deliberately strict: a
    /// driver with two leases still has to name the one it means, and an access
    /// that fits in neither is refused rather than rounded to the nearest.
    pub fn permits(&self, driver: Handle, offset: u64, len: u64) -> bool {
        self.leases
            .iter()
            .take(self.count)
            .flatten()
            .any(|lease| lease.permits(driver, offset, len))
    }

    /// The leases one driver holds.
    pub fn for_driver(&self, driver: Handle) -> usize {
        self.leases
            .iter()
            .take(self.count)
            .flatten()
            .filter(|lease| lease.driver == driver)
            .count()
    }

    /// Release everything a driver held, as its shutdown.
    pub fn release(&mut self, driver: Handle) -> usize {
        let mut released = 0;
        for slot in self.leases.iter_mut() {
            if let Some(lease) = slot {
                if lease.driver == driver {
                    *slot = None;
                    released += 1;
                }
            }
        }
        self.count = self.leases.iter().flatten().count();
        released
    }

    /// How many leases are live.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether nothing is leased.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

