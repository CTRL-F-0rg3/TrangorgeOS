#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader::{BootInfo, entry_point};

mod gdt;
mod interrupts;
mod memory;
mod testing;
mod vga_buffer;

use core::panic::PanicInfo;
use testing::Test;

static TESTS: &[Test] = &[
    Test {
        module: "vga_buffer",
        func: vga_buffer::self_test,
    },
    Test {
        module: "gdt",
        func: gdt::self_test,
    },
    Test {
        module: "interrupts",
        func: interrupts::self_test,
    },
    Test {
        module: "memory::paging",
        func: memory::paging::self_test,
    },
];

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init();
    memory::init(boot_info.physical_memory_offset);
    testing::run_all(TESTS);
    println!("Hello World{}", "!");

    hlt_loop();
}
