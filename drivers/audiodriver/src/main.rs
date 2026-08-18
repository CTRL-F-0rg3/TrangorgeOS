#![no_std]
#![no_main]

use driverspacelib as ds;

mod jacklib;
mod tone;

static mut JACK: jacklib::JackMgr = jacklib::JackMgr::new();
static mut BUF_VA: u64 = 0;
static mut PLAYED: bool = false;

#[link_section = ".text.ds_entry"]
#[no_mangle]
pub extern "C" fn ds_entry(params_va: u64) {
    unsafe {
        ds::init_once(params_va);

        if BUF_VA == 0 {
            let id = ds::mem::alloc_pages(4);
            let _ = id;
        }
    }

    unsafe {
        JACK.tick();

        if JACK.present() && !PLAYED && BUF_VA != 0 {
            let scratch = ds::DS_SCRATCH_VA as *mut u8;
            let mut tmp = [0u8; 4096];
            tone::fill_square(&mut tmp, 40);

            core::ptr::copy_nonoverlapping(tmp.as_ptr(), scratch, 4096);

            ds::jack::play(BUF_VA, 4096 * 4);
            PLAYED = true;
        }
    }

    ds::yield_to_kernel();
}

#[panic_handler]
fn panic(_i: &core::panic::PanicInfo) -> ! {
    loop {}
}