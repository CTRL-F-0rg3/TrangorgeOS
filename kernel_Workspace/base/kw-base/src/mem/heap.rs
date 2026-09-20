use crate::core::{KResult, KernelError};
use kstd_alloc::{Allocator, Layout, KernelHeap};

pub fn alloc<T>() -> KResult<*mut T> {
    let layout = Layout::new(core::mem::size_of::<T>(), core::mem::align_of::<T>());
    let ptr = KernelHeap.alloc(layout).map_err(KernelError::from)?;
    Ok(ptr as *mut T)
}

pub fn alloc_zeroed<T>() -> KResult<*mut T> {
    let ptr = alloc::<T>()?;
    unsafe { core::ptr::write_bytes(ptr, 0, 1); }
    Ok(ptr)
}

pub unsafe fn free<T>(ptr: *mut T) {
    let layout = Layout::new(core::mem::size_of::<T>(), core::mem::align_of::<T>());
    KernelHeap.dealloc(ptr as *mut u8, layout);
}

pub fn alloc_slice<T>(len: usize) -> KResult<*mut T> {
    let layout = Layout::new(core::mem::size_of::<T>() * len, core::mem::align_of::<T>());
    let ptr = KernelHeap.alloc(layout).map_err(KernelError::from)?;
    Ok(ptr as *mut T)
}

pub unsafe fn free_slice<T>(ptr: *mut T, len: usize) {
    let layout = Layout::new(core::mem::size_of::<T>() * len, core::mem::align_of::<T>());
    KernelHeap.dealloc(ptr as *mut u8, layout);
}