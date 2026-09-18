use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use crate::alloc::buddy::BuddyAllocator;
use ds_log::ds_error;

pub struct DriverHeap {
    buddy: BuddyAllocator,
    // TODO: Add array of SlabAllocator for sizes: 16, 32, 64, 128, 256, 512, 1024
}

impl DriverHeap {
    pub const fn new() -> Self {
        Self {
            buddy: BuddyAllocator::new(0, 0),
        }
    }

    pub fn init(&mut self, base_addr: usize, total_pages: usize) {
        self.buddy = BuddyAllocator::new(base_addr, total_pages);
    }
}

unsafe impl GlobalAlloc for DriverHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ds_error!("DriverHeap::alloc called before full initialization or unimplemented");
        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // TODO: Route to slab or buddy based on layout.size()
    }
}