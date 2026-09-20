// kernel_Workspace/kernel-bin/src/main.rs
//
// Thin executable wrapper around the legacy `kernel` engine.
// The kernel crate provides:
//   * `entry_point!(x86_boot)`  -> `_start` (bootloader 0.9)
//   * `#[panic_handler]`        -> gfx::panic_screen
//   * `#[global_allocator]`     -> mm::KernelAlloc
// So this binary only needs to link the library in.

#![no_std]
#![no_main]

extern crate kernel;
