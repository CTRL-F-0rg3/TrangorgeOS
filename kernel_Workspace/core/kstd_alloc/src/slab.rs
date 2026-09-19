use kstd_base::Status;
use crate::traits::{Allocator, Layout};

extern "C" {
    fn slab_alloc(cache: *mut u8) -> *mut u8;
    fn slab_free(cache: *mut u8, ptr: *mut u8);
    fn slab_cache_create(name: *const u8, obj_size: usize, align: usize) -> *mut u8;
    fn slab_cache_destroy(cache: *mut u8);
}

pub struct SlabCache {
    inner: *mut u8,
}

unsafe impl Send for SlabCache {}
unsafe impl Sync for SlabCache {}

impl SlabCache {
    pub fn new(name: &str, obj_size: usize, align: usize) -> Result<Self, Status> {
        let c_name = name.as_ptr();
        let cache = unsafe { slab_cache_create(c_name, obj_size, align) };
        if cache.is_null() {
            Err(Status::OutOfMemory)
        } else {
            Ok(Self { inner: cache })
        }
    }
}

impl Allocator for SlabCache {
    fn alloc(&self, _layout: Layout) -> Result<*mut u8, Status> {
        let ptr = unsafe { slab_alloc(self.inner) };
        if ptr.is_null() { Err(Status::OutOfMemory) } else { Ok(ptr) }
    }

    fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if !ptr.is_null() {
            unsafe { slab_free(self.inner, ptr) };
        }
    }
}

impl Drop for SlabCache {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { slab_cache_destroy(self.inner) };
        }
    }
}