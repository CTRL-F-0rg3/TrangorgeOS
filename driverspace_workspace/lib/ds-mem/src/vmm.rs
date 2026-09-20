#![no_std]

use kapi_abi::{DsCmd, DsError, Handle};
use kapi_syscall::sys_ipc_call;

pub fn map_memory(
    _manager_ep: Handle,
    phys_addr: u64,
    _virt_addr: u64,
    size: u64,
    flags: u64,
) -> Result<(), DsError> {
    let reply = sys_ipc_call(DsCmd::MemMapMmio, phys_addr, size, flags);
    if reply.is_ok() { Ok(()) } else { Err(reply.error_status()) }
}

pub fn unmap_memory(_manager_ep: Handle, virt_addr: u64, size: u64) -> Result<(), DsError> {
    let reply = sys_ipc_call(DsCmd::MemUnmapMmio, virt_addr, size, 0);
    if reply.is_ok() { Ok(()) } else { Err(reply.error_status()) }
}