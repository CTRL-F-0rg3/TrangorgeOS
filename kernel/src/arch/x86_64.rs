//! x86_64 bootstrap backend.
//!
//! This is the original kernel entry: it is booted by the `bootloader` v0.9
//! crate (which sets up GDT, IDT and an initial page table and passes a
//! [`BootInfo`] map), then hands control to the shared [`crate::kernel_main`].

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

/// x86_64 panic handler — prints to the GFX panic screen then halts.
#[panic_handler]
fn x86_panic(info: &PanicInfo) -> ! {
    crate::gfx::panic_screen::show(info)
}

/// Early x86_64 CPU bootstrap: serial, GDT, IDT, PIC and interrupts.
pub fn init() {
    crate::serial::init();
    crate::gdt::init();
    crate::interrupts::init_idt();
    unsafe { crate::interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

/// Idle loop (halt until the next interrupt).
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Entry point produced by the `bootloader` crate; delegates to the shared
/// kernel main passing the bootloader-provided memory map.
entry_point!(x86_boot);

fn x86_boot(boot_info: &'static BootInfo) -> ! {
    crate::kernel_main(boot_info)
}