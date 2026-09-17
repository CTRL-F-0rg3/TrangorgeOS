pub mod host;
pub mod pci_glue;
pub mod dma;
pub mod core;
pub mod class;

use host::xhci::init::Xhci;

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

static mut CONTROLLER: Option<Xhci> = None;

pub fn init() -> Result<(), UsbError> {
    if unsafe { CONTROLLER.is_some() } {
        return Ok(());
    }

    let xhci_pci = pci_glue::find_xhci()?;

    crate::println!(
        "[usb] xHCI controller: bus={} dev={} func={} bar0={:#x}",
        xhci_pci.dev.bus,
        xhci_pci.dev.dev,
        xhci_pci.dev.func,
        xhci_pci.bar0_phys
    );

    let regs = host::xhci::regs::XhciRegs::new(xhci_pci.bar0_phys)?;

    crate::println!(
        "[usb] xHCI caps: max_slots={} max_ports={} max_intrs={} addr64={} csz64={}",
        regs.max_slots,
        regs.max_ports,
        regs.max_intrs,
        regs.addr64,
        regs.csz64
    );

    let mut xhci = host::xhci::init::init(regs)?;

    xhci.scan_ports();

    unsafe {
        CONTROLLER = Some(xhci);
    }

    Ok(())
}

pub fn poll() {
    if let Some(x) = unsafe { CONTROLLER.as_mut() } {
        class::hid::poll(x);
    }
}

pub fn self_test() -> TestResult {
    match init() {
        Ok(()) => Ok("xHCI initialized OK"),
        Err(UsbError::NoController) => {
            crate::println!("[usb] no xHCI controller found (PCI 0x0C/0x03/0x30)");
            Err("no xHCI controller")
        }
        Err(UsbError::MapFailed) => Err("xHCI MMIO map failed"),
        Err(UsbError::Timeout) => Err("xHCI init timeout"),
        Err(UsbError::Invalid) => Err("xHCI init invalid"),
        Err(_) => Err("xHCI init failed"),
    }
}