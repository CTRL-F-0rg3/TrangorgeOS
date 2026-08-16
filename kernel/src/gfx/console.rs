use super::font::FONT8X8;
use super::framebuffer::{Framebuffer, PixelFormat, PALETTE16, rgb};
use super::galaxy;
use crate::mm::ffi;

// Glyph size in pixels. We always render 1:1 — no scaling, no bit merging —
// so a glyph is never squashed or deformed regardless of resolution.
const GLYPH_W: usize = 8;
const GLYPH_H: usize = 8;

// Max text buffer size provided by `vga_buffer` (classic 80x25). The console
// grid never exceeds this; at lower resolutions it simply shows fewer cells,
// but always 1:1 with the font.
pub const MAX_COLS: usize = 80;
pub const MAX_ROWS: usize = 25;

static mut FB: Option<Framebuffer> = None;
// Logical background snapshot (RGB888 per pixel), captured after the galaxy
// fade-in. Used to restore the area underneath transparent text.
static mut CLEAN: alloc::vec::Vec<u32> = alloc::vec::Vec::new();

// Actual text grid, computed in init() from the resolution
// (width/GLYPH_W, height/GLYPH_H, capped to MAX_COLS/MAX_ROWS).
static mut COLS: usize = 0;
static mut ROWS: usize = 0;

// Cache of the last drawn contents of each cell (character, attribute).
// refresh() only redraws cells that actually changed — this is the main
// saving, because a full 80x25 pass drawing each glyph pixel by pixel is
// expensive, and in practice most of the screen does not change between
// frames.
static mut CELL_CACHE: [(u8, u8); MAX_COLS * MAX_ROWS] = [(0, 0); MAX_COLS * MAX_ROWS];
static mut CACHE_VALID: bool = false;

// Virtual base + byte length of the last device-mapped framebuffer. A
// resolution switch releases the previous mapping before creating a new one,
// so repeated switches do not leak virtual address space.
static mut FB_DEV_VIRT: u64 = 0;
static mut FB_DEV_SIZE: usize = 0;

// PATCH, not a fix: something in vga_buffer (outside these files) serves
// characters in a line in reverse order relative to the visible console
// width. Instead of waiting for a fix at the source, we read column `col`
// as `cols-1-col`. If/when the real cause is found and fixed in
// vga_buffer.rs, this must be disabled (set to false), otherwise the text
// will be mirrored again, just the other way around.
const REVERSE_TEXT_COLS: bool = true;

fn fb() -> &'static mut Framebuffer {
    unsafe { FB.as_mut().unwrap() }
}

/// Current number of text grid columns (resolution-dependent).
pub fn cols() -> usize {
    unsafe { COLS }
}

/// Current number of text grid rows (resolution-dependent).
pub fn rows() -> usize {
    unsafe { ROWS }
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

// Programs the DAC with the standard 16-color VGA palette (for planar mode).
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

// The text-mode hardware cursor (CRTC register 0x0A, bit 5 = disable) stays
// "alive" after switching to a graphics mode if nobody explicitly disables
// it — hence the blinking/square artifact in the middle of the screen.
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

pub fn test_fill(r: u32, g: u32, b: u32) {
    let (w, h) = {
        let f = fb();
        (f.width, f.height)
    };

    for y in 0..h {
        for x in 0..w {
            fb().set(x, y, rgb(r, g, b));
        }
    }
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
    let stride = stride as usize; // bytes per row (for Planar4: width/8)

    // Round up to a page (vmm_map_device requires alignment; the VGA window
    // is up to 64 KiB).
    let size = ((stride * height) + 0xFFF) & !0xFFF;

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
        PixelFormat::Rgb888 => {} // true color — no DAC palette needed
    }
    disable_text_cursor();

    unsafe {
        // Mode 13h framebuffer is top-down and text comes out mirrored on the
        // X axis for this hardware; keep the historical workaround for it.
        super::framebuffer::FLIP = false;
        super::framebuffer::FLIP_X = format == PixelFormat::Indexed8;
    }

    unsafe {
        FB = Some(Framebuffer {
            ptr,
            width,
            height,
            stride,
            format,
        });

        // Console grid derived from the resolution, capped to the text buffer
        // size, always 1:1 with the glyph.
        COLS = (width / GLYPH_W).clamp(1, MAX_COLS);
        ROWS = (height / GLYPH_H).clamp(1, MAX_ROWS);

        // New resolution = cache invalid, force a full redraw.
        CACHE_VALID = false;
    }

    // Fade-in step count scales down for larger resolutions; very large
    // framebuffers (e.g. 1920x1080) render once so switching is instant.
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

    let mut t = 0u32;
    loop {
        galaxy::render(fb(), t.min(256));
        delay();
        if t >= 256 {
            break;
        }
        t += step_size;
    }

    // Capture the logical background (RGB888 per pixel) for transparent text.
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
    if unsafe { FB.is_none() } {
        return;
    }

    // First refresh after init()/resolution change: full redraw. Subsequent
    // calls: only changed cells.
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
            let src_col = if REVERSE_TEXT_COLS { cols - 1 - col } else { col };
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

/// Draws one text cell 1:1 with the 8x8 font — no scaling, no bit merging.
/// Bit 7 = leftmost pixel of row `gy` (order: top->bottom, left->right), so
/// a character never comes out mirrored or distorted.
///
/// Cell background: when the attribute has bg == 0 (default/black — how the
/// vast majority of the text buffer looks), we do NOT paint a flat color;
/// instead we restore the pixel underneath the text from the CLEAN buffer
/// (so the nebula shows through behind the text, as intended). When bg is
/// explicitly set to something other than 0 (e.g. a highlight), it is
/// painted opaquely with that color — as before.
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

    let (w, h) = {
        let f = fb();
        (f.width, f.height)
    };

    for gy in 0..GLYPH_H {
        let bits = glyph[gy];
        let py = row * GLYPH_H + gy;

        if py >= h {
            continue;
        }

        for gx in 0..GLYPH_W {
            let px = col * GLYPH_W + gx;

            if px >= w {
                continue;
            }

            let lit = bits & (0x80 >> gx) != 0;

            if lit {
                fb().set(px, py, rgb(fg.0, fg.1, fg.2));
            } else if transparent_bg {
                // Restore the pixel underneath from the background snapshot.
                unsafe {
                    let c = CLEAN[py * fb().width + px];
                    fb().set(px, py, c);
                }
            } else {
                fb().set(px, py, rgb(bg.0, bg.1, bg.2));
            }
        }
    }
}