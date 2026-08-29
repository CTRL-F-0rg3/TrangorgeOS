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

/// The one live xHCI controller, if any.
///
/// Previously the `Xhci` built by `host::xhci::init::init()` only ever
/// existed as a local variable inside `self_test()` and was dropped the
/// instant that function returned - so nothing could ever poll it again
/// after boot, and a plugged-in USB keyboard would never produce a single
/// keystroke no matter how correctly the rest of the stack worked.
///
/// This is its real persistent home. Kept as a bare `static mut` (matching
/// the convention already used for device state elsewhere in this driver -
/// see `class::mass::MASS0`, `class::hid::KEYS`) rather than
/// `spin::Mutex<Xhci>`, since `Xhci` holds raw MMIO/DMA pointers that
/// aren't `Send`, and this kernel doesn't run USB driver code concurrently
/// from more than one context.
static mut CONTROLLER: Option<Xhci> = None;

/// Brings up the xHCI controller if one is present, scans every port for a
/// connected device, and hands each one to whichever class driver claims
/// it (mass storage first, then HID keyboard). Idempotent - a second call
/// is a cheap no-op if a controller is already up.
///
/// This is the function that actually needs to run once at boot for USB
/// keyboard input and USB mass storage to work at all; `self_test()` below
/// is now just a thin diagnostic wrapper around this same path so the test
/// harness still reports pass/fail the way it always did.
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

    // Detects connected ports, resets/enables them, reads their
    // descriptors, and attaches a class driver (mass storage / HID) to
    // each one. See host::xhci::init::Xhci::scan_ports /
    // Xhci::attach_port - previously this only detected and reset ports
    // and never went any further.
    xhci.scan_ports();

    unsafe {
        CONTROLLER = Some(xhci);
    }

    Ok(())
}

/// Services pending USB interrupt-transfer completions - currently, HID
/// keyboard reports (see `class::hid::poll`). Must be called repeatedly
/// after `init()` for `class::hid::keyboard::take_char()` to ever produce
/// a keystroke; it's wired into the timer interrupt in `interrupts.rs`.
/// Safe (and cheap) to call even if no controller was ever brought up.
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