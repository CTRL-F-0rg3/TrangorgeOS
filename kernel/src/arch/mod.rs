//! Architecture abstraction layer for the kernel bootstrap.
//!
//! Exactly **one** architecture backend is compiled per build, selected by the
//! target architecture (`x86_64` or `riscv64`). It owns the CPU entry point
//! (`_start` / the `bootloader`-produced entry), [`arch::init`], the idle
//! `hlt` loop and the per-target [`#[panic_handler]`].
//!
//! The rest of the kernel talks to this layer through `crate::arch`, so the
//! architecture-dependent parts (boot, CPU init, interrupts, console) stay
//! isolated while the disk, network, driverpace and user-facing code remains
//! shared.

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "riscv64")]
pub mod riscv64;

#[cfg(target_arch = "x86_64")]
pub use x86_64 as imp;

#[cfg(target_arch = "riscv64")]
pub use riscv64 as imp;

/// Architecture-specific early-kernel initialization (CPU, serial, interrupts…).
pub fn init() {
    imp::init();
}

/// Park the CPU in the machine's idle state (`hlt` on x86_64, `wfi` on RISC-V).
pub fn hlt_loop() -> ! {
    imp::hlt_loop()
}