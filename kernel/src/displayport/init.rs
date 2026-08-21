extern "C" {
    fn dp_init() -> bool;
    fn dp_ready() -> bool;
}

pub fn init() -> bool {
    unsafe { dp_init() }
}

pub fn ready() -> bool {
    unsafe { dp_ready() }
}
