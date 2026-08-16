pub mod host;
pub mod pci_glue;
pub mod dma;
pub mod core;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbError {
    NoController,
    MapFailed,
    Timeout,
    NotReady,
    BadDescriptor,
    Invalid,
    Transfer(u8),
}

use crate::testing::TestResult;

/// Samotest USB: wykrycie kontrolera xHCI (PCI) i jego inicjalizacja.
pub fn self_test() -> TestResult {
    let xhci_pci = match pci_glue::find_xhci() {
        Ok(p) => p,
        Err(UsbError::NoController) => {
            crate::println!("[usb] no xHCI controller found (PCI 0x0C/0x03/0x30)");
            return Err("no xHCI controller");
        }
        Err(_) => return Err("xHCI PCI detection error"),
    };

    crate::println!(
        "[usb] xHCI controller: bus={} dev={} func={} bar0={:#x}",
        xhci_pci.dev.bus,
        xhci_pci.dev.dev,
        xhci_pci.dev.func,
        xhci_pci.bar0_phys
    );

    let regs = match host::xhci::regs::XhciRegs::new(xhci_pci.bar0_phys) {
        Ok(r) => r,
        Err(_) => return Err("xHCI MMIO map failed"),
    };

    crate::println!(
        "[usb] xHCI caps: max_slots={} max_ports={} max_intrs={} addr64={} csz64={}",
        regs.max_slots,
        regs.max_ports,
        regs.max_intrs,
        regs.addr64,
        regs.csz64
    );

    let mut xhci = match host::xhci::init::init(regs) {
        Ok(x) => x,
        Err(UsbError::Timeout) => return Err("xHCI init timeout"),
        Err(UsbError::Invalid) => return Err("xHCI init invalid"),
        Err(_) => return Err("xHCI init failed"),
    };

    xhci.scan_ports();

    Ok("xHCI initialized OK")
}
