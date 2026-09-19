#[inline(always)]
pub unsafe fn read_volatile<T>(src: *const T) -> T {
    core::ptr::read_volatile(src)
}

#[inline(always)]
pub unsafe fn write_volatile<T>(dst: *mut T, val: T) {
    core::ptr::write_volatile(dst, val)
}

#[inline(always)]
pub fn memory_barrier() {
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
}

#[inline(always)]
pub unsafe fn memcpy(dst: *mut u8, src: *const u8, n: usize) {
    core::ptr::copy_nonoverlapping(src, dst, n);
}

#[inline(always)]
pub unsafe fn memset(dst: *mut u8, val: u8, n: usize) {
    core::ptr::write_bytes(dst, val, n);
}

#[inline(always)]
pub unsafe fn memmove(dst: *mut u8, src: *const u8, n: usize) {
    core::ptr::copy(src, dst, n);
}