#![no_std]

use kapi_abi::{DsCmd, DsError, Handle};
use kapi_syscall::sys_ipc_call;

pub struct ManagerClient {
    manager_ep: Handle,
}

impl ManagerClient {
    pub const fn new(ep: Handle) -> Self {
        Self { manager_ep: ep }
    }

    pub fn request_mmio(&self, phys_base: u64, size: u64) -> Result<u64, DsError> {
        let reply = sys_ipc_call(DsCmd::MemMapMmio, phys_base, size, 0);
        if reply.is_ok() { Ok(reply.arg0) } else { Err(reply.error_status()) }
    }

    pub fn bind_irq(&self, irq_num: u32) -> Result<(), DsError> {
        let reply = sys_ipc_call(DsCmd::BindIrq, irq_num as u64, 0, 0);
        if reply.is_ok() { Ok(()) } else { Err(reply.error_status()) }
    }

    pub fn alloc_dma(&self, size: u64, flags: u32) -> Result<(u64, u64), DsError> {
        let reply = sys_ipc_call(DsCmd::MemAllocDma, size, flags as u64, 0);
        if reply.is_ok() { Ok((reply.arg0, reply.arg1)) } else { Err(reply.error_status()) }
    }

    pub fn pci_find(&self, class_code: u32) -> Result<u64, DsError> {
        let reply = sys_ipc_call(DsCmd::PciFind, class_code as u64, 0, 0);
        if reply.is_ok() { Ok(reply.arg0) } else { Err(reply.error_status()) }
    }
    
    pub fn get_page_phys(&self, virt_addr: u64) -> Result<u64, DsError> {
        let reply = sys_ipc_call(DsCmd::PagePhys, virt_addr, 0, 0);
        if reply.is_ok() { Ok(reply.arg0) } else { Err(reply.error_status()) }
    }
}