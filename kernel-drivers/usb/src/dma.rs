use super::UsbError;

/// A DMA-coherent buffer (physical + virtual address).
pub struct DmaBuf {
    pub phys: u64,
    pub virt: *mut u8,
    pub len: usize,
}

impl DmaBuf {
    pub fn new(len: usize) -> Result<Self, UsbError> {
        let mut phys = 0u64;
        let mut virt: *mut u8 = core::ptr::null_mut();

        let ok = unsafe { hw_sys::dma_alloc_coherent(len, 0xFFFF_FFFF, &mut phys, &mut virt) };

        if !ok || virt.is_null() {
            return Err(UsbError::Invalid);
        }

        Ok(Self { phys, virt, len })
    }

    pub fn zero(&mut self) {
        unsafe { core::ptr::write_bytes(self.virt, 0, self.len) }
    }
}

impl Drop for DmaBuf {
    fn drop(&mut self) {
        unsafe { hw_sys::dma_free_coherent(self.phys, self.virt, self.len) };
    }
}
