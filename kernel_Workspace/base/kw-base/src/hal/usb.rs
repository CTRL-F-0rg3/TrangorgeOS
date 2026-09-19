use crate::core::{KResult, KernelError};

extern "C" {
    fn k_usb_init() -> bool;
    fn k_usb_enum_devices() -> u32;
    fn k_usb_get_device_count() -> u32;
}

pub fn init() -> KResult<()> {
    if unsafe { k_usb_init() } {
        Ok(())
    } else {
        Err(KernelError::HardwareFault)
    }
}

pub fn enumerate_devices() -> KResult<u32> {
    let count = unsafe { k_usb_enum_devices() };
    if count == 0 && unsafe { k_usb_get_device_count() } == 0 {
        Err(KernelError::NotFound)
    } else {
        Ok(count)
    }
}