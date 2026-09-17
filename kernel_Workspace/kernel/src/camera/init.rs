extern "C" {
    fn camera_init() -> bool;
}

pub fn init() -> bool {
    unsafe { camera_init() }
}
