extern "C" {
    fn k_tick() -> u64;
}

#[inline(always)]
pub fn kernel_tick() -> u64 {
    unsafe { k_tick() }
}

#[inline(always)]
pub fn rdtsc() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        kernel_tick()
    }
}