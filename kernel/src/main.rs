#![no_std]
#![no_main]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]

// The allocator is provided by the x86_64 MM subsystem, and by the small
// bump allocator in `arch::riscv64` when building for RISC-V.
extern crate alloc;

// Architecture abstraction layer (bootstrap / panic / idle loop).
pub mod arch;

/// x86_64-only hardware subsystems (PCI/IDT/PIC, VGA, bootloader BitInfo,
/// driverspace, FS, NIC, gfx, terminal). These depend on x86_64 facilities
/// and are gated out on RISC-V. `mm` and `cpu` are architecture-portable
/// and live in this section too — they pick their backend by target_arch.
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

// ---- architecture-portable modules ---------------------------------------
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

// Kernel-wide init: delegates to the active architecture backend.
pub fn init() {
    arch::init();
}

// Idle the CPU (hlt/wfi) for the active architecture.
pub fn hlt_loop() -> ! {
    arch::hlt_loop()
}

/// Złączony system uprawnień: capabilities (`caps/` — co wolno) + polityka
/// (`policy/` — czy przepuszcza). Instalowany raz na obu architekturach.
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

/// x86_64 kernel main. Booted through `arch::x86_64` (bootloader `BitInfo`).
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

/// RISC-V kernel main (skeleton bootstrap for now).
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
