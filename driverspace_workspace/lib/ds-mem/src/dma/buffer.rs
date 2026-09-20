#![no_std]

use kapi_abi::{DsCmd, DsError, Handle};
use kapi_syscall::sys_ipc_call;

#[derive(Debug, Clone, Copy)]
pub struct DmaFlags(pub u32);

impl DmaFlags {
    pub const COHERENT: Self = Self(1 << 0);
    pub const HIGH_MEM: Self = Self(1 << 1);
    pub const CONTIGUOUS: Self = Self(1 << 2);
}

pub struct DmaBuffer {
    pub phys_addr: u64,
    pub virt_addr: u64,
    pub size: u64,
    pub handle: Handle,
}

impl DmaBuffer {
    pub fn allocate(manager_ep: Handle, size: u64, flags: DmaFlags) -> Result<Self, DsError> {
        let reply = sys_ipc_call(DsCmd::MemAllocDma, size, flags.0 as u64, 0);
        if reply.is_ok() {
            Ok(Self { phys_addr: reply.arg0, virt_addr: reply.arg1, size, handle: manager_ep })
        } else {
            Err(reply.error_status())
        }
    }
}

impl Drop for DmaBuffer {
    fn drop(&mut self) {
        let _ = sys_ipc_call(DsCmd::MemFreeDma, self.phys_addr, self.size, 0);
    }
}