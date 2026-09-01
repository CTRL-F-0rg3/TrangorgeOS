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


#[cfg(target_arch = "x86_64")]
pub fn total_cpus() -> u32 {
    smp::total_cpus()
}

#[cfg(target_arch = "riscv64")]
pub fn total_cpus() -> u32 {
    1
}
