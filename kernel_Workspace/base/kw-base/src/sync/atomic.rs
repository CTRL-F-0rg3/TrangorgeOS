// Re-exports and helpers for atomic operations.
pub use core::sync::atomic::*;

#[inline(always)]
pub fn compiler_fence() {
    core::sync::atomic::compiler_fence(Ordering::SeqCst);
}

#[inline(always)]
pub fn memory_barrier() {
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("mfence", options(nostack, preserves_flags)); }
}