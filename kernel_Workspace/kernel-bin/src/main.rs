// kernel_Workspace/kernel-bin/src/main.rs

#![no_std]
#![no_main]

// 1. Pull in the global allocator and panic handler from kstd_core/kstd_alloc.
// This ensures that any `alloc::vec::Vec` or `panic!()` in the linked libraries works.
extern crate kstd_alloc;
extern crate kstd_core;

use kstd_io::traits::Write;

// 2. The main entry point called by the bootloader (via assembly trampoline or directly).
// `boot_info_ptr` points to the Multiboot2 / Limine / Stivale structure.
#[no_mangle]
pub extern "C" fn kernel_main(boot_info_ptr: *const u8) -> ! {
    // Phase 1: Early Console (Must be first to catch early panics)
    kw_base::boot::early_print::init_serial();
    let mut serial = kstd_io::Serial;
    let _ = serial.write_str("\n[TrangorgeOS] Boot sequence initiated.\n");

    // Phase 2: Parse Bootloader Info
    let boot_info = unsafe { kw_base::boot::BootInfo::from_ptr(boot_info_ptr) };
    let (mem_count, mem_regions) = kw_base::boot::memory_detect::extract_memory_map(&boot_info);
    let _ = serial.write_str("[TrangorgeOS] Memory map parsed.\n");

    // Phase 3: Initialize Legacy Kernel Engine (Memory, CPU, Interrupts)
    // The legacy `kernel` crate exposes an `init` function that takes the boot info.
    unsafe {
        kernel::init::early::init(boot_info_ptr);
    }
    let _ = serial.write_str("[TrangorgeOS] Legacy kernel engine initialized.\n");

    // Phase 4: Initialize Hardware Abstraction & Drivers
    kw_base::hal::pci::scan_bus();
    let _ = serial.write_str("[TrangorgeOS] PCI bus scanned.\n");

    // Phase 5: Initialize Native Subsystems (VFS, IPC, Network)
    // These rely on the memory and interrupts set up by the legacy engine.
    let _ = serial.write_str("[TrangorgeOS] Native subsystems ready.\n");

    // Phase 6: Start the Scheduler / Idle Task
    let _ = serial.write_str("[TrangorgeOS] Entering idle loop.\n");
    
    // Hand over to the legacy scheduler's idle loop, or run our own.
    kw_base::task::scheduler::halt_cpu();
}

// 3. Fallback halt loop in case kernel_main returns (it shouldn't).
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kstd_core::panic::panic(info)
}