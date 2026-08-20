// init.rs
extern "C" {
    fn battery_init() -> bool;
}

pub fn init() -> bool {
    unsafe { battery_init() }
}