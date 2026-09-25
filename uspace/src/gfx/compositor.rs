//! A small software compositor: z-ordered surfaces composited into the
//! kernel-provided framebuffer. Mirrors the driver-space compositor protocol
//! (`gfx_protocol_workspace/crates/gfx-server/src/compositor.rs`).

use super::render::Color;

/// A rectangular, independently-drawn surface (a window's backing buffer).
pub struct Surface {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub z: i32,
    pub pixels: Vec<Color>,
}

impl Surface {
    pub fn new(id: u32, x: i32, y: i32, width: u32, height: u32, z: i32) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
            z,
            pixels: vec![0u32; (width * height) as usize],
        }
    }

    pub fn clear(&mut self, c: Color) {
        for p in self.pixels.iter_mut() {
            *p = c;
        }
    }
}

/// A z-ordered collection of surfaces composited onto a shared framebuffer.
pub struct Compositor {
    pub width: u32,
    pub height: u32,
    surfaces: Vec<Surface>,
    next_id: u32,
}

impl Compositor {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            surfaces: Vec::new(),
            next_id: 1,
        }
    }

    /// Create a surface and return its id.
    pub fn create_surface(&mut self, x: i32, y: i32, width: u32, height: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.surfaces
            .push(Surface::new(id, x, y, width, height, id as i32));
        id
    }

    pub fn surface_mut(&mut self, id: u32) -> Option<&mut Surface> {
        self.surfaces.iter_mut().find(|s| s.id == id)
    }

    /// Composite every surface (front-to-back by z) into the framebuffer.
    pub fn composite(&self, fb: &mut [u32]) {
        // Sort ascending z so later (higher-z) surfaces are drawn on top.
        let mut order: Vec<&Surface> = self.surfaces.iter().collect();
        order.sort_by_key(|s| s.z);

        for s in order {
            for row in 0..s.height {
                let dy = s.y + row as i32;
                if dy < 0 || dy >= self.height as i32 {
                    continue;
                }
                for col in 0..s.width {
                    let dx = s.x + col as i32;
                    if dx < 0 || dx >= self.width as i32 {
                        continue;
                    }
                    let src = s.pixels[(row * s.width + col) as usize];
                    fb[(dy as u32 * self.width + dx as u32) as usize] = src;
                }
            }
        }
    }
}
