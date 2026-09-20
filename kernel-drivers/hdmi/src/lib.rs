#![no_std]

//! `hdmi` — HDMI framebuffer bridge, extracted from `kernel/src/hdmi`.

extern "C" {
    fn hdmi_init_with(fb_phys: u64, w: u32, h: u32, stride: u32) -> bool;
    fn hdmi_ready() -> bool;
}

pub fn init() -> bool {
    let (mut w, mut h, mut stride, mut phys) = (0u32, 0u32, 0u32, 0u64);

    unsafe {
        if !hw_sys::fb_info(&mut w, &mut h, &mut stride, &mut phys) {
            return false;
        }

        hdmi_init_with(phys, w, h, stride)
    }
}

pub fn ready() -> bool {
    unsafe { hdmi_ready() }
}
