use super::UsbError;
use crate::drivers::pci;

pub const XHCI_CLASS: u8 = 0x0C;
pub const XHCI_SUBCLASS: u8 = 0x03;

/// A prog-if value that does mean "xHCI", kept for reporting only.
///
/// Not used to *match*. See [`find_xhci`].
pub const XHCI_PROGIF: u8 = 0x30;

pub struct XhciPci {
    pub dev: pci::PciDev,
    pub bar0_phys: u64,
}

/// Find the xHCI host controller.
///
/// # Why the prog-if is not matched
///
/// This used to require class 0x0C, subclass 0x03 *and* prog-if 0x30, and that
/// third field is what made the driver miss controllers. Prog-if is a revision
/// hint, not an identity: QEMU's `qemu-xhci` reports 1b36:000d with no such
/// prog-if, and real hardware is not consistent about it either. Requiring it
/// meant a machine with a perfectly good controller — the emulator, among
/// others — was reported as having no USB at all, and since nothing then drives
/// the HID class, a keyboard plugged into that controller is invisible. The
/// class and subclass pair already identify xHCI on its own; the prog-if is
/// printed when the controller is found so the revision is still visible.
pub fn find_xhci() -> Result<XhciPci, UsbError> {
    let dev = pci::find_class_subclass(XHCI_CLASS, XHCI_SUBCLASS).ok_or_else(|| {
        // Name every USB-class device actually present. "No controller" and
        // "no controller we can drive" are different faults — a machine with an
        // EHCI controller is not lacking USB, it has a generation this driver
        // does not speak — and the class triple is what tells them apart.
        let found = pci::find_all_by_class(XHCI_CLASS);
        if found.is_empty() {
            crate::println!("[usb] no USB host controller on the PCI bus");
        } else {
            for d in found {
                crate::println!(
                    "[usb] USB controller at {:02x}:{:02x}.{:x} class={:02x}/{:02x}/{:02x} (not xHCI)",
                    d.bus, d.dev, d.func, d.class(), d.subclass(), d.prog_if()
                );
            }
        }
        UsbError::NoController
    })?;

    let bar0 = dev.bar(0);

    if bar0 == 0 {
        return Err(UsbError::NoController);
    }

    dev.enable_mmio();

    Ok(XhciPci { dev, bar0_phys: bar0 })
}
