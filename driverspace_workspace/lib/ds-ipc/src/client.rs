//! IPC Client for drivers to communicate with ds-manager.

use kapi_abi::opcodes::Opcode;
use kapi_abi::errors::DsError;
use kapi_abi::primitives::Handle;

pub struct ManagerClient {
    manager_ep: Handle,
}

impl ManagerClient {
    pub const fn new(ep: Handle) -> Self {
        Self { manager_ep: ep }
    }

    pub fn request_mmio(&self, phys_base: u64, size: u64) -> Result<u64, DsError> {
        let mut payload = kapi_abi::payloads::mem::MmioMapPayload {
            phys_addr: kapi_abi::primitives::PhysAddr(phys_base),
            size,
            flags: 0,
            _pad: 0,
        };

        let mut reply_buf = [0u64; 2];
        
        let res = kapi_syscall::sys_ipc_call(
            self.manager_ep.0,
            Opcode::MemMapMmio as u32,
            &mut payload as *mut _ as *mut u8,
            core::mem::size_of_val(&payload) as u32,
            reply_buf.as_mut_ptr() as *mut u8,
            core::mem::size_of_val(&reply_buf) as u32,
        );

        match res {
            Ok(_) => Ok(reply_buf[0]), // Returns virtual address
            Err(e) => Err(e),
        }
    }

    pub fn bind_irq(&self, irq_num: u32) -> Result<(), DsError> {
        let payload = kapi_abi::payloads::irq::IrqBindPayload {
            irq_number: irq_num,
            flags: 0,
            _pad: 0,
        };

        kapi_syscall::sys_ipc_send(
            self.manager_ep.0,
            Opcode::IrqBind as u32,
            &payload as *const _ as *const u8,
            core::mem::size_of_val(&payload) as u32,
        )
    }
}