use super::UsbError;
use crate::drivers::pci;

pub const XHCI_CLASS: u8 = 0x0C;
pub const XHCI_SUBCLASS: u8 = 0x03;
pub const XHCI_PROGIF: u8 = 0x30;

pub struct XhciPci {
    pub dev: pci::PciDev,
    pub bar0_phys: u64,
}

pub fn find_xhci() -> Result<XhciPci, UsbError> {
    let dev = pci::find_class(XHCI_CLASS, XHCI_SUBCLASS, XHCI_PROGIF)
        .ok_or(UsbError::NoController)?;

    let bar0 = dev.bar(0);

    if bar0 == 0 {
        return Err(UsbError::NoController);
    }

    dev.enable_mmio();

    Ok(XhciPci { dev, bar0_phys: bar0 })
}