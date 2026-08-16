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

/// Bazowy adres direct mapu (zgodny z ARCH_DIRECT_MAP_BASE w memory.h).
pub const DIRECT_MAP_BASE: u64 = 0xFFFF888000000000;

pub fn phys_to_virt(phys: u64) -> *mut u8 {
    (DIRECT_MAP_BASE + phys) as *mut u8
}

pub fn self_test() -> Result<&'static str, &'static str> {
    // single frames: alignment + uniqueness
    let mut frames = [0u64; 8];

    for f in frames.iter_mut() {
        *f = alloc_frame().ok_or("phys: frame alloc failed")?;
        if *f % 4096 != 0 {
            return Err("phys: frame not page-aligned");
        }
    }

    for i in 0..frames.len() {
        for j in (i + 1)..frames.len() {
            if frames[i] == frames[j] {
                return Err("phys: frames not distinct");
            }
        }
    }

    for &f in frames.iter() {
        if !free_frame(f) {
            return Err("phys: frame free failed");
        }
    }

    // zero-frame musi być wyzerowany (przez direct map)
    let z = alloc_zero_frame().ok_or("phys: zero frame alloc failed")?;
    let zv = phys_to_virt(z);

    for k in 0..4096 {
        if unsafe { *zv.add(k) } != 0 {
            free_frame(z);
            return Err("phys: zero frame not zeroed");
        }
    }

    if !free_frame(z) {
        return Err("phys: zero frame free failed");
    }

    // ciągła alokacja 16 ramek
    let c = alloc_frames(16).ok_or("phys: frames alloc failed")?;

    if c == 0 || c % 4096 != 0 {
        return Err("phys: frames alloc misaligned");
    }

    if !free_frames(c, 16) {
        return Err("phys: frames free failed");
    }

    // wyrównanie do 2 MiB
    let a = alloc_frames_aligned(512, 512).ok_or("phys: aligned frames failed")?;

    if a % (512 * 4096) != 0 {
        free_frames(a, 512);
        return Err("phys: frames not 2M-aligned");
    }

    if !free_frames(a, 512) {
        return Err("phys: aligned frames free failed");
    }

    Ok("pmm frames roundtrip")
}