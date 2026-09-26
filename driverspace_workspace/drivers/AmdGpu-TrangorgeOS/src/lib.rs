//! `AmdGpu-TrangorgeOS` - the AMD GPU hardware layer.
//!
//! The driver side of `ds-fw-gpu`: it knows AMD silicon and speaks the
//! framework's [`GpuDevice`] contract.
//!
//! # One driver, and the split is real
//!
//! Unlike Intel - where a discrete Arc card and an integrated Iris Xe share a
//! register map and only differ in memory - AMD's integrated and discrete
//! parts are separated along three axes at once:
//!
//! * **Architecture.** GCN through RDNA 4. The generations are *not* ordered
//!   the way the names suggest: a 2023 APU is RDNA 2 while a 2019 discrete card
//!   is GCN 5, so "newer" is not a comparison. [`probe::Arch`] is an
//!   enumeration, not a scale.
//! * **Memory.** An APU shares system memory with the CPU and has none of its
//!   own; a discrete card has local memory behind a much wider interconnect.
//!   See [`mm::MemoryModel`].
//! * **Display.** An APU scans out of system memory, a discrete card out of
//!   local memory or a dedicated framebuffer.
//!
//! # Where the device data comes from
//!
//! `pci.ids` (2026-07), not Mesa's own table. Mesa's has 150 entries and stops
//! at GCN 1.2 - no Vega, no Navi, no RDNA, no APUs - so a driver built on it
//! would cover hardware that stopped shipping years ago. This crate's table
//! has 235 drivable ids across 37 families.
//!
//! # What is deliberately refused
//!
//! The Instinct MI-series accelerators and the console APUs share AMD's vendor
//! id and sit next to Radeon parts in `pci.ids`. They are left unrecognised
//! and refused: an MI250 is a datacenter part with no display engine, and
//! `pci.ids` carries no PCI class, so the name is the only signal available
//! here - and refusing is the safe reading of an ambiguous one.
//!
//! # What does not live here yet
//!
//! No register map, no command encoding, no ring submission. Those follow the
//! probe and memory rules once they are agreed.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod engine;
pub mod mm;
pub mod probe;
pub mod ring;

pub use ds_fw_gpu::{GpuDevice, GpuService};
pub use mm::{MemoryModel, Placement};
pub use probe::{AmdDevice, Arch, is_supported, probe};
