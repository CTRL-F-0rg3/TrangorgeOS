use super::ffi;
use core::ffi::c_void;

pub const PROT_READ: u32 = 1 << 0;
pub const PROT_WRITE: u32 = 1 << 1;
pub const PROT_EXEC: u32 = 1 << 2;
pub const PROT_USER: u32 = 1 << 3;

pub const MAP_ANONYMOUS: u32 = 1 << 0;
pub const MAP_PRIVATE: u32 = 1 << 1;
pub const MAP_FIXED: u32 = 1 << 3;

pub struct AddressSpace {
    ptr: *mut c_void,
}

impl AddressSpace {
    pub fn new() -> Option<Self> {
        let ptr = unsafe { ffi::aspace_create() };

        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn map_anon(&self, hint: u64, len: usize, prot: u32) -> Option<u64> {
        let at = unsafe { ffi::aspace_map_anon(self.ptr, hint, len, prot) };

        if at == 0 { None } else { Some(at) }
    }

    pub fn mmap(&self, addr: u64, len: usize, prot: u32, flags: u32) -> Option<u64> {
        let at = unsafe { ffi::mmap(self.ptr, addr, len, prot, flags) };

        if at == 0 { None } else { Some(at) }
    }

    pub fn munmap(&self, addr: u64, len: usize) -> bool {
        unsafe { ffi::munmap(self.ptr, addr, len) }
    }

    pub fn protect(&self, addr: u64, len: usize, prot: u32) -> bool {
        unsafe { ffi::aspace_protect(self.ptr, addr, len, prot) }
    }

    pub fn brk(&self, new_brk: u64) -> u64 {
        unsafe { ffi::aspace_brk(self.ptr, new_brk) }
    }

    pub fn switch(&self) {
        let handle = unsafe { ffi::aspace_paging_handle(self.ptr) };
        unsafe { ffi::paging_aspace_switch(handle) }
    }
}

impl Drop for AddressSpace {
    fn drop(&mut self) {
        unsafe { ffi::aspace_destroy(self.ptr) }
    }
}