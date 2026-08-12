

use super::types::SystemInfo;

#[repr(C)]
pub struct KernelTable {
    pub magic: u64,
    pub abi_major: u32,
    pub abi_minor: u32,

    pub console_write: extern "C" fn(*const u8, usize),

    pub malloc: extern "C" fn(usize) -> *mut u8,
    pub free: extern "C" fn(*mut u8, usize),

    pub fs_open: extern "C" fn(*const u8, u32) -> i64,
    pub fs_read: extern "C" fn(u64, *mut u8, usize) -> i64,
    pub fs_write: extern "C" fn(u64, *const u8, usize) -> i64,
    pub fs_close: extern "C" fn(u64),

    pub get_system_info: extern "C" fn(*mut SystemInfo),
}

impl KernelTable {
    pub fn validate(&self) -> bool {
        self.magic == super::ABI_MAGIC
            && self.abi_major == super::ABI_VERSION_MAJOR
    }

    pub fn print(&self, s: &str) {
        (self.console_write)(s.as_ptr(), s.len());
    }
}