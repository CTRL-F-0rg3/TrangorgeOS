
use crate::allocator::traits::{Frame, FrameAllocator, MapError, MapFlags, PhysAddr, VirtAddr, VirtualMapper};
use super::mapper::PageMapper;

pub struct AddressSpace {
    mapper: PageMapper,
}

impl AddressSpace {
    /// `pml4_phys` musi wskazywać na już zaalokowaną i wyzerowaną ramkę.
    pub fn new(pml4_frame: Frame, pml4_phys: PhysAddr, phys_to_virt_offset: u64) -> Self {
        Self {
            mapper: PageMapper::new(pml4_frame, pml4_phys, phys_to_virt_offset),
        }
    }

    /// Mapuje region `[virt, virt + size)`, alokując dla niego świeże ramki.
    pub unsafe fn map_region(
        &mut self,
        virt: VirtAddr,
        size: usize,
        flags: MapFlags,
        frame_alloc: &mut dyn FrameAllocator,
    ) -> Result<(), MapError> {
        let page_size = crate::allocator::config::PAGE_SIZE as u64;
        let pages = (size as u64 + page_size - 1) / page_size;

        for i in 0..pages {
            let page_virt = VirtAddr::new(virt.as_u64() + i * page_size);
            let frame = frame_alloc.allocate_frame().ok_or(MapError::FrameAllocationFailed)?;
            let phys = PhysAddr::new((frame.0 as u64) * page_size);
            self.mapper.map(page_virt, phys, flags, frame_alloc)?;
        }
        Ok(())
    }

    pub fn translate(&self, virt: VirtAddr) -> Option<PhysAddr> {
        self.mapper.translate(virt)
    }

    pub fn mapper_mut(&mut self) -> &mut PageMapper {
        &mut self.mapper
    }
}