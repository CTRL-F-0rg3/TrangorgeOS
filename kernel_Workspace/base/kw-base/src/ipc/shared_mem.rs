use kstd_base::{PhysAddr, VirtAddr, Size};
use crate::core::{KResult, KernelError};
use crate::mem::{ContiguousFrames, paging};

pub struct SharedMemory {
    frames: ContiguousFrames,
    virt: VirtAddr,
}

impl SharedMemory {
    pub fn allocate(pages: usize) -> KResult<Self> {
        let frames = ContiguousFrames::alloc(pages)?;
        let virt = gluecore::bridge::mm::vmm::alloc(
            frames.byte_size(), 
            gluecore::bridge::mm::vmm::VMM_FLAG_READ | gluecore::bridge::mm::vmm::VMM_FLAG_WRITE
        ).map_err(KernelError::from)?;

        for i in 0..pages {
            let phys = PhysAddr(frames.phys_addr().0 + (i as u64 * 4096));
            let v = VirtAddr(virt.0 + (i as u64 * 4096));
            paging::map_page(v, phys, paging::PAGE_PRESENT | paging::PAGE_WRITABLE)?;
        }

        Ok(Self { frames, virt })
    }

    pub fn virt_addr(&self) -> VirtAddr { self.virt }
    pub fn size(&self) -> usize { self.frames.byte_size() }
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        let _ = gluecore::bridge::mm::vmm::free(self.virt, self.frames.byte_size());
    }
}