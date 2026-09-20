use gfx_protocol::surfaces::{SurfaceId, PixelFormat};
use kstd_data::Vec;

struct Surface {
    id: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    z_order: i32,
    buffer: *mut u32,
}

pub struct DisplayServer {
    pub width: u32,
    pub height: u32,
    fb: *mut u32,
    surfaces: [Option<Surface>; 64],
    next_id: u32,
}

impl DisplayServer {
    pub fn new(width: u32, height: u32) -> Self {
        let fb_size = (width * height * 4) as usize;
        let layout = core::alloc::Layout::from_size_align(fb_size, 4096).unwrap();
        let fb = unsafe { kstd_alloc::KernelHeap.alloc(layout) } as *mut u32;

        Self { width, height, fb, surfaces: core::array::from_fn(|_| None), next_id: 1 }
    }

    pub fn create_surface(&mut self, width: u32, height: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let buf_size = (width * height * 4) as usize;
        let layout = core::alloc::Layout::from_size_align(buf_size, 4096).unwrap();
        let buffer = unsafe { kstd_alloc::KernelHeap.alloc(layout) } as *mut u32;

        for slot in self.surfaces.iter_mut() {
            if slot.is_none() {
                *slot = Some(Surface { id, x: 0, y: 0, width, height, z_order: 0, buffer });
                break;
            }
        }
        id
    }

    pub fn set_position(&mut self, id: u32, x: i32, y: i32) {
        for slot in self.surfaces.iter_mut() {
            if let Some(s) = slot {
                if s.id == id { s.x = x; s.y = y; break; }
            }
        }
    }

    pub fn commit(&mut self, _id: u32) {}

    pub fn composite(&mut self) {
        unsafe {
            for i in 0..(self.width * self.height) {
                self.fb.add(i as usize).write_volatile(0xFF101020);
            }
        }

        for slot in self.surfaces.iter() {
            if let Some(s) = slot {
                if s.buffer.is_null() { continue; }
                unsafe {
                    for row in 0..s.height {
                        let dy = s.y + row as i32;
                        if dy < 0 || dy >= self.height as i32 { continue; }
                        for col in 0..s.width {
                            let dx = s.x + col as i32;
                            if dx < 0 || dx >= self.width as i32 { continue; }
                            let src = s.buffer.add((row * s.width + col) as usize).read_volatile();
                            self.fb.add((dy as u32 * self.width + dx as u32) as usize).write_volatile(src);
                        }
                    }
                }
            }
        }
    }

    pub fn framebuffer_ptr(&self) -> *const u32 { self.fb }
    pub fn fb_size(&self) -> usize { (self.width * self.height * 4) as usize }
}