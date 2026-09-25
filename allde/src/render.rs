//! Software renderer: rectangles, text (8x8 font) and the cursor.

use crate::font::FONT8X8;

pub type Color = u32;

pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub fn fill_rect(buf: &mut [u32], w: u32, h: u32, x: i32, y: i32, rw: u32, rh: u32, c: Color) {
    for row in 0..rh {
        let dy = y + row as i32;
        if dy < 0 || dy >= h as i32 {
            continue;
        }
        for col in 0..rw {
            let dx = x + col as i32;
            if dx < 0 || dx >= w as i32 {
                continue;
            }
            buf[(dy as u32 * w + dx as u32) as usize] = c;
        }
    }
}

pub fn draw_rect(buf: &mut [u32], w: u32, h: u32, x: i32, y: i32, rw: u32, rh: u32, c: Color) {
    fill_rect(buf, w, h, x, y, rw, 1, c);
    fill_rect(buf, w, h, x, y + rh as i32 - 1, rw, 1, c);
    fill_rect(buf, w, h, x, y, 1, rh, c);
    fill_rect(buf, w, h, x + rw as i32 - 1, y, 1, rh, c);
}

pub fn draw_text(buf: &mut [u32], w: u32, h: u32, x: i32, y: i32, text: &str, c: Color) {
    let mut cx = x;
    for &b in text.as_bytes() {
        if (32..=126).contains(&b) {
            let glyph = &FONT8X8[(b - 32) as usize];
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if bits & (0x80 >> col) != 0 {
                        let dx = cx + col as i32;
                        let dy = y + row as i32;
                        if dx >= 0 && dx < w as i32 && dy >= 0 && dy < h as i32 {
                            buf[(dy as u32 * w + dx as u32) as usize] = c;
                        }
                    }
                }
            }
        }
        cx += 8;
    }
}

/// Draw a small block cursor (pointer).
pub fn draw_cursor(buf: &mut [u32], w: u32, h: u32, x: u32, y: u32) {
    let c = rgb(0xFF, 0xFF, 0xFF);
    let outline = rgb(0x10, 0x10, 0x10);
    fill_rect(buf, w, h, x as i32, y as i32, 8, 8, c);
    draw_rect(buf, w, h, x as i32, y as i32, 8, 8, outline);
}
