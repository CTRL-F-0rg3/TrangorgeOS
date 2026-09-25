//! Software renderer: fills, borders and 8x8 bitmap text.

pub type Color = u32;

/// Pack an RGB triple into an XRGB8888 pixel.
pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    0xFF00_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Fill a rectangle. Clipped to the buffer.
pub fn fill_rect(
    buf: &mut [u32],
    w: u32,
    h: u32,
    x: i32,
    y: i32,
    rw: u32,
    rh: u32,
    c: Color,
) {
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

/// Draw a 1-pixel rectangle border.
pub fn draw_rect(
    buf: &mut [u32],
    w: u32,
    h: u32,
    x: i32,
    y: i32,
    rw: u32,
    rh: u32,
    c: Color,
) {
    fill_rect(buf, w, h, x, y, rw, 1, c);
    fill_rect(buf, w, h, x, y + rh as i32 - 1, rw, 1, c);
    fill_rect(buf, w, h, x, y, 1, rh, c);
    fill_rect(buf, w, h, x + rw as i32 - 1, y, 1, rh, c);
}

/// Draw an ASCII string using the built-in 8x8 font. Unmapped glyphs render
/// as a solid block.
pub fn draw_text(
    buf: &mut [u32],
    w: u32,
    h: u32,
    x: i32,
    y: i32,
    text: &str,
    c: Color,
) {
    let mut cx = x;
    for &b in text.as_bytes() {
        let g = glyph(b);
        for (row, bits) in g.iter().enumerate() {
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
        cx += 8;
    }
}

/// 8x8 bitmap glyph for a small ASCII subset (space, A-Z, and a few symbols).
/// Returns a solid block for unmapped characters.
fn glyph(ch: u8) -> [u8; 8] {
    match ch {
        b' ' => [0x00; 8],
        b'T' => [0xFE, 0xFE, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10],
        b'E' => [0xFE, 0xFE, 0xC0, 0xC0, 0xFC, 0xFC, 0xC0, 0xC0],
        b'R' => [0xFC, 0xFE, 0xC6, 0xC6, 0xFC, 0xF8, 0xCC, 0xC6],
        b'M' => [0xC6, 0xEE, 0xFE, 0xD6, 0xD6, 0xC6, 0xC6, 0xC6],
        b'I' => [0xFE, 0xFE, 0x38, 0x38, 0x38, 0x38, 0xFE, 0xFE],
        b'N' => [0xC6, 0xE6, 0xF6, 0xDE, 0xCE, 0xC6, 0xC6, 0xC6],
        b'A' => [0x38, 0x7C, 0xC6, 0xC6, 0xFE, 0xFE, 0xC6, 0xC6],
        b'L' => [0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xFE, 0xFE],
        b'G' => [0x7C, 0xFE, 0xC6, 0xC0, 0xCE, 0xC6, 0xFE, 0x7C],
        b'O' => [0x7C, 0xFE, 0xC6, 0xC6, 0xC6, 0xC6, 0xFE, 0x7C],
        b'S' => [0x7C, 0xFE, 0xC6, 0x0E, 0x7C, 0xE0, 0xC6, 0xFE],
        b'D' => [0xF8, 0xFC, 0xC6, 0xC6, 0xC6, 0xC6, 0xFC, 0xF8],
        b'Y' => [0xC6, 0xEE, 0x7C, 0x38, 0x38, 0x38, 0x38, 0x38],
        b'K' => [0xC6, 0xCC, 0xD8, 0xF0, 0xF0, 0xD8, 0xCC, 0xC6],
        b'>' => [0x80, 0xC0, 0xE0, 0xF0, 0xE0, 0xC0, 0x80, 0x00],
        _ => [0xFF; 8],
    }
}
