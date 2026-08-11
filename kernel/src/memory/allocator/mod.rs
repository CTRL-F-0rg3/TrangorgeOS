

pub mod config;
pub mod traits;
pub mod stats;

pub mod physical;
pub mod r#virtual; 
pub mod heap;

pub use traits::{Frame, FrameAllocator, MapError, MapFlags, PhysAddr, VirtAddr, VirtualMapper};

use heap::LockedHeap;

#[global_allocator]
pub static HEAP_ALLOCATOR: LockedHeap = LockedHeap::new();

pub unsafe fn init_heap(heap_start: usize, heap_size: usize) {
    HEAP_ALLOCATOR.init(heap_start, heap_size);
}