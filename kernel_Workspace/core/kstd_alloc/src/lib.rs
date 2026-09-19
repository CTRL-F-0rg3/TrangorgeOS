#![no_std]

pub mod traits;
pub mod page;
pub mod heap;
pub mod slab;
pub mod oom;

pub use traits::{Allocator, Layout};
pub use heap::KernelHeap;

use core::alloc::{GlobalAlloc, Layout as CoreLayout};

pub struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: CoreLayout) -> *mut u8 {
        let l = Layout::new(layout.size(), layout.align());
        match KernelHeap.alloc(l) {
            Ok(ptr) => ptr,
            Err(_) => oom::invoke(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: CoreLayout) {
        let l = Layout::new(layout.size(), layout.align());
        KernelHeap.dealloc(ptr, l);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: CoreLayout, new_size: usize) -> *mut u8 {
        let l = Layout::new(layout.size(), layout.align());
        match KernelHeap.realloc(ptr, l, new_size) {
            Ok(p) => p,
            Err(_) => oom::invoke(),
        }
    }
}

#[global_allocator]
pub static GLOBAL: KernelAllocator = KernelAllocator;