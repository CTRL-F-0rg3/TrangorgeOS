use bootloader::{entry_point, BootInfo};
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

#[cfg(all(target_arch = "x86_64", not(test)))]
entry_point!(x86_boot);

#[cfg(all(target_arch = "x86_64", not(test)))]
fn x86_boot(boot_info: &'static BootInfo) -> ! {
    crate::kernel_main(boot_info)
}