use x86_64::instructions::port::Port;

pub static mut FLIP: bool = false;

pub static mut FLIP_X: bool = false;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Indexed8,
    Planar4,
    Rgb888,
}

pub const PALETTE16: [(u32, u32, u32); 16] = [
    (0, 0, 0), (0, 0, 170), (0, 170, 0), (0, 170, 170),
    (170, 0, 0), (170, 0, 170), (170, 85, 0), (170, 170, 170),
    (85, 85, 85), (85, 85, 255), (85, 255, 85), (85, 255, 255),
    (255, 85, 85), (255, 85, 255), (255, 255, 85), (255, 255, 255),
];

pub struct Framebuffer {
    pub ptr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: PixelFormat,
}

fn rgb332_index(r: u32, g: u32, b: u32) -> u8 {
    (((r >> 5) & 0x7) << 5 | ((g >> 5) & 0x7) << 2 | ((b >> 6) & 0x3)) as u8
}

fn rgb332_from_index(idx: u8) -> (u32, u32, u32) {
    let r = ((idx >> 5) & 0x7) as u32 * 255 / 7;
    let g = ((idx >> 2) & 0x7) as u32 * 255 / 7;
    let b = (idx & 0x3) as u32 * 255 / 3;
    (r, g, b)
}

fn rgb_to_index4(r: u32, g: u32, b: u32) -> u8 {
    let mut best = 0u8;
    let mut best_dist = u32::MAX;
    for (i, &(pr, pg, pb)) in PALETTE16.iter().enumerate() {
        let dr = r as i32 - pr as i32;
        let dg = g as i32 - pg as i32;
        let db = b as i32 - pb as i32;
        let dist = (dr * dr + dg * dg + db * db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best = i as u8;
        }
    }
    best
}

fn index4_to_rgb(idx: u8) -> (u32, u32, u32) {
    PALETTE16[(idx & 0xF) as usize]
}

impl Framebuffer {
    fn ry(&self, y: usize) -> usize {
        unsafe { if FLIP { self.height - 1 - y } else { y } }
    }

    fn rx(&self, x: usize) -> usize {
        unsafe { if FLIP_X { self.width - 1 - x } else { x } }
    }

    pub fn get(&self, x: usize, y: usize) -> u32 {
        let (r, g, b) = match self.format {
            PixelFormat::Indexed8 => {
                let idx = unsafe { *self.ptr.add(self.offset(x, y)) };
                rgb332_from_index(idx)
            }
            PixelFormat::Planar4 => index4_to_rgb(self.planar_get(x, y)),
            PixelFormat::Rgb888 => {
                let off = self.ry(y) * self.stride + self.rx(x) * 4;
                let v = unsafe { (self.ptr.add(off) as *const u32).read_volatile() };
                ((v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF)
            }
        };
        0xFF000000 | (r << 16) | (g << 8) | b
    }

    pub fn set(&mut self, x: usize, y: usize, c: u32) {
        let r = (c >> 16) & 0xFF;
        let g = (c >> 8) & 0xFF;
        let b = c & 0xFF;

        match self.format {
            PixelFormat::Indexed8 => {
                let off = self.offset(x, y);
                unsafe { *self.ptr.add(off) = rgb332_index(r, g, b) };
            }
            PixelFormat::Planar4 => {
                self.planar_set(x, y, rgb_to_index4(r, g, b));
            }
            PixelFormat::Rgb888 => {
                let off = self.ry(y) * self.stride + self.rx(x) * 4;
                unsafe {
                    (self.ptr.add(off) as *mut u32).write_volatile((r << 16) | (g << 8) | b);
                }
            }
        }
    }

    pub fn offset(&self, x: usize, y: usize) -> usize {
        self.ry(y) * self.stride + self.rx(x)
    }

    pub fn add(&mut self, x: usize, y: usize, r: u32, g: u32, b: u32) {
        let p = self.get(x, y);
        let pr = ((p >> 16) & 0xFF).saturating_add(r.min(255)).min(255);
        let pg = ((p >> 8) & 0xFF).saturating_add(g.min(255)).min(255);
        let pb = (p & 0xFF).saturating_add(b.min(255)).min(255);
        self.set(x, y, 0xFF000000 | (pr << 16) | (pg << 8) | pb);
    }

    fn planar_set(&self, x: usize, y: usize, color: u8) {
        let off = self.ry(y) * (self.width / 8) + self.rx(x) / 8;
        let bit = 0x80 >> (self.rx(x) & 7);

        unsafe {
            let mut gfx = Port::<u8>::new(0x3CE);
            let mut gdata = Port::<u8>::new(0x3CF);
            gfx.write(0x08); gdata.write(bit); 
            gfx.write(0x00); gdata.write(color); 
            gfx.write(0x01); gdata.write(0x0F); 
            let dummy = self.ptr.add(off).read_volatile();
            self.ptr.add(off).write_volatile(dummy);
        }
    }

    fn planar_get(&self, x: usize, y: usize) -> u8 {
        let off = self.ry(y) * (self.width / 8) + self.rx(x) / 8;
        let bit = 0x80 >> (self.rx(x) & 7);

        let mut color = 0u8;
        unsafe {
            let mut gfx = Port::<u8>::new(0x3CE);
            let mut gdata = Port::<u8>::new(0x3CF);
            for plane in 0..4u8 {
                gfx.write(0x04); gdata.write(plane); 
                if self.ptr.add(off).read_volatile() & bit != 0 {
                    color |= 1 << plane;
                }
            }
        }
        color
    }
}

pub fn rgb(r: u32, g: u32, b: u32) -> u32 {
    0xFF000000 | ((r & 0xFF) << 16) | ((g & 0xFF) << 8) | (b & 0xFF)
}
