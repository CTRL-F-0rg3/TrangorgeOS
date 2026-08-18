#![no_std]
#![no_main]

use driverspacelib as ds;
use ds::driver::{Driver, DeviceInfo};

mod drivers;

static mut STORAGE: drivers::storage::StorageDrv = drivers::storage::StorageDrv::new();
static mut REGISTERED: bool = false;

#[link_section = ".text.ds_entry"]
#[no_mangle]
pub extern "C" fn ds_entry(params_va: u64) {
    unsafe {
        ds::init_once(params_va);

        if !REGISTERED {
            ds::register(&mut STORAGE);
            REGISTERED = true;
        }
    }

    ds::tick();
    ds::yield_to_kernel();
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}