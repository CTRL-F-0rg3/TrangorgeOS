extern "C" {
    fn hdmi_init_with(fb_phys: u64, w: u32, h: u32, stride: u32) -> bool;
    fn hdmi_ready() -> bool;
}

pub fn init() -> bool {
    let (w, h, stride, phys) = crate::gfx::console::fb_info();
    unsafe { hdmi_init_with(phys, w, h, stride) }
}

pub fn ready() -> bool {
    unsafe { hdmi_ready() }
}
