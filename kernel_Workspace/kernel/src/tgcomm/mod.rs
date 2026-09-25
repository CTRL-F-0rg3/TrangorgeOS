//! Kernel-side shared-memory communication built on the independent `tg_comm`
//! protocol library.
//!
//! This module replaces the legacy `DsRing`/`DsMsg` transport with the strict
//! `tg_comm` shared-memory rings and its capability authorization gate, for
//! both the driver space and the user space (ring 3). The actual service
//! dispatch (`service::handle` and the device bridges) is preserved: a
//! `CommMsg` is authorized, then mapped back onto the legacy `DsMsg` shape and
//! handed to the existing handlers.

pub mod bridge;
pub mod dispatch;
pub mod driverspace;
pub mod userspace;
