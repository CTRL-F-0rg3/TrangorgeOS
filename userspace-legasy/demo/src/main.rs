#![no_std]
#![no_main]

use trangorgelibc as tr;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    tr::log("demo: hello, wysylam IPC do init (pid 1)");

    tr::ipc_send(1, 0xDEAD, 0xBEEF);

    for _ in 0..10 {
        tr::yield_cpu();
    }

    tr::log("demo: exit 0");
    tr::exit(0);
}

#[panic_handler]
fn panic(_i: &core::panic::PanicInfo) -> ! { loop {} }