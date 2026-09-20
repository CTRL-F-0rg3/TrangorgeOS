#![no_std]
#![allow(dead_code)]

//! `usb` — USB host controller (xHCI) + core device model + HID & mass-storage
//! class drivers, extracted from the legacy kernel (`kernel/src/drivers/usb`).
//!
//! Concrete actions exposed:
//!   * [`init`]  — probe PCI for an xHCI controller and bring it online
//!   * [`poll`]  — drain controller events and poll attached HID devices
//!
//! The network driver (`nic`) is intentionally **not** part of this set of
//! exported libraries and stays in the kernel.

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

static mut CONTROLLER: Option<Xhci> = None;

/// Probe the PCI bus for an xHCI controller and bring it online.
pub fn init() -> Result<(), UsbError> {
    if unsafe { CONTROLLER.is_some() } {
        return Ok(());
    }

    let xhci_pci = pci_glue::find_xhci()?;
    hw_sys::log::info("usb: xHCI controller found");

    let regs = host::xhci::regs::XhciRegs::new(xhci_pci.bar0_phys)?;

    let mut xhci = host::xhci::init::init(regs)?;
    xhci.scan_ports();

    unsafe {
        CONTROLLER = Some(xhci);
    }

    Ok(())
}

/// Poll attached HID devices and drain controller events.
pub fn poll() {
    if let Some(x) = unsafe { CONTROLLER.as_mut() } {
        class::hid::poll(x);
    }
}

/// Borrow the active controller (when it is online).
pub fn with_controller<F: FnOnce(&mut Xhci) -> R, R>(f: F) -> Option<R> {
    unsafe { CONTROLLER.as_mut().map(f) }
}
