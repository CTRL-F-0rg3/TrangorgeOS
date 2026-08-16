pub mod console;
pub mod font;
pub mod framebuffer;
pub mod galaxy;

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