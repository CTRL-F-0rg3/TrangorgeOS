#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use driverspacelib as ds;

mod drivers;

static mut STORAGE: drivers::storage::StorageDrv = drivers::storage::StorageDrv::new();
static mut REGISTERED: bool = false;

#[cfg(not(test))]
#[link_section = ".text.ds_entry"]
#[no_mangle]
pub extern "C" fn ds_entry(params_va: u64) {
    ds::init_once(params_va);

    unsafe {
        if !REGISTERED {
            // Wskaznik z `addr_of_mut!` zamiast `&mut STORAGE`: tworzenie
            // referencji do `static mut` jest odradzane (lint `static_mut_refs`).
            ds::register(&mut *core::ptr::addr_of_mut!(STORAGE));
            REGISTERED = true;
        }
    }

    ds::tick();
    ds::yield_to_kernel();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}