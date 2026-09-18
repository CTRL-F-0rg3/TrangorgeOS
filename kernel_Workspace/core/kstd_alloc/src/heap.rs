use kstd_base::Status;
use crate::traits::{Allocator, Layout};

extern "C" {
    fn kmalloc(size: usize) -> *mut u8;
    fn kzalloc(size: usize) -> *mut u8;
    fn kcalloc(n: usize, size: usize) -> *mut u8;
    fn krealloc(ptr: *mut u8, size: usize) -> *mut u8;
    fn kfree(ptr: *mut u8);
    fn kmalloc_aligned(size: usize, align: usize) -> *mut u8;
}

pub struct KernelHeap;

impl Allocator for KernelHeap {
    fn alloc(&self, layout: Layout) -> Result<*mut u8, Status> {
        let ptr = if layout.align() > 8 {
            unsafe { kmalloc_aligned(layout.size(), layout.align()) }
        } else {
            unsafe { kmalloc(layout.size()) }
        };
        if ptr.is_null() {
            Err(Status::OutOfMemory)
        } else {
            Ok(ptr)
        }
    }

    fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if !ptr.is_null() {
            unsafe { kfree(ptr) };
        }
    }

    fn realloc(&self, ptr: *mut u8, _old: Layout, new_size: usize) -> Result<*mut u8, Status> {
        let new_ptr = unsafe { krealloc(ptr, new_size) };
        if new_ptr.is_null() {
            Err(Status::OutOfMemory)
        } else {
            Ok(new_ptr)
        }
    }
}

pub fn alloc_zeroed(size: usize) -> Result<*mut u8, Status> {
    let ptr = unsafe { kzalloc(size) };
    if ptr.is_null() { Err(Status::OutOfMemory) } else { Ok(ptr) }
}

pub fn alloc_array(n: usize, elem_size: usize) -> Result<*mut u8, Status> {
    let ptr = unsafe { kcalloc(n, elem_size) };
    if ptr.is_null() { Err(Status::OutOfMemory) } else { Ok(ptr) }
}