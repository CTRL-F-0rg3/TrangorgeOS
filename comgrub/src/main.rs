// comgrub/src/main.rs
#![no_std]
#![no_main]

use core::panic::PanicInfo;


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[no_mangle]
pub extern "C" fn _rust_entry(magic: u32, info: *const u8) -> ! {
    
    kernel::kernel_main(magic, info)
}