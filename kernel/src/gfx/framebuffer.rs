pub static mut FLIP: bool = false;

pub struct Framebuffer {
    pub ptr: *mut u32,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
}

impl Framebuffer {
    fn ry(&self, y: usize) -> usize {
        unsafe {
            if FLIP { self.height - 1 - y } else { y }
        }
    }

    pub fn get(&self, x: usize, y: usize) -> u32 {
        unsafe { *self.ptr.add(self.ry(y) * self.stride + x) }
    }

    pub fn set(&mut self, x: usize, y: usize, c: u32) {
        unsafe { *self.ptr.add(self.ry(y) * self.stride + x) = c }
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