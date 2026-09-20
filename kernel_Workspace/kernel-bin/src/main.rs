// kernel_Workspace/kernel-bin/src/main.rs
//
// The bootable binary of TrangorgeOS. `cargo bootimage` wraps it into
// `target/x86_64-kernel/debug/bootimage-kernel-bin.bin`.
//
// This crate owns the bootloader 0.9 entry point (`_start`): it is the only
// place in the workspace allowed to define it. The fully featured kernel lives
// in the `kernel` library crate, the new IPC/DriverSpace layer in `src/lib.rs`.

#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // New layer: banner + plumbing that is linked into the image before the
    // legacy kernel takes over.
    kernel_bin::announce_boot();

    // Legacy kernel: full boot (arch, mm, gfx, drivers, fs, tests, terminal).
    // It never returns (`kernel::terminal::run() -> !`).
    kernel::kernel_main(boot_info)
}
