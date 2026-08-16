use super::ffi;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;

pub fn kmalloc(size: usize) -> Option<*mut u8> {
    let p = unsafe { ffi::kmalloc(size) };

    if p.is_null() { None } else { Some(p as *mut u8) }
}

pub fn kzalloc(size: usize) -> Option<*mut u8> {
    let p = unsafe { ffi::kzalloc(size) };

    if p.is_null() { None } else { Some(p as *mut u8) }
}

pub fn krealloc(ptr: *mut u8, size: usize) -> Option<*mut u8> {
    let p = unsafe { ffi::krealloc(ptr as *mut c_void, size) };

    if p.is_null() { None } else { Some(p as *mut u8) }
}

pub fn kfree(ptr: *mut u8) {
    unsafe { ffi::kfree(ptr as *mut c_void) }
}

pub fn virt_to_phys(ptr: *mut u8) -> u64 {
    unsafe { ffi::kvirt_to_phys(ptr as *mut c_void) }
}

pub struct KernelAlloc;

unsafe impl GlobalAlloc for KernelAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = if layout.align() <= 16 {
            ffi::kmalloc(layout.size())
        } else {
            ffi::kmalloc_aligned(layout.size(), layout.align())
        };

        p as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        ffi::kfree(ptr as *mut c_void)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = if layout.align() <= 16 {
            ffi::kzalloc(layout.size())
        } else {
            ffi::kmalloc_aligned(layout.size(), layout.align())
        };

        p as *mut u8
    }
}