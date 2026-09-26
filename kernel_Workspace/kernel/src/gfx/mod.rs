pub mod console;
pub mod demo;
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

// ── the `hw-sys` FFI surface ──────────────────────────────────────────────────
//
// `kernel-drivers/{gfx,hdmi,displayport}` are standalone `no_std` crates that
// reach the runtime through the thin `extern "C"` block in `hw-sys`. Those
// symbols had no definitions on this side, so those libraries could not link;
// they are provided here, which is what makes the exported driver libraries
// usable rather than merely buildable.

/// The framebuffer, in the `hw-sys` calling convention.
///
/// # Why this is not `console::fb_info`
///
/// `hw-sys` declares the stride in *bytes* and asks for a *physical* address,
/// and `console::fb_info` returns pixels and a mapped pointer. Two different
/// answers to the same question is the drift `FramebufferDesc` exists to
/// remove, so this answers in the descriptor's units.
///
/// # Why it reports any format
///
/// The old path returned false for anything that was not `Rgb888`, which meant
/// that on a `Mode13h` console it reported "no framebuffer" for a framebuffer
/// that plainly existed. A driver asking what is on screen deserves the truth
/// about the console frame too, especially the format.
#[no_mangle]
pub extern "C" fn fb_info(
    out_w: *mut u32,
    out_h: *mut u32,
    out_stride: *mut u32,
    out_phys: *mut u64,
) -> bool {
    if out_w.is_null() || out_h.is_null() || out_stride.is_null() || out_phys.is_null() {
        return false;
    }

    let desc = console::fb_desc();
    if !desc.is_usable() {
        return false;
    }

    // SAFETY: all four pointers were checked non-null above, and each is
    // written exactly once.
    unsafe {
        *out_w = desc.width;
        *out_h = desc.height;
        *out_stride = desc.stride_px;
        *out_phys = desc.phys;
    }
    true
}

/// Map `len` bytes of device memory and return the virtual address.
///
/// The mapping is device memory: a driver that read a register back
/// immediately after writing it would otherwise read its own write sitting in
/// a cache, and MMIO to a display engine is precisely the case where the
/// device never signals that the write landed.
#[no_mangle]
pub extern "C" fn mmio_map(phys: u64, len: usize, out_virt: *mut u64) -> bool {
    if out_virt.is_null() || len == 0 {
        return false;
    }
    // Mapping address zero maps whatever physical page is there, which is never
    // what a caller meant.
    if phys == 0 {
        return false;
    }

    let mut virt: u64 = 0;
    if !unsafe { crate::mm::ffi::vmm_map_device(phys, len, &mut virt as *mut u64) } {
        return false;
    }

    // SAFETY: `out_virt` was checked non-null above and written once.
    unsafe { *out_virt = virt };
    true
}

/// Undo a mapping made by [`mmio_map`].
///
/// The length must match the one the mapping was made with: a shorter unmap
/// leaves the tail mapped and still writable, and a longer one tears down pages
/// that belong to something else.
#[no_mangle]
pub extern "C" fn mmio_unmap(virt: u64, len: usize) {
    if virt == 0 || len == 0 {
        return;
    }
    unsafe {
        crate::mm::ffi::vmm_unmap_device(virt, len);
    }
}

