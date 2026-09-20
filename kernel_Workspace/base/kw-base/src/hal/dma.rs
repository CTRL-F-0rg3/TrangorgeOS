use kstd_base::{PhysAddr, VirtAddr, Size};
use crate::core::{KResult, KernelError};
use crate::mem::ContiguousFrames;

pub struct DmaBuffer {
    frames: ContiguousFrames,
    virt: VirtAddr,
}

impl DmaBuffer {
    pub fn allocate(pages: usize) -> KResult<Self> {
        let frames = ContiguousFrames::alloc(pages)?;
        let virt = gluecore::bridge::mm::vmm::alloc(
            frames.byte_size(),
            gluecore::bridge::mm::vmm::VMM_FLAG_READ | gluecore::bridge::mm::vmm::VMM_FLAG_WRITE | gluecore::bridge::mm::vmm::VMM_FLAG_DEVICE
        ).map_err(KernelError::from)?;

        Ok(Self { frames, virt })
    }

    pub fn phys_addr(&self) -> PhysAddr { self.frames.phys_addr() }
    pub fn virt_addr(&self) -> VirtAddr { self.virt }
    pub fn size(&self) -> usize { self.frames.byte_size() }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.virt.0 as *const u8, self.size()) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.virt.0 as *mut u8, self.size()) }
    }
}