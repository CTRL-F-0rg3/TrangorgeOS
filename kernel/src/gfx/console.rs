use super::font::FONT8X8;
use super::framebuffer::{Framebuffer, PixelFormat, PALETTE16, rgb};
use super::galaxy;
use crate::mm::ffi;

const GLYPH_W: usize = 8;
const GLYPH_H: usize = 8;

pub const MAX_COLS: usize = 80;
pub const MAX_ROWS: usize = 25;

static mut FB: Option<Framebuffer> = None;
static mut FB_PHYS: u64 = 0;
static mut ENABLED: bool = true;
static mut CLEAN: alloc::vec::Vec<u32> = alloc::vec::Vec::new();

static mut COLS: usize = 0;
static mut ROWS: usize = 0;

static mut CELL_CACHE: [(u8, u8); MAX_COLS * MAX_ROWS] = [(0, 0); MAX_COLS * MAX_ROWS];
static mut CACHE_VALID: bool = false;

static mut FB_DEV_VIRT: u64 = 0;
static mut FB_DEV_SIZE: usize = 0;

static mut REVERSE_TEXT_COLS: bool = true;

fn fb() -> &'static mut Framebuffer {
    unsafe { FB.as_mut().unwrap() }
}

pub fn cols() -> usize {
    unsafe { COLS }
}

pub fn rows() -> usize {
    unsafe { ROWS }
}

pub fn fb_info() -> (u32, u32, u32, u64) {
    unsafe {
        match FB.as_ref() {
            Some(framebuffer) if framebuffer.format == PixelFormat::Rgb888 && FB_PHYS != 0 => (
                framebuffer.width as u32,
                framebuffer.height as u32,
                (framebuffer.stride / 4) as u32,
                FB_PHYS,
            ),
            _ => (0, 0, 0, 0),
        }
    }
}

fn framebuffer_info() -> Option<(u32, u32, u32, u64, bool)> {
    unsafe {
        match FB.as_ref() {
            Some(fb)
                if fb.format == PixelFormat::Rgb888
                    && FB_PHYS != 0
                    && !fb.ptr.is_null() =>
            {
                Some((
                    fb.width as u32,
                    fb.height as u32,
                    (fb.stride / 4) as u32,
                    fb.ptr as u64,
                    super::framebuffer::FLIP,
                ))
            }
            _ => None,
        }
    }
}

pub(crate) fn framebuffer_info_pub() -> Option<(u32, u32, u32, u64, bool)> {
    framebuffer_info()
}

pub fn set_enabled(enabled: bool) {
    unsafe {
        ENABLED = enabled;
    }
}

#[no_mangle]
pub extern "C" fn console_set_enabled(on: i32) {
    set_enabled(on != 0);
}

fn set_palette_rgb332() {
    use x86_64::instructions::port::Port;

    let mut idx = Port::<u8>::new(0x3C8);
    let mut data = Port::<u8>::new(0x3C9);

    unsafe {
        idx.write(0);

        for i in 0..256u32 {
            let r = ((i >> 5) & 0x7) * 9;
            let g = ((i >> 2) & 0x7) * 9;
            let b = (i & 0x3) * 21;

            data.write(r as u8);
            data.write(g as u8);
            data.write(b as u8);
        }
    }
}

fn set_palette16() {
    use x86_64::instructions::port::Port;

    let mut idx = Port::<u8>::new(0x3C8);
    let mut data = Port::<u8>::new(0x3C9);

    unsafe {
        idx.write(0);
        for &(r, g, b) in PALETTE16.iter() {
            data.write(r as u8);
            data.write(g as u8);
            data.write(b as u8);
        }
    }
}

fn disable_text_cursor() {
    use x86_64::instructions::port::Port;

    let mut idx = Port::<u8>::new(0x3D4);
    let mut data = Port::<u8>::new(0x3D5);

    unsafe {
        idx.write(0x0Au8);
        data.write(0x20u8);
    }
}

fn delay() {
    for _ in 0..50_000 {
        core::hint::spin_loop();
    }
}

pub fn test_fill(r: u32, g: u32, b: u32) -> bool {
    if unsafe { FB.is_none() } {
        return false;
    }

    let (w, h) = {
        let f = fb();
        (f.width, f.height)
    };

    for y in 0..h {
        for x in 0..w {
            fb().set(x, y, rgb(r, g, b));
        }
    }

    true
}

pub fn resync_background() -> bool {
    if unsafe { FB.is_none() } {
        return false;
    }

    let (w, h) = {
        let f = fb();
        (f.width, f.height)
    };

    unsafe {
        CLEAN.clear();
        CLEAN.reserve(w * h);
        for y in 0..h {
            for x in 0..w {
                CLEAN.push(fb().get(x, y));
            }
        }
        CACHE_VALID = false;
    }

    true
}

pub fn init(fb_addr: u64, width: u32, height: u32, stride: u32, format: PixelFormat) -> bool {
    if width == 0 || height == 0 || stride == 0 {
        return false;
    }

    if !unsafe { ffi::mm_ready() } {
        return false;
    }

    let width = width as usize;
    let height = height as usize;
    let stride = stride as usize; 


    let size = ((stride * height) + 0xFFF) & !0xFFF;

    unsafe {
        FB_PHYS = if fb_addr >= 0xFFFF800000000000 { 0 } else { fb_addr };
        ENABLED = true;
    }

    let ptr = if fb_addr >= 0xFFFF800000000000 {
        fb_addr as *mut u8
    } else {
        unsafe {
            if FB_DEV_VIRT != 0 {
                ffi::vmm_unmap_device(FB_DEV_VIRT, FB_DEV_SIZE);
                FB_DEV_VIRT = 0;
                FB_DEV_SIZE = 0;
            }
        }

        let mut virt = 0u64;

        if !unsafe { ffi::vmm_map_device(fb_addr, size, &mut virt) } {
            return false;
        }

        unsafe {
            FB_DEV_VIRT = virt;
            FB_DEV_SIZE = size;
        }

        virt as *mut u8
    };

    match format {
        PixelFormat::Indexed8 => set_palette_rgb332(),
        PixelFormat::Planar4 => set_palette16(),
        PixelFormat::Rgb888 => {} 
    }
    disable_text_cursor();

    unsafe {
        super::framebuffer::FLIP = format == PixelFormat::Rgb888;
        super::framebuffer::FLIP_X = format == PixelFormat::Indexed8;

        REVERSE_TEXT_COLS = format != PixelFormat::Rgb888;
    }

    unsafe {
        FB = Some(Framebuffer {
            ptr,
            width,
            height,
            stride,
            format,
        });


        COLS = (width / GLYPH_W).clamp(1, MAX_COLS);
        ROWS = (height / GLYPH_H).clamp(1, MAX_ROWS);


        CACHE_VALID = false;
    }


    let total_px = width * height;
    let steps: u32 = if total_px > 1_000_000 {
        1
    } else if total_px > 400_000 {
        4
    } else if total_px > 150_000 {
        8
    } else {
        17
    };
    let step_size = (256 / steps.max(1)).max(1);

    if steps <= 1 {
        galaxy::render(fb(), 256);
    } else {
        let mut t = 0u32;
        loop {
            galaxy::render(fb(), t.min(256));
            delay();
            if t >= 256 {
                break;
            }
            t += step_size;
        }
    }


    unsafe {
        CLEAN.clear();
        CLEAN.reserve(width * height);
        for y in 0..height {
            for x in 0..width {
                CLEAN.push(fb().get(x, y));
            }
        }
    }

    refresh();

    true
}

pub fn refresh() {
    if unsafe { FB.is_none() || !ENABLED } {
        return;
    }


    let first = unsafe { !CACHE_VALID };

    if first {
        let (w, h) = {
            let f = fb();
            (f.width, f.height)
        };

        unsafe {
            for y in 0..h {
                for x in 0..w {
                    fb().set(x, y, CLEAN[y * w + x]);
                }
            }
        }
    }

    let (cols, rows) = unsafe { (COLS, ROWS) };

    for row in 0..rows {
        for col in 0..cols {
            let src_col = if unsafe { REVERSE_TEXT_COLS } { cols - 1 - col } else { col };
            let (ch, attr) = crate::vga_buffer::text_cell(row, src_col);
            let idx = row * MAX_COLS + col;

            let changed = unsafe { CELL_CACHE[idx] != (ch, attr) };

            if !first && !changed {
                continue;
            }

            unsafe {
                CELL_CACHE[idx] = (ch, attr);
            }

            draw_cell(row, col, ch, attr);
        }
    }

    unsafe {
        CACHE_VALID = true;
    }
}

fn draw_cell(row: usize, col: usize, ch: u8, attr: u8) {
    let bg_idx = attr >> 4;
    let transparent_bg = bg_idx == 0;

    let fg = PALETTE16[(attr & 0x0F) as usize];
    let bg = PALETTE16[bg_idx as usize];

    let glyph = if (0x20..=0x7E).contains(&ch) {
        FONT8X8[(ch - 0x20) as usize]
    } else {
        FONT8X8[('?' as u8 - 0x20) as usize]
    };

    let (w, h, format) = {
        let f = fb();
        (f.width, f.height, f.format)
    };


    let flip = format == PixelFormat::Rgb888;

    for gy in 0..GLYPH_H {
        let bits = glyph[gy];
        let py = row * GLYPH_H + gy;

        if py >= h {
            continue;
        }

        let vy = if flip { h - 1 - py } else { py };

        for gx in 0..GLYPH_W {
            let px = col * GLYPH_W + gx;

            if px >= w {
                continue;
            }

            let src_gx = if flip { GLYPH_W - 1 - gx } else { gx };
            let lit = bits & (0x80 >> src_gx) != 0;

            if lit {
                fb().set(px, vy, rgb(fg.0, fg.1, fg.2));
            } else if transparent_bg {
                unsafe {
                    let c = CLEAN[vy * fb().width + px];
                    fb().set(px, vy, c);
                }
            } else {
                fb().set(px, vy, rgb(bg.0, bg.1, bg.2));
            }
        }
    }
}