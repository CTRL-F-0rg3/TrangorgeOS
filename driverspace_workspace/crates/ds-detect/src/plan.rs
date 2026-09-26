//! The boot plan: which drivers to start, and with what.
//!
//! This is the piece that answers the question the whole layer exists for -
//! "the image carries twelve drivers; the machine has an audio controller and
//! a network card; which two do I actually start?"
//!
//! ## The rules, and why each one exists
//!
//! 1. **Match by exact id first, then by class.** An exact `(vendor, device)`
//!    entry knows what it is talking to; a class entry is a fallback for
//!    hardware the build did not enumerate. Running both would give one device
//!    two drivers.
//! 2. **One driver per device.** The first match wins, and the rest are
//!    recorded as conflicts rather than started.
//! 3. **Skip what the image does not carry.** A device with no matching entry
//!    stays unbound; that is the normal case on a machine with hardware this
//!    image was not built for.
//! 4. **Honour the policy.** `OnDemand` needs a match, `Always` does not, and
//!    `Manual` never starts automatically.
//! 5. **Calibrate only what needs it.** Calibration reads hardware state to
//!    find the limits, so it is only meaningful once the driver matched
//!    something.
//!
//! Point 4 and 5 are the reason this is a *plan* and not a filter: the answer
//! is a list of actions to perform, each with the resources it will be granted.

use heapless::Vec;

use crate::{
    hw::ScannedDevice,
    manifest::{LoadPolicy, Manifest, ManifestEntry},
};

/// How many drivers one boot plan may start.
pub const MAX_PLAN: usize = Manifest::MAX_ENTRIES;

/// One driver the boot decided to start, and why.
///
/// `Clone` but not `Copy`, for the same reason as [`BootPlan`]: the matched
/// device list is a `heapless::Vec`, and a plan is built once and consumed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedStart {
    /// Index into the manifest, so the caller can recover the full entry.
    pub entry_index: usize,
    /// The devices that matched this driver.
    pub devices: Vec<usize, MAX_PLAN>,
    /// Whether to calibrate before handing the driver its resources.
    pub calibrate: bool,
    /// Why this driver was chosen, for the boot log.
    pub reason: StartReason,
}

/// Why a driver is in the plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartReason {
    /// A scanned device matched this entry.
    DeviceMatched,
    /// Policy is `Always`, so it starts regardless.
    PolicyAlways,
    /// Policy is `critical`, so it must be up for the others to work.
    Critical,
}

/// The outcome of planning: what to start, and what was deliberately skipped.
#[derive(Clone, Debug)]
pub struct BootPlan {
    pub starts: Vec<PlannedStart, MAX_PLAN>,
    /// Devices no entry claimed, by index into the scanned list.
    pub unbound: Vec<usize, MAX_PLAN>,
    /// Drivers present but held back by policy `Manual`.
    pub deferred: Vec<usize, MAX_PLAN>,
}

impl BootPlan {
    /// An empty plan.
    pub fn empty() -> Self {
        Self {
            starts: Vec::new(),
            unbound: Vec::new(),
            deferred: Vec::new(),
        }
    }

    /// Whether anything at all will be started.
    pub fn is_empty(&self) -> bool {
        self.starts.is_empty()
    }

    /// The number of drivers that will run, as opposed to those skipped.
    pub fn start_count(&self) -> usize {
        self.starts.len()
    }
}

/// Build the boot plan for `manifest` against the devices found in `devices`.
///
/// `devices` is the scan result; the indices stored in the plan refer to it, so
/// a caller can go back to the hardware for the BARs and the BDF.
pub fn plan(manifest: &Manifest, devices: &[ScannedDevice]) -> BootPlan {
    let mut result = BootPlan::empty();
    // Which device indices already have a driver. This is what enforces one
    // driver per device.
    let mut claimed: Vec<bool, MAX_PLAN> = Vec::new();
    while claimed.len() < devices.len() {
        if claimed.push(false).is_err() {
            break;
        }
    }

    // ── rules 1 and 2: exact matches first, then class matches ─────────────
    for (index, entry) in manifest.iter().enumerate() {
        if entry.policy == LoadPolicy::Manual {
            // rule 4: present, but never automatic.
            let _ = result.deferred.push(index);
            continue;
        }

        let mut matched: Vec<usize, MAX_PLAN> = Vec::new();
        for (device_index, device) in devices.iter().enumerate() {
            if claimed[device_index] || !entry.matches(device) {
                continue;
            }
            if matched.push(device_index).is_err() {
                break;
            }
        }

        if matched.is_empty() {
            // Nothing matched: this driver stays out of the plan unless its
            // policy says otherwise (rule 4).
            continue;
        }

        for device_index in matched.iter() {
            claimed[*device_index] = true;
        }
        let _ = result.starts.push(PlannedStart {
            entry_index: index,
            devices: matched,
            // rule 5: only a driver that matched something can calibrate.
            calibrate: true,
            reason: StartReason::DeviceMatched,
        });
    }

    // ── rule 4: unconditional and critical drivers ────────────────────────
    for (index, entry) in manifest.iter().enumerate() {
        if entry.policy != LoadPolicy::Always && !entry.critical {
            continue;
        }
        // Already started because something matched it: the match is the
        // stronger reason, so leave the entry alone.
        if result.starts.iter().any(|start| start.entry_index == index) {
            continue;
        }
        let _ = result.starts.push(PlannedStart {
            entry_index: index,
            devices: Vec::new(),
            // A driver started without a device has nothing to calibrate.
            calibrate: false,
            reason: if entry.critical {
                StartReason::Critical
            } else {
                StartReason::PolicyAlways
            },
        });
    }

    // ── rule 3: report what stays unbound, for the boot log ───────────────
    for (device_index, taken) in claimed.iter().enumerate() {
        if !taken {
            let _ = result.unbound.push(device_index);
        }
    }

    result
}
