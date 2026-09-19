use crate::core::{PhysAddr, KResult, KernelError};
use gluecore::bridge::mm::pmm;

pub struct ContiguousFrames {
    addr: PhysAddr,
    count: usize,
}

impl ContiguousFrames {
    pub fn alloc(count: usize) -> KResult<Self> {
        let addr = pmm::alloc_frames(count).map_err(KernelError::from)?;
        Ok(Self { addr, count })
    }

    pub fn alloc_aligned(count: usize, align_frames: usize) -> KResult<Self> {
        let addr = pmm::alloc_frames_aligned(count, align_frames).map_err(KernelError::from)?;
        Ok(Self { addr, count })
    }

    pub fn phys_addr(&self) -> PhysAddr { self.addr }
    pub fn frame_count(&self) -> usize { self.count }
    pub fn byte_size(&self) -> usize { self.count * 4096 }

    pub fn into_raw(self) -> (PhysAddr, usize) {
        let addr = self.addr;
        let count = self.count;
        core::mem::forget(self);
        (addr, count)
    }

    pub unsafe fn from_raw(addr: PhysAddr, count: usize) -> Self {
        Self { addr, count }
    }
}

impl Drop for ContiguousFrames {
    fn drop(&mut self) {
        let _ = pmm::free_frames(self.addr, self.count);
    }
}