//! alokacje oraz świeże strony dla kubełków slab.

pub mod buddy_heap;
pub mod slab;

use buddy_heap::BuddyAllocator;
use slab::{SlabBucket, SLAB_REFILL_SIZE, SLAB_SIZES};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{null_mut, NonNull};
use spin::Mutex;

const SLAB_BUCKET_COUNT: usize = SLAB_SIZES.len();

pub struct KernelHeap {
    buddy: BuddyAllocator,
    slabs: [SlabBucket; SLAB_BUCKET_COUNT],
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

    /// # Bezpieczeństwo: patrz [`BuddyAllocator::add_memory`].
    pub unsafe fn init(&mut self, start_addr: usize, size: usize) {
        self.buddy.add_memory(start_addr, size);
    }

    /// Dokłada kolejny region do już działającej sterty.
    /// # Bezpieczeństwo: patrz [`BuddyAllocator::add_memory`].
    pub unsafe fn grow(&mut self, start_addr: usize, size: usize) {
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

    pub fn used_bytes(&self) -> usize { self.buddy.allocated_bytes() }
    pub fn total_bytes(&self) -> usize { self.buddy.total_bytes() }
}

pub struct LockedHeap(Mutex<KernelHeap>);

impl LockedHeap {
    pub const fn new() -> Self {
        Self(Mutex::new(KernelHeap::new()))
    }

    /// # Bezpieczeństwo: patrz [`KernelHeap::init`]. Wywołać raz, zanim
    /// jakikolwiek kod użyje alokacji na stercie (`Box`, `Vec`, ...).
    pub unsafe fn init(&self, start_addr: usize, size: usize) {
        self.0.lock().init(start_addr, size);
    }

    /// # Bezpieczeństwo: patrz [`KernelHeap::grow`].
    pub unsafe fn grow(&self, start_addr: usize, size: usize) {
        self.0.lock().grow(start_addr, size);
    }

    pub fn used_bytes(&self) -> usize { self.0.lock().used_bytes() }
    pub fn total_bytes(&self) -> usize { self.0.lock().total_bytes() }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut heap = self.0.lock();
        // Layout.size() nie musi być >= align() - liczymy realny wymagany rozmiar,
        // inaczej mały, mocno wyrównany typ dostałby źle wyrównaną pamięć.
        let size = layout.size().max(layout.align());
        let align = layout.align();

        if let Some(slab) = heap.select_slab(size) {
            if let Some(ptr) = slab.allocate() {
                return ptr.as_ptr();
            }

            // Pusty kubełek: pobieramy jedną stronę z Buddy i dzielimy ją
            // na sloty, zamiast marnować całą stronę na jeden mały obiekt.
            let block_size = slab.block_size;
            let refill_size = SLAB_REFILL_SIZE.max(block_size);
            if let Some(chunk) = heap.buddy.allocate(refill_size, block_size) {
                slab.refill(chunk, refill_size);
                if let Some(ptr) = slab.allocate() {
                    return ptr.as_ptr();
                }
            }
            return null_mut();
        }

        // Duże alokacje (> 2048 B) idą bezpośrednio do Buddy.
        heap.buddy
            .allocate(size, align)
            .map_or(null_mut(), |ptr| ptr.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let Some(non_null) = NonNull::new(ptr) else { return };
        let mut heap = self.0.lock();
        let size = layout.size().max(layout.align());
        let align = layout.align();

        if let Some(slab) = heap.select_slab(size) {
            slab.deallocate(non_null);
        } else {
            heap.buddy.deallocate(non_null, size, align);
        }
    }
}