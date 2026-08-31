#![no_std]
#![no_main]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]

extern crate alloc;

pub mod arch;

#[cfg(target_arch = "x86_64")]
mod bluetooth;
mod cpu;
#[cfg(target_arch = "x86_64")]
mod drivers;
#[cfg(target_arch = "x86_64")]
mod driverspaceinit;
#[cfg(target_arch = "x86_64")]
mod fs;
#[cfg(target_arch = "x86_64")]
mod gdt;
#[cfg(target_arch = "x86_64")]
mod gfx;
#[cfg(target_arch = "x86_64")]
mod hdmi;
#[cfg(target_arch = "x86_64")]
mod interrupts;
#[cfg(target_arch = "x86_64")]
mod kernel_glue;
mod mm;
#[cfg(target_arch = "x86_64")]
mod nic;
#[cfg(target_arch = "x86_64")]
mod pci;
#[cfg(target_arch = "x86_64")]
mod terminal;

mod caps;
mod policy;
mod serial;
mod testing;
mod vga_buffer;

use testing::Test;

#[cfg(target_arch = "x86_64")]
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
        module: "fs",
        func: fs::self_test,
    },
    Test {
        module: "drivers::usb",
        func: drivers::usb::self_test,
    },
    Test {
        module: "cpu",
        func: cpu::self_test,
    },
    Test {
        module: "gfx",
        func: gfx::self_test,
    },
    Test {
        module: "caps",
        func: caps::self_test,
    },
    Test {
        module: "shelduler",
        func: cpu::shelduler::self_test,
    },
];

#[cfg(not(target_arch = "x86_64"))]
static TESTS: &[Test] = &[
    Test {
        module: "mm",
        func: mm::self_test,
    },
    Test {
        module: "shelduler",
        func: cpu::shelduler::self_test,
    },
        Test {
        module: "caps",
        func: caps::self_test,
    },
];

pub fn init() {
    arch::init();
}

pub fn hlt_loop() -> ! {
    arch::hlt_loop()
}

pub fn init_permissions() {
    match caps::init() {
        Ok(()) => println!(
            "[caps] capability store initialized OK (kernel world {})",
            caps::check::kernel_world_id()
        ),
        Err(e) => println!("[caps] initialization FAILED: {}", e),
    }
    policy::install();
    println!("[policy] unified permission engine installed (hook + audit)");
}

#[cfg(target_arch = "x86_64")]
pub fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {
    init();

    if mm::init_from_boot_info(boot_info) {
        println!("[mm] allocator initialized OK");
    } else {
        println!("[mm] allocator initialization FAILED");
    }

    init_permissions();

    if gfx::init() {
        println!("[gfx] framebuffer initialized OK");
        if hdmi::init::init() {
            println!("[hdmi] framebuffer bridge initialized OK");
        } else {
            println!("[hdmi] framebuffer bridge initialization FAILED");
        }
    } else {
        println!("[gfx] framebuffer initialization FAILED");
    }

    pci::init();

    match nic::runtime::init() {
        Ok(()) => println!("[nic] virtio-net initialized OK"),
        Err(error) => println!("[nic] virtio-net initialization FAILED: {:?}", error),
    }

    if bluetooth::init::init() {
        println!("[bluetooth] initialized OK");
    } else {
        println!("[bluetooth] initialization FAILED");
    }

    fs::init();
    cpu::init(boot_info);
    testing::run_all(TESTS);
    println!("Welcome in my Galaxy!");

    gfx::refresh();

    terminal::init();
    terminal::run();
}

#[cfg(target_arch = "riscv64")]
pub fn kernel_main_riscv() -> ! {
    init();
    println!("TrangorgeOS RISC-V: boot OK (heap + serial)");

    if mm::init_riscv() {
        println!(
            "[mm] riscv backend initialized (frames + heap + vmm + Sv39 tables)"
        );
    } else {
        println!("[mm] riscv backend initialization FAILED");
    }
    cpu::init_riscv();

    init_permissions();

    println!("TrangorgeOS RISC-V 64-bit — bootstrap OK");
    println!("(bump heap + unified permissions: caps + policy)");

    testing::run_all(TESTS);

    println!("Idle. (riscv64 port under development)");
    loop {
        hlt_loop();
    }
}
