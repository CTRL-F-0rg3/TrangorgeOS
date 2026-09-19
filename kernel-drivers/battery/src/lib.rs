#![no_std]

//! `battery` — battery management bridge, extracted from `kernel/src/battery`.

extern "C" {
    fn battery_init() -> bool;
}

pub fn init() -> bool {
    unsafe { battery_init() }
}
