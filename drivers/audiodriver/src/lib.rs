#![no_std]

pub mod jacklib;
pub mod tone;

extern "C" {
    fn ad_init(nam_va: u64, bm_va: u64) -> i32;
    fn ad_play(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> i32;
    fn ad_capture(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> i32;
    fn ad_stop() -> i32;
    fn ad_jack_present() -> i32;
    fn ad_set_amp(on: i32) -> i32;
    fn ad_position() -> u32;
}

pub fn init(nam_va: u64, bm_va: u64) -> bool {
    unsafe { ad_init(nam_va, bm_va) == 0 }
}

pub fn play(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> bool {
    unsafe { ad_play(data_phys, len, bdl_va, bdl_phys) == 0 }
}

pub fn capture(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> bool {
    unsafe { ad_capture(data_phys, len, bdl_va, bdl_phys) == 0 }
}

pub fn stop() {
    unsafe { let _ = ad_stop(); }
}

pub fn jack_present() -> bool {
    unsafe { ad_jack_present() != 0 }
}

pub fn set_amp(on: bool) {
    unsafe { let _ = ad_set_amp(on as i32); }
}

pub fn position() -> u32 {
    unsafe { ad_position() }
}