// Framebuffer w trybie VGA 13h (320x200, 8 bpp). Kolor wewnętrznie jest
// przechowywany jako 8-bitowy indeks RGB332, ale publiczne API działa na
// u32 (RGB888) — konwersja odbywa się przy set/get.

pub struct Framebuffer {
    pub ptr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
}

fn to_index(r: u32, g: u32, b: u32) -> u8 {
    (((r >> 5) & 0x7) << 5 | ((g >> 5) & 0x7) << 2 | ((b >> 6) & 0x3)) as u8
}

fn from_index(idx: u8) -> (u32, u32, u32) {
    let r = ((idx >> 5) & 0x7) as u32 * 255 / 7;
    let g = ((idx >> 2) & 0x7) as u32 * 255 / 7;
    let b = (idx & 0x3) as u32 * 255 / 3;
    (r, g, b)
}

impl Framebuffer {
    pub fn get(&self, x: usize, y: usize) -> u32 {
        let idx = unsafe { *self.ptr.add(y * self.stride + x) };
        let (r, g, b) = from_index(idx);

        0xFF000000 | (r << 16) | (g << 8) | b
    }

    pub fn set(&mut self, x: usize, y: usize, c: u32) {
        let r = (c >> 16) & 0xFF;
        let g = (c >> 8) & 0xFF;
        let b = c & 0xFF;

        unsafe { *self.ptr.add(y * self.stride + x) = to_index(r, g, b) };
    }

    pub fn add(&mut self, x: usize, y: usize, r: u32, g: u32, b: u32) {
        let p = self.get(x, y);

        let pr = ((p >> 16) & 0xFF).min(255 - r.min(255)) + r.min(255);
        let pg = ((p >> 8) & 0xFF).min(255 - g.min(255)) + g.min(255);
        let pb = (p & 0xFF).min(255 - b.min(255)) + b.min(255);

        self.set(x, y, 0xFF000000 | (pr << 16) | (pg << 8) | pb);
    }
}

pub fn rgb(r: u32, g: u32, b: u32) -> u32 {
    0xFF000000 | ((r & 0xFF) << 16) | ((g & 0xFF) << 8) | (b & 0xFF)
}