//! A small pixel-level drawing surface: rectangles and text straight into the
//! framebuffer.
//!
//! # Why this exists when there is already a console
//!
//! `gfx::console` is a *text* console with a fixed 80x25 grid
//! (`vga_buffer::BUFFER_WIDTH`/`HEIGHT`), so a line drawn through it can never be
//! wider than 80 characters no matter how large the display mode is. That is a
//! property of the console, not of the screen.
//!
//! The pre-login menu is a TUI rather than a printed list for exactly that
//! reason: it draws into the framebuffer, so its width is bounded by pixels and
//! not by a character grid. The 101-column banner fits comfortably on a 1280-wide
//! mode, and does not fit at all through the console.
//!
//! # Coordinate system
//!
//! Pixels, origin top-left, like the framebuffer itself. Text is placed by its
//! top-left corner in pixels, not by a row and column, because a TUI is laying
//! out a box and not filling a terminal.

use crate::gfx::console::try_fb;
use crate::gfx::font::FONT8X8;

/// An RGB colour.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u32);

impl Color {
    /// Build a colour from components.
    ///
    /// The packing is written out rather than delegated to
    /// `gfx::framebuffer::rgb`, which is an ordinary function and so cannot be
    /// called from the `const` associated functions below.
    pub const fn from_rgb(r: u32, g: u32, b: u32) -> Self {
        Self((r << 16) | (g << 8) | b)
    }

    /// The panel background: not black, so the border reads as a shape.
    pub const fn panel() -> Self {
        Self::from_rgb(16, 20, 32)
    }

    /// The border and the title.
    pub const fn edge() -> Self {
        Self::from_rgb(90, 170, 255)
    }

    /// The highlighted row's fill.
    pub const fn highlight() -> Self {
        Self::from_rgb(40, 90, 160)
    }

    /// Ordinary text.
    pub const fn text() -> Self {
        Self::from_rgb(200, 205, 215)
    }

    /// Text sitting on a highlighted row, where a dark fill needs a light ink.
    pub const fn text_on_highlight() -> Self {
        Self::from_rgb(255, 255, 255)
    }

    /// The dimmer text used for the "(current)" marker and the help lines.
    pub const fn dim() -> Self {
        Self::from_rgb(120, 130, 150)
    }
}

/// A framebuffer to draw on, sized in pixels.
pub struct Canvas {
    pub width: u32,
    pub height: u32,
}

/// Borrow the framebuffer, or `None` when there is none to draw on.
///
/// A failure here is not fatal to a caller: the menu can fall back to the text
/// console rather than draw nothing at all.
pub fn canvas() -> Option<Canvas> {
    let fb = try_fb()?;
    Some(Canvas {
        width: fb.width as u32,
        height: fb.height as u32,
    })
}

/// Fill a rectangle, clipped to the screen.
///
/// The clipping is the point: a caller laying out a box by arithmetic will
/// eventually name a rectangle that runs off the edge, and on a framebuffer that
/// is a write outside the mapping — a page fault, not a clipped pixel.
pub fn fill_rect(c: &Canvas, x: i64, y: i64, w: i64, h: i64, color: Color) {
    let Some(fb) = try_fb() else { return };

    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + w).min(c.width as i64);
    let y1 = (y + h).min(c.height as i64);

    // Flip the clipped band rather than the requested one: clipping has to
    // happen in the caller's coordinates, where the edges are meaningful.
    let mut yy = flipped(c, y0, y1 - y0);
    while yy < flipped(c, y0, 0) {
        let mut xx = x0;
        while xx < x1 {
            fb.set(xx as usize, yy as usize, color.0);
            xx += 1;
        }
        yy += 1;
    }
}

/// Draw a rectangle outline of thickness 1.
pub fn stroke_rect(c: &Canvas, x: i64, y: i64, w: i64, h: i64, color: Color) {
    fill_rect(c, x, y, w, 1, color);
    fill_rect(c, x, y + h - 1, w, 1, color);
    fill_rect(c, x, y, 1, h, color);
    fill_rect(c, x + w - 1, y, 1, h, color);
}

/// One character cell, in pixels. The font is 8x8.
const GLYPH: i64 = 8;

/// The framebuffer row that the top of a logical band of `h` rows lands on.
///
/// # Why the TUI is the only thing that flips
///
/// The linear framebuffer the Bochs VBE extension hands out is bottom-up: row 0
/// is the *last* scanline on screen, so drawing at `y = 0` puts the picture at
/// the bottom and the whole menu arrives standing on its head.
///
/// This is a property of how the menu is composited, not of the console, and it
/// is corrected here and nowhere else. Flipping in `Framebuffer` itself would
/// also turn over the galaxy background and the text console, which are drawn
/// through paths that already place their own pixels and are not the thing
/// being complained about. So the correction lives at the one boundary that
/// introduces the problem: the TUI's own drawing calls.
fn flipped(c: &Canvas, y: i64, h: i64) -> i64 {
    c.height as i64 - y - h
}

/// Draw one character at `(x, y)`, scaled by `scale`.
///
/// `scale` exists because the same 8x8 font has two useful sizes here: 1x for the
/// banner, which is 101 characters wide and would not fit doubled, and 2x for the
/// mode list, which is short and worth making legible from a normal distance.
pub fn draw_char(c: &Canvas, x: i64, y: i64, ch: char, color: Color, scale: i64) {
    let Some(fb) = try_fb() else { return };

    let cp = ch as u32;
    if !(0x20..0x7F).contains(&cp) {
        return;
    }

    let glyph = &FONT8X8[(cp - 0x20) as usize];
    let cw = GLYPH * scale;

    if x < 0 || y < 0 || x + cw > c.width as i64 || y + cw > c.height as i64 {
        return;
    }

    // The band this glyph occupies, turned over into framebuffer rows. Both the
    // band's origin and the order of the rows within it are inverted, so the
    // glyph is not merely moved but genuinely stood upright.
    let base = flipped(c, y, cw);

    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..GLYPH {
            // `FONT8X8` is one byte per row, low bit leftmost. A set bit is ink.
            if bits & (1 << col) == 0 {
                continue;
            }
            // `row` counts down from the top of the cell, so the scanline is
            // `base + row` counted up from the flipped band.
            let fyy = base + row as i64;
            for sy in 0..scale {
                for sx in 0..scale {
                    fb.set(
                        (x + col * scale + sx) as usize,
                        (fyy + scale - 1 - sy) as usize,
                        color.0,
                    );
                }
            }
        }
    }
}

/// Draw a string, returning the x just past the last character.
pub fn draw_text(c: &Canvas, x: i64, y: i64, text: &str, color: Color, scale: i64) -> i64 {
    let mut cx = x;
    for ch in text.chars() {
        if ch == '\n' {
            continue;
        }
        draw_char(c, cx, y, ch, color, scale);
        cx += GLYPH * scale;
    }
    cx
}

/// Width in pixels of `text` at `scale`.
pub fn text_width(text: &str, scale: i64) -> i64 {
    text.chars().count() as i64 * GLYPH * scale
}
