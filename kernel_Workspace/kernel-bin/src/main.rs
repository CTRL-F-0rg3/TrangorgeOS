// kernel_Workspace/kernel-bin/src/main.rs

#![no_std]
#![no_main]

extern crate kstd_alloc;
extern crate kstd_core;

use bootloader::{entry_point, bootinfo::BootInfo};
use kstd_io::traits::Write;
use kapi_abi::DsMsg;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    let mut serial = kstd_io::Serial;
    
    serial.write_str("\n[Kernel-Bin] Starting...\n").ok();
    
    unsafe { kernel::init(); }
    serial.write_str("[Kernel-Bin] Legacy kernel initialized\n").ok();
    
    serial.write_str("[Kernel-Bin] Testing IPC ring...\n").ok();
    
    // Poprawna inicjalizacja DsMsg (bez arg3)
    let msg = DsMsg {
        id: 0,
        cmd: 0xFF,
        flags: 0,
        arg0: 0x12345678,
        arg1: 0xABCDEF00,
        arg2: 0,
        status: 0,
        pad: 0,
    };
    
    serial.write_str("[Kernel-Bin] IPC test message created\n").ok();
    
    serial.write_str("[Kernel-Bin] Entering IPC loop...\n").ok();
    kernel_ipc_loop(&mut serial);
}

// Dodano '_' przed serial, żeby uciszyć warning o nieużywanej zmiennej
fn kernel_ipc_loop(_serial: &mut kstd_io::Serial) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

// #[panic_handler]
// fn panic(info: &core::panic::PanicInfo) -> ! {
//     kstd_core::panic::panic(info)
// }