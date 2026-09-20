#![no_std]

//! `bluetooth` — Bluetooth controller bridge, extracted from
//! `kernel/src/bluetooth`.

extern "C" {
    fn bt_init() -> bool;
    fn bt_ready() -> bool;
}

pub fn init() -> bool {
    unsafe { bt_init() }
}

pub fn ready() -> bool {
    unsafe { bt_ready() }
}
