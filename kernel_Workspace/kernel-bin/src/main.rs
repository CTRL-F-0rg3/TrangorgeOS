#![no_std]
#![no_main]

extern crate kstd_alloc;
extern crate kstd_core;

use bootloader::{entry_point, bootinfo::BootInfo};
use kstd_io::traits::Write;
use kstd_base::Status;
use kapi_abi::DsMsg;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    let mut serial = kstd_io::Serial;
    
    serial.write_str("\n[Kernel-Bin] Starting...\n").ok();
    
    // Kernel init nie przyjmuje argumentów - bootloader przekazuje BootInfo przez entry_point
    unsafe { kernel::init(); }
    serial.write_str("[Kernel-Bin] Legacy kernel initialized\n").ok();
    
    // DriverSpace nie jest jeszcze zintegrowany jako pojedynczy crate
    // Na razie tylko testujemy IPC ring
    serial.write_str("[Kernel-Bin] Testing IPC ring...\n").ok();
    
    let msg = DsMsg {
        cmd: 0xFF,
        status: 0,
        arg0: 0x12345678,
        arg1: 0xABCDEF00,
        arg2: 0,
        arg3: 0,
    };
    
    serial.write_str("[Kernel-Bin] IPC test message created\n").ok();
    
    // W przyszłości tutaj będzie inicjalizacja DriverSpace
    // Na razie wchodzimy w pętlę IPC
    serial.write_str("[Kernel-Bin] Entering IPC loop...\n").ok();
    kernel_ipc_loop(&mut serial);
}

fn kernel_ipc_loop(serial: &mut kstd_io::Serial) -> ! {
    loop {
        // Na razie tylko halt - w przyszłości będzie odbiór wiadomości z DriverSpace
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kstd_core::panic::panic(info)
}