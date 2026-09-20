pub mod hid;
pub mod mass;

use crate::core::device::UsbDevice;

pub trait ClassDriver {
    fn probe(&self, dev: &UsbDevice) -> bool;
}
