pub mod hid;
pub mod mass;
use crate::drivers::usb::usbcore::device::UsbDevice;

pub trait ClassDriver {
    fn probe(&self, dev: &UsbDevice) -> bool;
}