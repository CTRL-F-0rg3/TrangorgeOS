
pub mod bitmap;
pub mod boot_bridge;
pub mod buddy;

use crate::allocator::traits::{Frame, FrameAllocator};
use bitmap::BitmapFrameAllocator;
use spin::Mutex;

pub struct PhysicalAllocator {
    inner: Mutex<Option<BitmapFrameAllocator<'static>>>,
}

impl PhysicalAllocator {
    pub const fn uninitialized() -> Self {
        Self { inner: Mutex::new(None) }
    }

    pub fn init(&self, bitmap: &'static mut [u64], total_frames: usize) {
        let mut guard = self.inner.lock();
        assert!(guard.is_none(), "PhysicalAllocator już zainicjalizowany!");
        *guard = Some(BitmapFrameAllocator::new(bitmap, total_frames));
    }

    pub fn init_with(&self, allocator: BitmapFrameAllocator<'static>) {
        let mut guard = self.inner.lock();
        assert!(guard.is_none(), "PhysicalAllocator już zainicjalizowany!");
        *guard = Some(allocator);
    }

    pub fn allocate_frame(&self) -> Option<Frame> {
        let mut guard = self.inner.lock();
        let alloc = guard.as_mut().expect("PhysicalAllocator niezainicjalizowany");
        FrameAllocator::allocate_frame(alloc)
    }

    pub fn deallocate_frame(&self, frame: Frame) {
        let mut guard = self.inner.lock();
        let alloc = guard.as_mut().expect("PhysicalAllocator niezainicjalizowany");
        FrameAllocator::deallocate_frame(alloc, frame)
    }

    pub fn stats(&self) -> (usize, usize) {
        let guard = self.inner.lock();
        let alloc = guard.as_ref().expect("PhysicalAllocator niezainicjalizowany");
        (alloc.total_frames(), alloc.free_frames())
    }
}

pub static PHYS_ALLOCATOR: PhysicalAllocator = PhysicalAllocator::uninitialized();