#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use bootloader::{BootInfo, entry_point};

mod gdt;
mod gfx;
mod interrupts;
mod mm;
mod nic;
mod pci;
mod serial;
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
        module: "nic::ethernet",
        func: nic::ethernet::self_test,
    },
    Test {
        module: "nic::packet",
        func: nic::packet::self_test,
    },
    Test {
        module: "nic::virtio::queue",
        func: nic::virtio::queue::self_test,
    },
    Test {
        module: "pci",
        func: pci::self_test,
    },
    Test {
        module: "mm::physical",
        func: mm::phys::self_test,
    },
    Test {
        module: "mm::heap_api",
        func: mm::api::self_test,
    },
    Test {
        module: "mm::vmm",
        func: mm::virt::self_test,
    },
    Test {
        module: "mm::address_space",
        func: mm::space::self_test,
    },
    Test {
        module: "mm::allocator",
        func: mm::self_test,
    },
    Test {
        module: "gfx",
        func: gfx::self_test,
    },
];

pub fn init() {
    serial::init();
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

    if mm::init_from_boot_info(boot_info) {
        println!("[mm] allocator initialized OK");
    } else {
        println!("[mm] allocator initialization FAILED");
    }

    if gfx::init() {
        println!("[gfx] framebuffer initialized OK");
    } else {
        println!("[gfx] framebuffer initialization FAILED");
    }

    pci::init();
    testing::run_all(TESTS);
    println!("Welcome in my Galaxy{}", "!");

    gfx::refresh();

    hlt_loop();
}