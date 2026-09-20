#![no_std]

use kapi_abi::{DsCmd, DsError, Handle};
use kapi_syscall::sys_ipc_call;

pub struct MmioRegion {
    pub phys_base: u64,
    pub virt_base: u64,
    pub size: u64,
    pub handle: Handle,
}

impl MmioRegion {
    pub fn map(manager_ep: Handle, phys_base: u64, size: u64) -> Result<Self, DsError> {
        let reply = sys_ipc_call(DsCmd::MemMapMmio, phys_base, size, 0);
        if reply.is_ok() {
            Ok(Self { phys_base, virt_base: reply.arg0, size, handle: manager_ep })
        } else {
            Err(reply.error_status())
        }
    }
}

impl Drop for MmioRegion {
    fn drop(&mut self) {
        let _ = sys_ipc_call(DsCmd::MemUnmapMmio, self.virt_base, self.size, 0);
    }
}

pub struct Volatile<T>(pub T);

impl<T: Copy> Volatile<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(&self.0) }
    }

    #[inline(always)]
    pub fn write(&mut self, value: T) {
        unsafe { core::ptr::write_volatile(&mut self.0, value); }
    }
}