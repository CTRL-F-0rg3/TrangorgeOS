pub mod console;
pub mod font;
pub mod framebuffer;
pub mod galaxy;
pub mod vga;

use framebuffer::PixelFormat;

pub const FB_PHYS: u64 = 0xA0000;

/// Currently active video mode.
static mut CURRENT: vga::VideoMode = vga::VideoMode::Mode13h;

pub fn init() -> bool {
    console::init(FB_PHYS, 320, 200, 320, PixelFormat::Indexed8)
}

pub fn init_mode(fb_phys: u64, width: u32, height: u32, stride: u32) -> bool {
    console::init(fb_phys, width, height, stride, PixelFormat::Indexed8)
}

/// Switches the video mode (direct VGA register programming) and re-initializes
/// the console. Returns true on success.
pub fn set_resolution(mode: vga::VideoMode) -> bool {
    vga::set_mode(mode);
    unsafe { CURRENT = mode; }

    match mode {
        vga::VideoMode::Mode13h => console::init(FB_PHYS, 320, 200, 320, PixelFormat::Indexed8),
        vga::VideoMode::Mode12h => console::init(FB_PHYS, 640, 480, 80, PixelFormat::Planar4),
    }
}

/// Human-readable name of the active resolution.
pub fn current_resolution() -> &'static str {
    match unsafe { CURRENT } {
        vga::VideoMode::Mode13h => "320x200",
        vga::VideoMode::Mode12h => "640x480",
    }
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
