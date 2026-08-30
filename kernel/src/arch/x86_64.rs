//! x86_64 bootstrap backend.
//!
//! This is the original kernel entry: it is booted by the `bootloader` v0.9
//! crate (which sets up GDT, IDT and an initial page table and passes a
//! [`BootInfo`] map), then hands control to the shared [`crate::kernel_main`].

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

/// x86_64 panic handler — prints to the GFX panic screen then halts.
/// (`not(test)`: on the host test harness std provides the panic handler.)
#[cfg(not(test))]
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

/// Current CPU ID (read-once). Reads the initial APIC id from CPUID leaf 1,
/// EBX bits 31:24. Falls back to 0 (BSP) if the leaf is unavailable.
pub fn current_cpu() -> usize {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        use core::arch::x86_64::__cpuid;
        (__cpuid(1).ebx >> 24) as usize
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        0
    }
}

/// Monotonic kernel time: the Time Stamp Counter. Monotonic per-CPU and good
/// enough for TTL capability grants and audit timestamps on this platform.
pub fn now() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

/// Entry point produced by the `bootloader` crate; delegates to the shared
/// kernel main passing the bootloader-provided memory map.
#[cfg(all(target_arch = "x86_64", not(test)))]
entry_point!(x86_boot);

#[cfg(all(target_arch = "x86_64", not(test)))]
fn x86_boot(boot_info: &'static BootInfo) -> ! {
    crate::kernel_main(boot_info)
}