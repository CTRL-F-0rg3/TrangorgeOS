// kernel_Workspace/kernel-bin/src/lib.rs
//
// TrangorgeOS "new world" layer: DriverSpace / IPC glue that the legacy
// `kernel` crate hands control to (`kernel::kernel_main` -> `extern "C"
// kernel_bin_entry`). It is linked into the `kernel-bin` binary; the bootloader
// entry point (`_start`) stays in `src/main.rs`.

#![no_std]

use kstd_io::traits::Write;

/// Serial banner emitted right before the legacy kernel takes over.
pub fn announce_boot() {
    let mut serial = kstd_io::Serial;
    serial
        .write_str("\n[Kernel-Bin] starting: handing over to the legacy kernel\n")
        .ok();
}

/// Handoff target for the legacy kernel.
///
/// `kernel::kernel_main` ends with `extern "C" { fn kernel_bin_entry(*const u8) -> ! }`,
/// so this symbol has to exist in the bootimage. The bootloader `BootInfo`
/// pointer is not used yet: the new layer currently only owns the idle loop.
///
/// # Safety
/// Called from the kernel, so `_boot_info_ptr` must be the pointer the
/// bootloader passed in `rdi` (never dereferenced here).
#[no_mangle]
pub unsafe extern "C" fn kernel_bin_entry(_boot_info_ptr: *const u8) -> ! {
    announce_boot();
    kernel_ipc_loop()
}

/// Idle loop of the new entry layer (IPC rings are driven from interrupts).
pub fn kernel_ipc_loop() -> ! {
    #[cfg(target_arch = "x86_64")]
    loop {
        unsafe { core::arch::asm!("hlt") }
    }

    #[cfg(not(target_arch = "x86_64"))]
    loop {
        core::hint::spin_loop();
    }
}
