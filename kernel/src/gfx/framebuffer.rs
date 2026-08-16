pub static mut FLIP: bool = false;

// Analogicznie do FLIP (oś Y): jeśli tekst/tło wychodzi lustrzanie w
// poziomie, włącz to. Domyślnie false — patrz komentarz w console::init()
// o tym, jak to przetestować i co zrobić jeśli to nie pomoże.
pub static mut FLIP_X: bool = false;

// Framebuffer w trybie VGA 13h (8 bpp). Publiczne API działa na RGB888
// (u32); konwersja do 8-bitowego indeksu RGB332 odbywa się przy set/get.

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
    fn ry(&self, y: usize) -> usize {
        unsafe {
            if FLIP { self.height - 1 - y } else { y }
        }
    }

    fn rx(&self, x: usize) -> usize {
        unsafe {
            if FLIP_X { self.width - 1 - x } else { x }
        }
    }

    pub fn get(&self, x: usize, y: usize) -> u32 {
        let idx = unsafe { *self.ptr.add(self.offset(x, y)) };
        let (r, g, b) = from_index(idx);

        0xFF000000 | (r << 16) | (g << 8) | b
    }

    pub fn set(&mut self, x: usize, y: usize, c: u32) {
        let r = (c >> 16) & 0xFF;
        let g = (c >> 8) & 0xFF;
        let b = c & 0xFF;

        let off = self.offset(x, y);
        unsafe { *self.ptr.add(off) = to_index(r, g, b) };
    }

    /// Fizyczny offset bajtu w pamięci framebuffera dla logicznych (x, y),
    /// z uwzględnieniem FLIP/FLIP_X. Publiczne, żeby console.rs mogło
    /// bezpiecznie kopiować surowe bajty (np. z bufora CLEAN) bez
    /// duplikowania tej logiki i bez ryzyka rozjazdu przy zmianie flag.
    pub fn offset(&self, x: usize, y: usize) -> usize {
        self.ry(y) * self.stride + self.rx(x)
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