pub mod console;
pub mod font;
pub mod framebuffer;
pub mod galaxy;
pub mod panic_screen;
pub mod vga;

use framebuffer::PixelFormat;

pub const FB_PHYS: u64 = 0xA0000;

static mut CURRENT: vga::VideoMode = vga::VideoMode::Mode13h;
static mut CURRENT_W: u32 = 320;
static mut CURRENT_H: u32 = 200;

pub fn init() -> bool {
    unsafe {
        CURRENT = vga::VideoMode::Mode13h;
        CURRENT_W = 320;
        CURRENT_H = 200;
    }
    console::init(FB_PHYS, 320, 200, 320, PixelFormat::Indexed8)
}

pub fn init_mode(fb_phys: u64, width: u32, height: u32, stride: u32) -> bool {
    console::init(fb_phys, width, height, stride, PixelFormat::Indexed8)
}


pub fn set_resolution(mode: vga::VideoMode) -> bool {
    vga::bochs_disable();
    vga::set_mode(mode);
    unsafe { CURRENT = mode; }

    match mode {
        vga::VideoMode::Mode13h => {
            unsafe { CURRENT_W = 320; CURRENT_H = 200; }
            console::init(FB_PHYS, 320, 200, 320, PixelFormat::Indexed8)
        }
        vga::VideoMode::Mode12h => {
            unsafe { CURRENT_W = 640; CURRENT_H = 480; }
            console::init(FB_PHYS, 640, 480, 80, PixelFormat::Planar4)
        }
    }
}

pub fn set_resolution_w_h(width: u32, height: u32) -> bool {
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return false;
    }

    let Some(lfb) = vga::bochs_lfb_base() else {
        return false;
    };

    if !vga::bochs_set_mode(width, height, 32) {
        return false;
    }

    let stride = width * 4;
    if !console::init(lfb, width, height, stride, PixelFormat::Rgb888) {
        return false;
    }

    unsafe {
        CURRENT_W = width;
        CURRENT_H = height;
    }

    true
}

pub fn current_resolution() -> (u32, u32) {
    unsafe { (CURRENT_W, CURRENT_H) }
}

pub fn refresh() {
    console::refresh();
}

pub fn self_test() -> Result<&'static str, &'static str> {
    if !init() {
        return Err("gfx: framebuffer init failed");
    }

    refresh();

    Ok("gfx framebuffer + galaxy + console overlay")
}

#[no_mangle]
pub extern "C" fn gfx_refresh() {
    console::refresh();
}

#[no_mangle]
pub extern "C" fn gfx_init(fb_phys: u64,
                            width: u32,
                            height: u32,
                            stride: u32) -> bool {
    console::init(fb_phys, width, height, stride, PixelFormat::Indexed8)
}


#[no_mangle]
pub extern "C" fn gfx_fb_info_raw(w: *mut u32,
                                  h: *mut u32,
                                  s: *mut u32,
                                  base: *mut u64,
                                  flip: *mut i32) -> i32 {
    if w.is_null() || h.is_null() || s.is_null() || base.is_null() || flip.is_null() {
        return -1;
    }

    unsafe {
        match console::framebuffer_info_pub() {
            Some((fw, fh, fstride, ptr, fliprows)) => {
                *w = fw;
                *h = fh;
                *s = fstride;
                *base = ptr;
                *flip = if fliprows { 1 } else { 0 };
                0
            }
            None => -1,
        }
    }
}
