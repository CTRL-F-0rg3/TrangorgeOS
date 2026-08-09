#![no_std]
#![no_main]

mod testing;
mod vga_buffer;

use core::panic::PanicInfo;
use testing::Test;

static TESTS: &[Test] = &[Test {
    module: "vga_buffer",
    func: vga_buffer::self_test,
}];

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    testing::run_all(TESTS);
    println!("Hello World{}", "!");

    loop {}
}
