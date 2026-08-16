use super::ffi;

pub fn alloc_frame() -> Option<u64> {
    let mut phys = 0u64;

    if unsafe { ffi::pmm_alloc_frame(&mut phys) } {
        Some(phys)
    } else {
        None
    }
}

pub fn alloc_zero_frame() -> Option<u64> {
    let mut phys = 0u64;

    if unsafe { ffi::pmm_alloc_zero_frame(&mut phys) } {
        Some(phys)
    } else {
        None
    }
}

pub fn alloc_frames(count: usize) -> Option<u64> {
    let mut phys = 0u64;

    if unsafe { ffi::pmm_alloc_frames(count, &mut phys) } {
        Some(phys)
    } else {
        None
    }
}

pub fn alloc_frames_aligned(count: usize, align: usize) -> Option<u64> {
    let mut phys = 0u64;

    if unsafe { ffi::pmm_alloc_frames_aligned(count, align, &mut phys) } {
        Some(phys)
    } else {
        None
    }
}

pub fn free_frame(phys: u64) -> bool {
    unsafe { ffi::pmm_free_frame(phys) }
}

pub fn free_frames(phys: u64, count: usize) -> bool {
    unsafe { ffi::pmm_free_frames(phys, count) }
}

pub fn reserve(base: u64, len: u64) {
    unsafe { ffi::arch_memory_reserve_range(base, len) }
}

pub fn total_bytes() -> u64 {
    unsafe { ffi::mm_total_ram() }
}

pub fn free_bytes() -> u64 {
    unsafe { ffi::mm_free_ram() }
}