use kstd_base::{PhysAddr, VirtAddr, Size, Status};

#[repr(u32)]
pub enum DmaDirection {
    ToDevice = 1,
    FromDevice = 2,
    Bidirectional = 0,
}

// Linux DMA API shim. 
// Under the hood, this must request physical grants from the kernel via DriverSpace IPC.
pub struct DmaMapping {
    pub virt: VirtAddr,
    pub phys: PhysAddr,
    pub size: Size,
}

impl DmaMapping {
    // Equivalent to dma_map_single
    pub fn map_single(_cpu_addr: VirtAddr, _size: Size, _dir: DmaDirection) -> Result<PhysAddr, Status> {
        // TODO: Call native DMA grant IPC
        Err(Status::NotSupported)
    }

    // Equivalent to dma_unmap_single
    pub fn unmap_single(_phys: PhysAddr, _size: Size, _dir: DmaDirection) {
        // TODO: Revoke DMA grant IPC
    }
}