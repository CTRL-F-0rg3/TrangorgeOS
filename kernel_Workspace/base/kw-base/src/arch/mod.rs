pub mod interrupts;
pub mod registers;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "riscv64")]
pub mod riscv64;

pub use interrupts::{disable, enable, halt, are_enabled, InterruptGuard};