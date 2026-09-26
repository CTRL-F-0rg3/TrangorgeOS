//! The power device-class framework (`ds-fw-power`).
//!
//! A sibling of `ds-fw-audio` / `ds-fw-video`: it holds only the **device-class
//! contract and the wire vocabulary**, never hardware access. The hardware lives
//! in `drivers/Power-TrangorgeOS` and the management logic in `crates/ds-manager`.
//!
//! ## Power is the odd one out
//!
//! Every other device class in this workspace has the same shape: enumerate,
//! negotiate, move data. Power does not, and pretending otherwise is how battery
//! drivers end up wrong in ways nothing else here is.
//!
//! * **It is read-mostly, and the read must not be gated.** A user who cannot
//!   see the battery level cannot use the machine, so enumerating sources and
//!   reading a percentage need no capability at all. Only *changing* how the
//!   machine charges is a capability (`CapId::POWER_POLICY`).
//! * **One wrong number is worse than no number.** A gauge that jumps 20% in a
//!   sample destroys trust in the whole indicator, and a "8 hours remaining"
//!   shown on a full laptop is worse than nothing. [`smooth`] is the answer, and
//!   it is the substantive part of this crate.
//! * **The existing vocabulary is not ours.** `kernel/src/interfejs/trangorge.h`
//!   already defines `BATT_STATE_DISCHARGING = 0`, `CHARGING = 1`, `FULL = 2` and
//!   a status of `{present, state, percent, voltage_mv, temp_c, health}`. Those
//!   numbers are reproduced exactly, so a port does not have to choose between
//!   the C header and this framework.
//!
//! ## Why the back ends are behind one trait
//!
//! A laptop's battery is reached three different ways, and which one exists
//! varies by machine: ACPI (`_BIX` / `_BST`, the x86 way), a Smart Battery over
//! SMBus (the older x86 way, and what most ARM boards look like), and a
//! proprietary fuel gauge (MAX17048, BQ27xxx - the phone way). None of them is
//! mandatory.
//!
//! [`traits::PowerBackend`] is the seam. The framework never asks *which*; a
//! driver registers whichever it found, and a machine with none still compiles,
//! loads, and reports "no battery" rather than failing.

#![no_std]

pub mod smooth;
pub mod traits;
pub mod types;

pub use kapi_abi::payloads::power::{
    Health, PowerInfo, PowerPolicy, PowerState, PowerStatus, kind,
};
pub use smooth::{Smoother, DEFAULT_LOW_PERCENT};
pub use traits::PowerBackend;
pub use types::SourceId;
