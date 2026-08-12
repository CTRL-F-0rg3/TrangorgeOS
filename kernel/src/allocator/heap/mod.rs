pub mod buddy_heap;
pub mod slab;

use crate::allocator::stats;
use buddy_heap::BuddyAllocator;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{NonNull, null_mut};
use slab::{SLAB_SIZES, SlabBucket};
use spin::Mutex;

const SLAB_REFILL_BLOCKS: usize = 16;

pub struct KernelHeap {
    buddy: BuddyAllocator,
    slabs: [SlabBucket; 9],
}

impl KernelHeap {
    pub const fn new() -> Self {
        Self {
            buddy: BuddyAllocator::new(),
            slabs: [
                SlabBucket::new(8),
                SlabBucket::new(16),
                SlabBucket::new(32),
                SlabBucket::new(64),
                SlabBucket::new(128),
                SlabBucket::new(256),
                SlabBucket::new(512),
                SlabBucket::new(1024),
                SlabBucket::new(2048),
            ],
        }
    }

    pub unsafe fn init(&mut self, start_addr: usize, size: usize) {
        unsafe {
            self.buddy.add_memory(start_addr, size);
        }
    }

    fn select_slab(&mut self, size: usize) -> Option<&mut SlabBucket> {
        for (i, &slab_size) in SLAB_SIZES.iter().enumerate() {
            if size <= slab_size {
                return Some(&mut self.slabs[i]);
            }
        }
        None
    }
}

pub struct LockedHeap(Mutex<KernelHeap>);

unsafe impl Sync for LockedHeap {}

impl LockedHeap {
    pub const fn new() -> Self {
        Self(Mutex::new(KernelHeap::new()))
    }

    pub unsafe fn init(&self, start_addr: usize, size: usize) {
        unsafe {
            self.0.lock().init(start_addr, size);
        }
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut heap = self.0.lock();
        let size = layout.size();
        let align = layout.align();

        let block_size = {
            let Some(slab) = heap.select_slab(size) else {
                return match heap.buddy.allocate(size, align) {
                    Some(ptr) => {
                        stats::record_alloc(size);
                        ptr.as_ptr()
                    }
                    None => null_mut(),
                };
            };
            if let Some(ptr) = slab.allocate() {
                stats::record_alloc(size);
                return ptr.as_ptr();
            }
            slab.block_size
        };

        let chunk_size = block_size * SLAB_REFILL_BLOCKS;
        let Some(chunk) = heap.buddy.allocate(chunk_size, block_size) else {
            return null_mut();
        };

        let base = chunk.as_ptr();
        if let Some(slab) = heap.select_slab(size) {
            for i in 0..SLAB_REFILL_BLOCKS {
                let block_ptr = unsafe { base.add(i * block_size) };
                unsafe {
                    slab.push_block(block_ptr);
                }
            }
            if let Some(ptr) = slab.allocate() {
                stats::record_alloc(size);
                return ptr.as_ptr();
            }
        }

        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let Some(non_null) = NonNull::new(ptr) else {
            return;
        };
        let mut heap = self.0.lock();
        let size = layout.size();
        let align = layout.align();

        if let Some(slab) = heap.select_slab(size) {
            unsafe {
                slab.deallocate(non_null);
            }
        } else {
            unsafe {
                heap.buddy.deallocate(non_null, size, align);
            }
        }
        stats::record_dealloc(size);
    }
}
