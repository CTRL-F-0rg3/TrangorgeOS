#![no_std]
#![no_main]

use trangorgelibc as tr;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    tr::log("tr-init: session manager (pid self)");

    tr::spawn("/bin/shell.elf");
    tr::spawn("/bin/demo.elf");

    loop {
        if let Some(m) = tr::ipc_recv() {
            tr::log("tr-init: ipc from");
            tr::put_u32(m.from);
        }

        tr::yield_cpu();
    }
}

#[panic_handler]
fn panic(_i: &core::panic::PanicInfo) -> ! { loop {} }