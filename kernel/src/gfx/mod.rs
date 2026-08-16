pub mod console;
pub mod font;
pub mod framebuffer;
pub mod galaxy;

pub const FB_PHYS: u64 = 0xA0000;
pub const FB_WIDTH: u32 = 320;
pub const FB_HEIGHT: u32 = 200;
pub const FB_STRIDE: u32 = 320;

pub fn init() -> bool {
    console::init(FB_PHYS, FB_WIDTH, FB_HEIGHT, FB_STRIDE)
}

pub fn init_mode(fb_phys: u64, width: u32, height: u32, stride: u32) -> bool {
    console::init(fb_phys, width, height, stride)
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
    console::init(fb_phys, width, height, stride)
}