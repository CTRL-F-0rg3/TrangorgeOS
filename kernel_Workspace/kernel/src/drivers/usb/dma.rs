use super::UsbError;
use crate::mm::ffi;
use core::ffi::c_void;

pub struct DmaBuf {
    pub phys: u64,
    pub virt: *mut u8,
    pub len: usize,
}

impl DmaBuf {
    pub fn new(len: usize) -> Result<Self, UsbError> {
        let mut phys = 0u64;
        let mut virt: *mut c_void = core::ptr::null_mut();

        let ok = unsafe {
            ffi::dma_alloc_coherent(len, 0xFFFF_FFFF, &mut phys, &mut virt)
        };

        if !ok || virt.is_null() {
            return Err(UsbError::Invalid);
        }

        Ok(Self {
            phys,
            virt: virt as *mut u8,
            len,
        })
    }

    pub fn zero(&mut self) {
        unsafe { core::ptr::write_bytes(self.virt, 0, self.len) }
    }
}

impl Drop for DmaBuf {
    fn drop(&mut self) {
        unsafe {
            ffi::dma_free_coherent(self.phys, self.virt as *mut c_void, self.len);
        }
    }
}
