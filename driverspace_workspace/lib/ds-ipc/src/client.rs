#![no_std]

use kapi_abi::{DsCmd, DsMsg, Status, CapId};
use kapi_syscall;

pub struct ManagerClient;

impl ManagerClient {
    pub fn request_mmio(phys_base: u64, size: u64) -> Result<u64, Status> {
        let reply = kapi_syscall::sys_ipc_call(DsCmd::ReqMmio, phys_base, size, 0);
        
        if reply.is_ok() {
            Ok(reply.arg0)
        } else {
            Err(reply.error_status())
        }
    }

    pub fn bind_irq(irq_num: u32) -> Result<(), Status> {
        let reply = kapi_syscall::sys_ipc_call(DsCmd::ReqIrq, irq_num as u64, 0, 0);
        
        if reply.is_ok() {
            Ok(())
        } else {
            Err(reply.error_status())
        }
    }

    pub fn alloc_dma(size: u64, flags: u32) -> Result<(u64, u64), Status> {
        let reply = kapi_syscall::sys_ipc_call(DsCmd::ReqDma, size, flags as u64, 0);
        
        if reply.is_ok() {
            Ok((reply.arg0, reply.arg1))
        } else {
            Err(reply.error_status())
        }
    }

    pub fn pci_find(class_code: u32) -> Result<u64, Status> {
        let reply = kapi_syscall::sys_ipc_call(DsCmd::PciFind, class_code as u64, 0, 0);
        
        if reply.is_ok() {
            Ok(reply.arg0)
        } else {
            Err(reply.error_status())
        }
    }
    
    pub fn get_page_phys(virt_addr: u64) -> Result<u64, Status> {
        let reply = kapi_syscall::sys_ipc_call(DsCmd::PagePhys, virt_addr, 0, 0);
        
        if reply.is_ok() {
            Ok(reply.arg0)
        } else {
            Err(reply.error_status())
        }
    }
}