//! `ds-manager` - the driver-space supervisor.
//!
//! # What it is for
//!
//! A driver is code that runs close to the hardware. The question this crate
//! answers is: *which drivers run, and what is each one allowed to touch?*
//!
//! It does that in four steps, in this order, and the order is the point:
//!
//! ```text
//!   1. detect    ds-detect scans PCI, matches the manifest, plans, calibrates
//!   2. grant     what the driver may do, from the manifest and the boot policy
//!   3. lease     what it may touch, from the calibration
//!   4. spawn     start it, with both
//! ```
//!
//! Capabilities and resources are settled *before* the driver exists as a
//! running thing. A driver that started first could touch hardware before the
//! manager had decided it was allowed to.
//!
//! # Why the enforcement is here and not in the driver
//!
//! Because a driver is not the thing being trusted - it is the thing being
//! constrained. [`Manager::permits`] is the only place a request is judged, and
//! it judges against a table built from the manifest rather than from anything
//! the driver says about itself. A driver that ignores the answer is a driver
//! that cannot do damage, because the resources it would need are leased to
//! somebody else.
//!
//! # Why the platform is a trait
//!
//! Reading PCI configuration space and starting a process are both privileged,
//! both kernel-side, and neither is available in a test. [`Platform`] is the
//! seam: the real implementation talks to the kernel, the tests supply a fake
//! that describes a machine. The rules in this crate - which driver starts,
//! with what, touching what - are then testable on any machine, including one
//! with no driverspace at all.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod manager;
pub mod resources;
pub mod table;

pub use manager::{Manager, Platform, SpawnError, Started};
pub use resources::{Lease, Region, ResourceTable};
pub use table::{CapTable, Grant};
