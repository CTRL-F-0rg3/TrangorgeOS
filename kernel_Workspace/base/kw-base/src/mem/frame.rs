use crate::core::{PhysAddr, KResult, KernelError};
use gluecore::bridge::mm::pmm;

pub struct Frame {
    addr: PhysAddr,
}

impl Frame {
    pub fn alloc() -> KResult<Self> {
        let addr = pmm::alloc_frame().map_err(KernelError::from)?;
        Ok(Self { addr })
    }

    pub fn alloc_zeroed() -> KResult<Self> {
        let addr = pmm::alloc_zero_frame().map_err(KernelError::from)?;
        Ok(Self { addr })
    }

    pub fn phys_addr(&self) -> PhysAddr {
        self.addr
    }

    pub fn into_raw(self) -> PhysAddr {
        let addr = self.addr;
        core::mem::forget(self);
        addr
    }

    pub unsafe fn from_raw(addr: PhysAddr) -> Self {
        Self { addr }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        let _ = pmm::free_frame(self.addr);
    }
}