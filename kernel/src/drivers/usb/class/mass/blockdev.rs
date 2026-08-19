use super::{scsi, UsbMass};
use crate::fs::driver::block::{BlockDevice, DriverError};

impl BlockDevice for UsbMass {
    fn name(&self) -> &'static str {
        "usb0"
    }

    fn block_size(&self) -> usize {
        self.block_len as usize
    }

    fn block_count(&self) -> u64 {
        self.sectors as u64
    }

    fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError> {
        let m = unsafe { super::MASS0.as_mut().ok_or(DriverError::Io)? };

        super::with_controller(|x| {
            scsi::read10(x, m, block as u32, 1, m.data.phys)
        }).map_err(|_| DriverError::Io)?;

        unsafe {
            core::ptr::copy_nonoverlapping(m.data.virt, buf.as_mut_ptr(),
                                           m.block_len as usize);
        }

        Ok(())
    }

    fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError> {
        let m = unsafe { super::MASS0.as_mut().ok_or(DriverError::Io)? };

        unsafe {
            core::ptr::copy_nonoverlapping(buf.as_ptr(), m.data.virt,
                                           m.block_len as usize);
        }

        super::with_controller(|x| {
            scsi::write10(x, m, block as u32, 1, m.data.phys)
        }).map_err(|_| DriverError::Io)?;

        Ok(())
    }
}