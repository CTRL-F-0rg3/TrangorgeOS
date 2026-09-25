//! The demo graphical environment: a window with a terminal, drawn through the
//! compositor into the kernel-provided framebuffer.

use super::compositor::Compositor;
use super::render::{draw_rect, draw_text, fill_rect, rgb, Color};

pub const BG: Color = rgb(0x10, 0x12, 0x18);
pub const WIN_BG: Color = rgb(0x1E, 0x22, 0x2C);
pub const BORDER: Color = rgb(0x3A, 0x44, 0x5A);
pub const TITLE: Color = rgb(0x2A, 0x35, 0x4A);
pub const TEXT: Color = rgb(0xD0, 0xD8, 0xE0);
pub const PROMPT: Color = rgb(0x7C, 0xF0, 0x9C);

/// Draw a window whose contents are a small terminal, through the compositor.
///
/// `fb` is the kernel-provided framebuffer buffer; `width`/`height` its size.
pub fn draw_demo_window(fb: &mut [u32], width: u32, height: u32) {
    // Background.
    fill_rect(fb, width, height, 0, 0, width, height, BG);

    let mut comp = Compositor::new(width, height);

    // Window sized to fit the buffer with a margin (the kernel buffer can be
    // larger than the demo on a real display).
    let ww: u32 = (width.saturating_sub(40)).min(420);
    let wh: u32 = (height.saturating_sub(40)).min(260);
    let wx = (width as i32 - ww as i32) / 2;
    let wy = (height as i32 - wh as i32) / 2;

    let id = comp.create_surface(wx, wy, ww, wh);

    // Draw the window content into the surface's local buffer.
    if let Some(s) = comp.surface_mut(id) {
        let buf = &mut s.pixels[..];
        fill_rect(buf, ww, wh, 0, 0, ww, wh, WIN_BG);
        draw_rect(buf, ww, wh, 0, 0, ww, wh, BORDER);
        fill_rect(buf, ww, wh, 1, 1, ww - 2, 22, TITLE);
        draw_text(buf, ww, wh, 8, 5, "TERMINAL", TEXT);

        draw_text(buf, ww, wh, 14, 38, "TRANGORGEOS", TEXT);
        draw_text(buf, ww, wh, 14, 58, "> READY", PROMPT);
        draw_text(buf, ww, wh, 14, 78, "KERNEL OK", TEXT);
    }

    // Composite the surface(s) into the kernel framebuffer.
    comp.composite(fb);
}
