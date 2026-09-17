use crate::abi::DsError;

/// Opaque device identity handed to drivers during enumeration / attach.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DeviceInfo {
    pub device_id: u64,
    pub vendor_id: u64,
    pub class_code: u64,
}

/// Any driver that lives in driver space and is registered with the kernel
/// driver-space manager.
pub trait Driver {
    fn init(&mut self, info: &DeviceInfo) -> Result<(), DsError>;
}