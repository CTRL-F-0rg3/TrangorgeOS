use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn x86_panic(info: &PanicInfo) -> ! {
    crate::gfx::panic_screen::show(info)
}

pub fn init() {
    crate::serial::init();
    crate::gdt::init();
    crate::interrupts::init_idt();
    unsafe { crate::interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

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

pub fn now() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

// NOTE: the bootloader entry point (`_start`) is defined by the *binary* crate
// (`kernel-bin/src/main.rs` via `bootloader::entry_point!`), which then calls
// `crate::kernel_main`. A library must not define `_start` as well, otherwise
// rustc fails with "symbol `_start` is already defined" (two `entry_point!`
// expansions in the same crate) or the linker reports a duplicate symbol when
// both the library and the binary end up in the same image.