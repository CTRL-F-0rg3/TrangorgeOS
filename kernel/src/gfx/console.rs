use super::font::FONT8X8;
use super::framebuffer::{Framebuffer, PixelFormat, PALETTE16, rgb};
use super::galaxy;
use crate::mm::ffi;

// Glyph size in pixels (8x8 font, drawn 1:1 — never scaled or deformed). The
// console grid is recomputed from the resolution on every switch.
const GLYPH_W: usize = 8;
const GLYPH_H: usize = 8;

// Max text buffer size provided by `vga_buffer` (classic 80x25). The console
// grid never exceeds this; at higher resolutions it shows more cells, at
// lower ones fewer — but always 1:1 with the font.
pub const MAX_COLS: usize = 80;
pub const MAX_ROWS: usize = 25;

static mut FB: Option<Framebuffer> = None;
static mut FB_PHYS: u64 = 0;
static mut ENABLED: bool = true;
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

// Legacy VGA modes (13h/12h) serve text reversed on this hardware, so the
// console must read columns back-to-front for them. The Bochs VBE linear
// framebuffer (Rgb888) uses a standard top-left layout and needs no reversal.
// The value is set in init() from the pixel format.
static mut REVERSE_TEXT_COLS: bool = true;

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

pub fn set_enabled(enabled: bool) {
    unsafe {
        ENABLED = enabled;
    }
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
        PixelFormat::Rgb888 => {} // true color — no DAC palette needed
    }
    disable_text_cursor();

    unsafe {
        // Mode 13h framebuffer is top-down and text comes out mirrored on the
        // X axis for this hardware; keep the historical workaround for it.
        // The Bochs VBE linear framebuffer (Rgb888) is bottom-up on QEMU's
        // stdvga card: scanline 0 lives at the end of the buffer, so the whole
        // image must be flipped on the Y axis to come out upright.
        super::framebuffer::FLIP = format == PixelFormat::Rgb888;
        super::framebuffer::FLIP_X = format == PixelFormat::Indexed8;

        // Legacy VGA modes need reversed column order; the linear framebuffer
        // (Rgb888) does not.
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

        // Text grid recomputed fresh from the resolution (1:1 with the glyph),
        // capped to the text buffer size.
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

    if steps <= 1 {
        // Very large framebuffers: render once at full brightness — no fade.
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
    if unsafe { FB.is_none() || !ENABLED } {
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

    let (w, h, format) = {
        let f = fb();
        (f.width, f.height, f.format)
    };

    // Two SEPARATE corrections are needed for Rgb888 (Bochs LFB) — do not
    // conflate them:
    //
    // 1. BLOCK position (Y only): the text grid is capped to
    //    MAX_COLS/MAX_ROWS, so at large resolutions it only covers a small
    //    y-band (e.g. 0..200 out of 0..1080). Framebuffer::set()/get() flip
    //    y around the *screen* height for Rgb888 (see Framebuffer::ry and
    //    the note in console::init) — correct for full-screen content like
    //    the galaxy, but that same flip carries the small text band to the
    //    *opposite* edge of the screen (bottom instead of top, rows in
    //    reverse order). Fixed below via `vy`, which mirrors the logical y
    //    around the screen height *before* handing it to set()/CLEAN, so
    //    set()'s own flip lands it back at the intended physical row. This
    //    also means gy itself must NOT be reversed when reading glyph rows:
    //    vy already reverses how gy maps to the physical row, so reversing
    //    the bits too would flip the glyph a second time (upside down).
    //
    // 2. GLYPH shape (X only): set()/get() never touch x for Rgb888
    //    (FLIP_X is only set for Indexed8 — see console::init), so there is
    //    no screen-level X correction to piggyback on. The glyph's columns
    //    still need reading back-to-front on this format to come out
    //    non-mirrored (confirmed empirically) — see src_gx below.
    //
    // Both only apply to Rgb888; legacy formats have no screen-level flip.
    let flip = format == PixelFormat::Rgb888;

    for gy in 0..GLYPH_H {
        // NOTE: gy is intentionally NOT reversed here. The block-position
        // correction below (vy = h - 1 - py) already inverts how gy maps to
        // the physical row, so reversing the glyph's row order on top of
        // that would flip it a second time — which is exactly what was
        // making letters render upside down. Only X still needs its own
        // mirror (see src_gx below), since vy only touches Y.
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
                // Restore the pixel underneath from the background snapshot.
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