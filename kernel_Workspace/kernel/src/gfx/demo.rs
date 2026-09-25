//! The `demouserspace` graphical-window demo — drawn directly on the kernel
//! framebuffer from the in-system terminal.
//!
//! This mirrors `uspace::gfx::app::draw_demo_window` but runs in kernel space
//! (no_std) so it can be triggered by the `demouserspace` terminal command.

use super::console::try_fb;
use super::font::FONT8X8;
use super::framebuffer::{rgb, Framebuffer};

fn fill_rect(fb: &mut Framebuffer, x: usize, y: usize, w: usize, h: usize, c: u32) {
    for yy in 0..h {
        for xx in 0..w {
            fb.set(x + xx, y + yy, c);
        }
    }
}

fn draw_rect(fb: &mut Framebuffer, x: usize, y: usize, w: usize, h: usize, c: u32) {
    fill_rect(fb, x, y, w, 1, c);
    fill_rect(fb, x, y + h - 1, w, 1, c);
    fill_rect(fb, x, y, 1, h, c);
    fill_rect(fb, x + w - 1, y, 1, h, c);
}

fn draw_text(fb: &mut Framebuffer, x: usize, y: usize, text: &str, c: u32) {
    let mut cx = x;
    for &b in text.as_bytes() {
        if b >= 32 && b <= 126 {
            let glyph = &FONT8X8[(b - 32) as usize];
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if bits & (0x80 >> col) != 0 {
                        fb.set(cx + col, y + row, c);
                    }
                }
            }
        }
        cx += 8;
    }
}

/// Draw a window with a small terminal onto the framebuffer. Returns `false`
/// when the framebuffer is not available (gfx inactive).
pub fn draw_demo_window() -> bool {
    let fb: &mut Framebuffer = match try_fb() {
        Some(f) => f,
        None => return false,
    };

    let w = fb.width;
    let h = fb.height;

    // Background.
    fill_rect(&mut *fb, 0, 0, w, h, rgb(0x10, 0x12, 0x18));

    // Window geometry (fits the buffer with a margin).
    let ww = (w.saturating_sub(40)).min(420);
    let wh = (h.saturating_sub(40)).min(260);
    let wx = (w - ww) / 2;
    let wy = (h - wh) / 2;

    // Window body + border + title bar.
    fill_rect(&mut *fb, wx, wy, ww, wh, rgb(0x1E, 0x22, 0x2C));
    draw_rect(&mut *fb, wx, wy, ww, wh, rgb(0x3A, 0x44, 0x5A));
    fill_rect(&mut *fb, wx + 1, wy + 1, ww - 2, 22, rgb(0x2A, 0x35, 0x4A));

    // Terminal content.
    draw_text(&mut *fb, wx + 8, wy + 5, "TERMINAL", rgb(0xD0, 0xD8, 0xE0));
    draw_text(&mut *fb, wx + 14, wy + 38, "TRANGORGEOS", rgb(0xD0, 0xD8, 0xE0));
    draw_text(&mut *fb, wx + 14, wy + 58, "> READY", rgb(0x7C, 0xF0, 0x9C));
    draw_text(&mut *fb, wx + 14, wy + 78, "KERNEL OK", rgb(0xD0, 0xD8, 0xE0));

    true
}

/// Draw a desktop-like frame (two tiled windows + a cursor) — the in-system
/// preview of the `all-de` desktop. The full interactive environment lives in
/// the userspace `allde` crate.
pub fn draw_desktop() -> bool {
    let fb: &mut Framebuffer = match try_fb() {
        Some(f) => f,
        None => return false,
    };

    let w = fb.width;
    let h = fb.height;

    fill_rect(&mut *fb, 0, 0, w, h, rgb(0x10, 0x12, 0x18));

    let col_w = w / 2;
    for (i, title) in ["terminal 1", "terminal 2"].iter().enumerate() {
        let x = i * col_w;
        fill_rect(&mut *fb, x, 0, col_w - 2, h, rgb(0x1E, 0x22, 0x2C));
        draw_rect(&mut *fb, x, 0, col_w - 2, h, rgb(0x3A, 0x44, 0x5A));
        fill_rect(&mut *fb, x + 1, 1, col_w - 4, 20, rgb(0x2A, 0x35, 0x4A));
        draw_text(&mut *fb, x + 6, 4, title, rgb(0xD0, 0xD8, 0xE0));
        draw_text(&mut *fb, x + 6, 28, "user@allde:~$", rgb(0x7C, 0xF0, 0x9C));
    }

    // Cursor.
    fill_rect(&mut *fb, col_w, h / 2, 8, 8, rgb(0xFF, 0xFF, 0xFF));

    true
}
