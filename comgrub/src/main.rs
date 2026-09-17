
#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Wejście jądra. Wywołuje je `_rust_entry` z konwencją multiboot2:
// `magic` (EAX) i `info` (EBX). Symbol dostarcza obraz jądra podczas linkowania
// — dlatego jądro musi eksportować shim zgodny z multiboot2 o tej nazwie.
extern "C" {
    fn kernel_main(magic: u32, info: *const u8) -> !;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) }
    }
}

/// Punkt wejścia Rust wywoływany z `boot.asm` (`call _rust_entry`).
#[unsafe(no_mangle)]
pub extern "C" fn _rust_entry(magic: u32, info: *const u8) -> ! {
    unsafe { kernel_main(magic, info) }
}