use super::ffi;
use core::ops::BitOr;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VmmFlags(u32);

impl VmmFlags {
    pub const NONE: VmmFlags = VmmFlags(0);
    pub const WRITE: VmmFlags = VmmFlags(1 << 0);
    pub const USER: VmmFlags = VmmFlags(1 << 1);
    pub const NX: VmmFlags = VmmFlags(1 << 2);
    pub const DEVICE: VmmFlags = VmmFlags(1 << 3);
    pub const ZERO: VmmFlags = VmmFlags(1 << 4);

    pub const fn bits(self) -> u32 {
        self.0
    }
}

impl BitOr for VmmFlags {
    type Output = VmmFlags;

    fn bitor(self, rhs: VmmFlags) -> VmmFlags {
        VmmFlags(self.0 | rhs.0)
    }
}

pub const WRITE: VmmFlags = VmmFlags::WRITE;
pub const USER: VmmFlags = VmmFlags::USER;
pub const NX: VmmFlags = VmmFlags::NX;
pub const DEVICE: VmmFlags = VmmFlags::DEVICE;
pub const ZERO: VmmFlags = VmmFlags::ZERO;

pub fn alloc(bytes: usize, flags: VmmFlags) -> Option<u64> {
    let mut virt = 0u64;

    if unsafe { ffi::vmm_alloc(bytes, flags.bits(), &mut virt) } {
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

pub fn self_test() -> Result<&'static str, &'static str> {
    let v = alloc(4096, WRITE).ok_or("vmm: alloc failed")?;

    if v == 0 {
        return Err("vmm: alloc returned 0");
    }

    if !free(v, 4096) {
        return Err("vmm: free failed");
    }

    // większy region + USER
    let v2 = alloc(8192, WRITE | USER).ok_or("vmm: alloc(8192) failed")?;

    if v2 == 0 {
        return Err("vmm: alloc(8192) returned 0");
    }

    if !free(v2, 8192) {
        return Err("vmm: free(8192) failed");
    }

    let d = map_device(0xFEE00000, 4096).ok_or("vmm: map_device failed")?;

    if d == 0 {
        return Err("vmm: map_device returned 0");
    }

    if !unmap_device(d, 4096) {
        return Err("vmm: unmap_device failed");
    }

    Ok("vmm alloc/free/map_device roundtrip")
}