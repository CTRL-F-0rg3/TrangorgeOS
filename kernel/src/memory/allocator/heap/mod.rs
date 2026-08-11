pub mod buddy_heap;
pub mod slab;

use buddy_heap::BuddyAllocator;
use slab::{SlabBucket, SLAB_SIZES};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{null_mut, NonNull};
use spin::Mutex;

pub struct KernelHeap {
    buddy: BuddyAllocator,
    slabs: [SlabBucket; 9], // Odpowiada rozmiarom z SLAB_SIZES
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
        self.buddy.add_memory(start_addr, size);
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

impl LockedHeap {
    pub const fn new() -> Self {
        Self(Mutex::new(KernelHeap::new()))
    }

    pub unsafe fn init(&self, start_addr: usize, size: usize) {
        self.0.lock().init(start_addr, size);
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut heap = self.0.lock();
        let size = layout.size();
        let align = layout.align();

        // 1. Jeśli rozmar pasuje do Slaba, próbujemy alokować z kubełka
        if let Some(slab) = heap.select_slab(size) {
            if let Some(ptr) = slab.allocate() {
                return ptr.as_ptr();
            }

            // Kubełek jest pusty -> alokujemy nowy blok z Buddy i dodajemy do Slaba
            let block_size = slab.block_size;
            if let Some(ptr) = heap.buddy.allocate(block_size, block_size) {
                return ptr.as_ptr();
            }
        }

        // 2. Dla dużych alokacji (> 2048B) lub gdy Slab się skończył – uderzamy do Buddy
        heap.buddy
            .allocate(size, align)
            .map_or(null_mut(), |ptr| ptr.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let Some(non_null) = NonNull::new(ptr) else { return };
        let mut heap = self.0.lock();
        let size = layout.size();
        let align = layout.align();

        // Jeśli to był mały obiekt, zwracamy go do odpowiedniego Slaba
        if let Some(slab) = heap.select_slab(size) {
            slab.deallocate(non_null);
        } else {
            // Duży obiekt wraca bezpośrednio do Buddy Allocatora
            heap.buddy.deallocate(non_null, size, align);
        }
    }
}