#![no_std]
#![no_main]

use trangorgelibc as tr;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    tr::log("tr-init: hello from ring 3");
    tr::log("tr-init: session manager online");

    loop {
        // tu później: tr::spawn("/bin/shell.elf") itd.
        tr::yield_cpu();
    }
}

#[panic_handler]
fn panic(_i: &core::panic::PanicInfo) -> ! {
    loop {}
}