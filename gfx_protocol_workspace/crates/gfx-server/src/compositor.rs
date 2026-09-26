//! The compositor: surfaces, z-order, and the composited frame.
//!
//! # Why this allocates rather than maps
//!
//! The composited frame is *not* the framebuffer the scanout engine reads. It
//! is a staging buffer the server rasterises into, and the whole frame is then
//! handed to the GPU bridge for presentation. Keeping the two separate is what
//! lets a client surface land directly in the framebuffer later, without this
//! file having to change.
//!
//! Allocation goes through `ds_mem::DmaBuffer`, not a bare `GlobalAlloc`. A bare
//! heap would hand the compositor memory the GPU cannot DMA from, and
//! `GpuBuffer` requires a physical address: a buffer that cannot be named
//! physically cannot be uploaded.

use core::ptr::null_mut;

use ds_mem::dma::buffer::{DmaBuffer, DmaFlags};
use gfx_protocol::surfaces::PixelFormat;
use kapi_abi::DsError;

/// How many surfaces the server tracks.
///
/// Sixty-four is a deliberate ceiling, not a guess at a display's capacity: the
/// table is fixed, so `create_surface` refuses rather than failing to allocate
/// later - at which point the caller is already mid-frame.
pub const MAX_SURFACES: usize = 64;

/// The background colour behind every surface, as `0xAARRGGBB`.
const BACKGROUND: u32 = 0xFF10_1020;

struct Surface {
    id: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    z_order: i32,
    /// Owned, and released with the surface.
    buffer: DmaBuffer,
}

pub struct DisplayServer {
    pub width: u32,
    pub height: u32,
    /// Bytes per scanline of the composited frame.
    stride_bytes: u32,
    fb: DmaBuffer,
    surfaces: [Option<Surface>; MAX_SURFACES],
    next_id: u32,
}

impl DisplayServer {
    /// A server for a `width` x `height` output, or the reason it cannot have one.
    ///
    /// A zero dimension is refused here rather than producing a framebuffer of
    /// zero pixels that every later bounds check would then have to
    /// special-case.
    pub fn new(width: u32, height: u32) -> Result<Self, DsError> {
        if width == 0 || height == 0 {
            return Err(DsError::InvalidMessage);
        }

        // A composited frame of `width` pixels is at least `width * 4` bytes
        // per scanline, times the height.
        let fb_size = (width as u64) * (height as u64) * 4;
        let fb = DmaBuffer::allocate(
            kapi_abi::Handle(0),
            fb_size,
            DmaFlags(DmaFlags::COHERENT.0 | DmaFlags::CONTIGUOUS.0),
        )?;

        Ok(Self {
            width,
            height,
            stride_bytes: width * 4,
            fb,
            // `Surface` owns a `DmaBuffer` and so is not `Copy`; the inline
            // `const` block is what lets a non-Copy value be repeated here.
            surfaces: [const { None }; MAX_SURFACES],
            next_id: 1,
        })
    }

    /// Create a surface, returning its id, or `0` when it could not be made.
    ///
    /// `0` is not a valid id: `next_id` starts at one, so zero is free to mean
    /// "no surface was created" without colliding with a real handle.
    pub fn create_surface(&mut self, width: u32, height: u32) -> u32 {
        if width == 0 || height == 0 {
            return 0;
        }

        let buffer_size = (width as u64) * (height as u64) * 4;
        let Ok(buffer) = DmaBuffer::allocate(
            kapi_abi::Handle(0),
            buffer_size,
            DmaFlags(DmaFlags::COHERENT.0),
        ) else {
            return 0;
        };

        let id = self.next_id;
        self.next_id += 1;

        for slot in self.surfaces.iter_mut() {
            if slot.is_none() {
                *slot = Some(Surface { id, x: 0, y: 0, width, height, z_order: 0, buffer });
                return id;
            }
        }

        0
    }

    /// Destroy a surface and release its buffer.
    pub fn destroy_surface(&mut self, id: u32) -> bool {
        for slot in self.surfaces.iter_mut() {
            if let Some(s) = slot {
                if s.id == id {
                    *slot = None;
                    return true;
                }
            }
        }
        false
    }

    /// Move a surface, or ignore the request if it names no live surface.
    pub fn set_position(&mut self, id: u32, x: i32, y: i32) {
        for slot in self.surfaces.iter_mut() {
            if let Some(s) = slot {
                if s.id == id {
                    s.x = x;
                    s.y = y;
                    break;
                }
            }
        }
    }

    /// Set a surface's z-order.
    pub fn set_z_order(&mut self, id: u32, z: i32) {
        for slot in self.surfaces.iter_mut() {
            if let Some(s) = slot {
                if s.id == id {
                    s.z_order = z;
                    break;
                }
            }
        }
    }

    /// Mark a surface dirty.
    ///
    /// A no-op today: `composite` redraws the whole frame either way. It exists
    /// so that when damage tracking lands, the commit path is the one place
    /// that has to change, rather than every caller.
    pub fn commit(&mut self, _id: u32) {}

    /// Composite every surface, in z-order, into the staging frame.
    ///
    /// The whole frame is redrawn, background included. A partial update would
    /// need the previous frame to blend against, and the composited buffer *is*
    /// the previous frame here, so there is nothing to read from.
    pub fn composite(&mut self) {
        // SAFETY: `fb` is a live allocation of at least `width * height * 4`
        // bytes, and both factors were checked non-zero in `new`, so every
        // index below is inside it.
        unsafe {
            let fb = self.fb.virt_addr as *mut u32;
            let total = (self.width as usize) * (self.height as usize);
            for i in 0..total {
                fb.add(i).write_volatile(BACKGROUND);
            }
        }

        for entry in self.draw_order() {
            if entry.1 != 0 {
                self.blit(entry.1);
            }
        }
    }

    /// Surface ids in the order they must be drawn: lowest z first.
    ///
    /// This returns owned data rather than sorting the surface table in place,
    /// because the drawing loop reads the buffers out of that table and sorting
    /// it would move them underneath the loop.
    fn draw_order(&self) -> [(i32, u32); MAX_SURFACES] {
        let mut order = [(0i32, 0u32); MAX_SURFACES];
        let mut count = 0usize;

        for surface in self.surfaces.iter().flatten() {
            if count < MAX_SURFACES {
                order[count] = (surface.z_order, surface.id);
                count += 1;
            }
        }

        // Insertion sort: the array is tiny and is usually already ordered, and
        // this needs no allocation.
        for i in 1..count {
            let mut j = i;
            while j > 0 && order[j - 1] > order[j] {
                order.swap(j - 1, j);
                j -= 1;
            }
        }

        order
    }

    /// Draw one surface into the composited frame, clipped to both rectangles.
    fn blit(&mut self, id: u32) {
        let Some(surface) = self.surfaces.iter().flatten().find(|s| s.id == id) else {
            return;
        };

        if surface.buffer.virt_addr == 0 {
            return;
        }

        // Clip once, up front, so the inner loop is a straight copy. Clipping
        // per pixel would be correct but several times the work in the common
        // case of a surface hanging off an edge.
        let x0 = surface.x.max(0);
        let y0 = surface.y.max(0);
        let x1 = (surface.x + surface.width as i32).min(self.width as i32);
        let y1 = (surface.y + surface.height as i32).min(self.height as i32);
        if x1 <= x0 || y1 <= y0 {
            return;
        }

        let copy_w = (x1 - x0) as usize;
        // The source offset of the clipped region's top-left corner, in pixels.
        let src_x = (x0 - surface.x) as usize;
        let src_y = (y0 - surface.y) as usize;
        let fb = self.fb.virt_addr as *mut u32;
        let src = surface.buffer.virt_addr as *const u32;

        for row in 0..(y1 - y0) as usize {
            // SAFETY: `src_y + row` is below the surface's height and
            // `y0 + row` below the frame's height, both by the clipping above;
            // `copy_w` is bounded by the narrower of the two rects. Source and
            // destination are separate allocations, so the copies cannot alias.
            unsafe {
                let s = src.add((src_y + row) * (surface.width as usize) + src_x);
                let d = fb.add((y0 as usize + row) * (self.width as usize) + x0 as usize);
                core::ptr::copy_nonoverlapping(s, d, copy_w);
            }
        }
    }

    /// The composited frame, for presentation.
    pub fn framebuffer_ptr(&self) -> *const u32 {
        if self.fb.virt_addr == 0 {
            null_mut()
        } else {
            self.fb.virt_addr as *const u32
        }
    }

    /// The composited frame's size in bytes.
    pub fn fb_size(&self) -> usize {
        (self.stride_bytes as usize) * (self.height as usize)
    }

    /// The composited frame's format.
    ///
    /// The staging frame is always 32-bit; the format the scanout engine wants
    /// is the GPU bridge's business, decided when it presents.
    pub fn format(&self) -> PixelFormat {
        PixelFormat::XRGB8888
    }
}