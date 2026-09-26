//! The capability table: who may do what, and how that is decided.
//!
//! # Why this is the manager's most important file
//!
//! A driver is not trusted because it is well written. It is trusted because
//! the table says so, and the table is built here from three inputs:
//!
//! 1. what the manifest entry claims the driver needs,
//! 2. what calibration found the device actually has,
//! 3. what the manager is willing to hand out at boot.
//!
//! # The three failure modes this prevents
//!
//! * **Over-grant.** A driver that reads its own capability bits and assumes
//!   they were earned. It does not matter what a driver believes: the check is
//!   here, and a request without the bit is refused before the driver is
//!   reached.
//! * **Under-grant.** A driver started with `NONE` because the manifest forgot
//!   to list a bit, then refusing a legitimate call. The `missing` report names
//!   exactly which bit, so this is diagnosable from a log rather than a hang.
//! * **Lending.** One driver's handle used by another. Entries are keyed by
//!   driver identity, never by pid or by a name a client can choose, so a
//!   client cannot ask for a grant that belongs to someone else.

use ds_detect::manifest::ManifestEntry;
use kapi_abi::CapId;

/// One driver's entry in the table.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Grant {
    /// What the manifest entry asked for.
    pub requested: CapId,
    /// What the manager actually granted.
    ///
    /// This is `requested` minus anything the boot policy withholds, which
    /// is why it is stored separately: a client asking for more than it was
    /// granted is a different event from one asking for more than the manifest
    /// wanted.
    pub granted: CapId,
    /// What was asked for but not granted, for the boot log.
    pub withheld: CapId,
}

impl Grant {
    /// An empty grant: a driver that may do nothing.
    pub const fn none() -> Self {
        Self { requested: CapId::NONE, granted: CapId::NONE, withheld: CapId::NONE }
    }

    /// Build a grant that gives everything asked for.
    pub const fn full(requested: CapId) -> Self {
        Self { requested, granted: requested, withheld: CapId::NONE }
    }

    /// Whether a client of this driver may make a request needing `capability`.
    ///
    /// The whole enforcement point. A caller checks this; nothing else does.
    pub const fn permits(self, capability: CapId) -> bool {
        self.granted.has(capability)
    }
}

/// The table, over a fixed number of drivers.
///
/// Fixed size for the same reason the manifest is: this runs before anything
/// is loaded, and an allocation at that point is an allocation that can fail
/// at the worst possible moment.
#[derive(Clone, Debug)]
pub struct CapTable {
    entries: [Option<Grant>; Self::MAX_DRIVERS],
    count: usize,
}

impl CapTable {
    /// How many drivers one table holds.
    pub const MAX_DRIVERS: usize = ds_detect::manifest::Manifest::MAX_ENTRIES;

    /// A table with nothing granted.
    pub const fn empty() -> Self {
        Self { entries: [None; Self::MAX_DRIVERS], count: 0 }
    }

    /// Add a driver's grant, from what its manifest entry asked for.
    ///
    /// `policy` is what the boot decided may be handed out. A requested bit
    /// outside `policy` is withheld rather than refused, so the driver still
    /// starts and can report what it is missing.
    pub fn insert(&mut self, requested: CapId, policy: CapId) -> bool {
        if self.count >= Self::MAX_DRIVERS {
            return false;
        }
        let granted = requested.intersection(policy);
        self.entries[self.count] = Some(Grant {
            requested,
            granted,
            withheld: requested.difference(granted),
        });
        self.count += 1;
        true
    }

    /// A driver's grant, by index.
    pub fn get(&self, index: usize) -> Option<Grant> {
        self.entries.get(index).copied().flatten()
    }

    /// How many drivers have a grant.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether nothing has been granted.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// The grant a driver should be started with, from its manifest entry.
    ///
    /// `privileged` is what the boot may hand out. A requested bit outside it
    /// is withheld rather than refused, so the driver still starts and can
    /// report what it is missing - see [`Grant::withheld`].
    pub fn grant_for(entry: &ManifestEntry, privileged: CapId) -> Grant {
        let granted = entry.capabilities.intersection(privileged);
        Grant {
            requested: entry.capabilities,
            granted,
            withheld: entry.capabilities.difference(granted),
        }
    }
}
