use kapi_abi::primitives::{Handle, PhysAddr, VirtAddr};
use kapi_abi::errors::DsError;
use kapi_abi::opcodes::Opcode;
use kapi_abi::payloads::mem::DmaAllocPayload;
use kapi_syscall::sys_ipc_call;
use ds_log::ds_error;
use bitflags::bitflags;

bitflags! {
    pub struct DmaFlags: u32 {
        const COHERENT   = 1 << 0;
        const HIGH_MEM   = 1 << 1;
        const CONTIGUOUS = 1 << 2;
    }
}

pub struct DmaBuffer {
    pub phys_addr: PhysAddr,
    pub virt_addr: VirtAddr,
    pub size: u64,
    pub handle: Handle,
}

impl DmaBuffer {
    pub fn allocate(manager_ep: Handle, size: u64, flags: DmaFlags) -> Result<Self, DsError> {
        let mut payload = DmaAllocPayload {
            size,
            flags: flags.bits(),
            _pad: 0,
        };

        let mut reply_buf = [0u64; 3];

        let res = unsafe {
            sys_ipc_call(
                manager_ep.0,
                Opcode::MemAllocDma as u32,
                &mut payload as *mut _ as *mut u8,
                core::mem::size_of::<DmaAllocPayload>() as u32,
                reply_buf.as_mut_ptr() as *mut u8,
                core::mem::size_of_val(&reply_buf) as u32,
            )
        };

        match res {
            Ok(_) => Ok(Self {
                phys_addr: PhysAddr(reply_buf[0]),
                virt_addr: VirtAddr(reply_buf[1]),
                size,
                handle: Handle(reply_buf[2] as u32),
            }),
            Err(e) => {
                ds_error!("Failed to allocate DMA buffer: {:?}", e);
                Err(e)
            }
        }
    }
}

impl Drop for DmaBuffer {
    fn drop(&mut self) {
        ds_log::ds_trace!("Dropping DMA buffer at {:?}", self.phys_addr);
        // TODO: IPC call to ds-manager to free the DMA buffer
    }
}