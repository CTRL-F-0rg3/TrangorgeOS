use kapi_abi::primitives::{Handle, PhysAddr, VirtAddr};
use kapi_abi::errors::DsError;
use kapi_abi::opcodes::Opcode;
use kapi_abi::payloads::mem::MmioMapPayload;
use kapi_syscall::sys_ipc_call;
use ds_log::ds_error;

pub struct MmioRegion {
    pub phys_base: PhysAddr,
    pub virt_base: VirtAddr,
    pub size: u64,
    pub handle: Handle,
}

impl MmioRegion {
    pub fn map(manager_ep: Handle, phys_base: u64, size: u64) -> Result<Self, DsError> {
        let mut payload = MmioMapPayload {
            phys_addr: PhysAddr(phys_base),
            size,
            flags: 0,
            _pad: 0,
        };

        let mut reply_buf = [0u64; 2];

        let res = unsafe {
            sys_ipc_call(
                manager_ep.0,
                Opcode::MemMapMmio as u32,
                &mut payload as *mut _ as *mut u8,
                core::mem::size_of::<MmioMapPayload>() as u32,
                reply_buf.as_mut_ptr() as *mut u8,
                core::mem::size_of_val(&reply_buf) as u32,
            )
        };

        match res {
            Ok(_) => Ok(Self {
                phys_base: PhysAddr(phys_base),
                virt_base: VirtAddr(reply_buf[0]),
                size,
                handle: Handle(reply_buf[1] as u32),
            }),
            Err(e) => {
                ds_error!("Failed to map MMIO region: {:?}", e);
                Err(e)
            }
        }
    }

    #[inline(always)]
    pub unsafe fn read32(&self, offset: u64) -> u32 {
        debug_assert!(offset + 4 <= self.size, "MMIO read out of bounds");
        let ptr = (self.virt_base.0 + offset) as *const Volatile<u32>;
        (*ptr).read()
    }

    #[inline(always)]
    pub unsafe fn write32(&self, offset: u64, value: u32) {
        debug_assert!(offset + 4 <= self.size, "MMIO write out of bounds");
        let ptr = (self.virt_base.0 + offset) as *mut Volatile<u32>;
        (*ptr).write(value);
    }
}

#[repr(transparent)]
pub struct Volatile<T>(pub T);

impl<T: Copy> Volatile<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        core::ptr::read_volatile(&self.0)
    }

    #[inline(always)]
    pub fn write(&mut self, value: T) {
        core::ptr::write_volatile(&mut self.0, value);
    }
}