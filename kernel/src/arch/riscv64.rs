//! RISC-V (riscv64gc) bootstrap backend.
//!
//! *Skeleton / scaffold stage*: this is the minimal bootable RISC-V path that
//! makes the kernel *compilable* for `riscv64gc-unknown-none-elf`. It brings
//! up the serial console (NS16550A UART over MMIO, which the QEMU `virt`
//! machine maps at 0x10000000), prints a banner through the shared
//! `println!` pipeline and idles with `wfi`.
//!
//! Missing (tracked for the full port): CLINT timer/source, Sv39 paging,
//! device-tree parsing, PLIC interrupt handling and the full kernel bootstrap.

use core::panic::PanicInfo;

/// RISC-V panic handler: dump the message over the serial console and idle.
#[panic_handler]
fn riscv_panic(info: &PanicInfo) -> ! {
    crate::serial::write_str("\n[PANIC] ");
    crate::serial::print_args(format_args!("{}\n", info));
    crate::hlt_loop()
}

/// Early RISC-V initialization.
pub fn init() {
    crate::serial::init();
}

/// Idle CPU (`wfi` — WAIT FOR interrupt).
pub fn hlt_loop() -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

/// Bare-metal CPU entry point. OpenSBI transfers control here after loading
/// the kernel at the address in `riscv64-link.ld` (0x8020_0000 on `virt`).
#[no_mangle]
pub extern "C" fn _start() -> ! {
    crate::kernel_main_riscv()
}