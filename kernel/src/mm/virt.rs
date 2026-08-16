use super::ffi;

pub const WRITE: u32 = 1 << 0;
pub const USER: u32 = 1 << 1;
pub const NX: u32 = 1 << 2;
pub const DEVICE: u32 = 1 << 3;
pub const ZERO: u32 = 1 << 4;

pub fn alloc(bytes: usize, flags: u32) -> Option<u64> {
    let mut virt = 0u64;

    if unsafe { ffi::vmm_alloc(bytes, flags, &mut virt) } {
        Some(virt)
    } else {
        None
    }
}

pub fn free(virt: u64, bytes: usize) -> bool {
    unsafe { ffi::vmm_free(virt, bytes) }
}

pub fn map_device(phys: u64, len: usize) -> Option<u64> {
    let mut virt = 0u64;

    if unsafe { ffi::vmm_map_device(phys, len, &mut virt) } {
        Some(virt)
    } else {
        None
    }
}

pub fn unmap_device(virt: u64, len: usize) -> bool {
    unsafe { ffi::vmm_unmap_device(virt, len) }
}