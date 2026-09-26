//! `IntelGpu-TrangorgeOS` - the Intel GPU hardware layer.
//!
//! This is the driver side of `ds-fw-gpu`: it knows Intel silicon and speaks
//! the framework's [`GpuDevice`] contract. It is one driver for both
//! integrated and discrete parts, and the reason is worth stating plainly,
//! because the usual expectation is the opposite.
//!
//! # One driver, two memory models
//!
//! Intel's discrete parts - Arc DG1, DG2, Battlemage - are built on the same
//! Xe cores as the integrated Iris Xe of the same period. There is no
//! register-map difference and no command-encoding difference, so there is
//! nothing to justify a second driver. Mesa agrees: its `iris` driver serves
//! both from one code path.
//!
//! What genuinely differs is memory:
//!
//! * an integrated part has no memory of its own, and every buffer is system
//!   memory reached through a GTT page table the driver builds;
//! * a discrete part has local memory as well, and which of the two a buffer
//!   goes in is a decision worth getting right.
//!
//! So [`mm::MemoryModel`] is a separate axis from [`probe::GenKind`]. Generation
//! picks the register map; the memory model picks where a buffer lives. The
//! distinction is a *runtime* one, decided per buffer, not a build-time one.
//!
//! # What lives here
//!
//! * [`probe`] - PCI id to generation and memory model. Pure: it takes ids,
//!   not a live BAR, so the whole classification is testable on a machine
//!   with no Intel GPU.
//! * [`mm`] - the memory model, and the placement rule.
//!
//! # What does not live here yet
//!
//! No register map, no command encoding, no ring submission. Those follow the
//! probe and memory rules once they are agreed, and each is a large piece of
//! work on its own.

#![no_std]

extern crate alloc;

pub mod engine;
pub mod mm;
pub mod probe;
pub mod ring;

pub use ds_fw_gpu::{GpuDevice, GpuService};
pub use mm::{MemoryModel, Placement};
pub use probe::{GenKind, IntelDevice, is_supported, probe};
