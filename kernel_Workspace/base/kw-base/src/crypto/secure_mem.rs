use core::ptr;

// Securely zero out memory. Uses volatile writes to prevent the compiler 
// from optimizing the memset away (crucial for wiping keys).
pub fn secure_zero(ptr: *mut u8, len: usize) {
    unsafe {
        for i in 0..len {
            ptr::write_volatile(ptr.add(i), 0);
        }
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}

pub struct SecureBuffer {
    ptr: *mut u8,
    len: usize,
}

impl SecureBuffer {
    pub fn new(len: usize) -> Option<Self> {
        let layout = core::alloc::Layout::from_size_align(len, 8).ok()?;
        let ptr = unsafe { kstd_alloc::KernelHeap.alloc(layout) }.ok()?;
        secure_zero(ptr, len);
        Some(Self { ptr, len })
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        secure_zero(self.ptr, self.len);
        let layout = core::alloc::Layout::from_size_align(self.len, 8).unwrap();
        unsafe { kstd_alloc::KernelHeap.dealloc(self.ptr, layout); }
    }
}