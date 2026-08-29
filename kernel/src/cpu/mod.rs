//! Portable CPU subsystem: architecture bring-up + the shared scheduler.
//!
//! * x86_64 — `smp` (AP bring-up over LAPIC/ACPI/trampoline), `acpi`,
//!   `lapic`, `trampoline`; `init`/`poweroff`/`reboot`/`self_test` come from
//!   the SMP module.
//! * riscv64 — `riscv` (SBI system reset, hart id); SMP bring-up via SBI HSM
//!   is a future milestone. The `shelduler` module is compiled on both.

#[cfg(target_arch = "x86_64")]
pub mod acpi;
#[cfg(target_arch = "x86_64")]
pub mod lapic;
#[cfg(target_arch = "riscv64")]
pub mod riscv;
#[cfg(target_arch = "x86_64")]
mod smp;
#[cfg(target_arch = "x86_64")]
pub mod trampoline;

pub mod shelduler;

#[cfg(target_arch = "x86_64")]
pub use smp::{init, poweroff, reboot, self_test};

#[cfg(target_arch = "riscv64")]
pub fn init_riscv() {
    riscv::init();
}

/// Number of CPUs visible to the kernel.
#[cfg(target_arch = "x86_64")]
pub fn total_cpus() -> u32 {
    smp::total_cpus()
}

#[cfg(target_arch = "riscv64")]
pub fn total_cpus() -> u32 {
    1
}
