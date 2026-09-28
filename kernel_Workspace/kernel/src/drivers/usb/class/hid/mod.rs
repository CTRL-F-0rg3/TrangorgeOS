pub mod keyboard;
pub mod report;

use crate::drivers::usb::host::xhci::control;
use crate::drivers::usb::host::xhci::init::Xhci;
use crate::drivers::usb::host::xhci::trb::*;
use crate::drivers::usb::host::xhci::ring::TransferRing;
use crate::drivers::usb::dma::DmaBuf;
use crate::drivers::usb::core::device::UsbDevice;
use crate::drivers::usb::core::speed::EP_INTERRUPT;
use crate::drivers::usb::UsbError;

// `kprintf` is a C variadic, and Rust will not promote a `u8` through `...`:
// the callee reads it as a full word. The cast is what makes the value mean
// what the format string claims.
use core::ffi::c_uint;

extern "C" {
    fn kprintf(fmt: *const u8, ...);
}

const SET_IDLE: u8 = 0x0A;
const SET_PROTOCOL: u8 = 0x0B;

pub struct HidKeyboard {
    pub slot: u8,
    pub ep_idx: u32,
    ring: TransferRing,
    data: DmaBuf,
    prev: [u8; 6],
}

static mut KEYS: [Option<HidKeyboard>; 2] = [None, None];

fn submit(x: &mut Xhci, kb: &mut HidKeyboard) {
    kb.ring.enqueue(Trb::normal(kb.data.phys, 8));
    x.regs.doorbell(kb.slot as u32, kb.ep_idx, 0);
}

pub fn attach(x: &mut Xhci, dev: &mut UsbDevice) -> Result<bool, UsbError> {
    let mut iface = 0u8;
    let mut found = false;

    // Report what was actually on the wire, not just whether it matched. The
    // match test below is narrow — boot keyboard only — and "no USB keyboard
    // found" downstream is indistinguishable between "there is no keyboard" and
    // "there is a keyboard and we did not recognise it". Those need different
    // fixes, so the class triple each interface actually carried is printed
    // before the decision is made.
    let mut ifaces_seen = 0u8;
    for i in 0..dev.iface_count {
        let f = &dev.ifaces[i];
        unsafe {
            kprintf(b"usb: hid: iface %d class=%d subclass=%d protocol=%d\n\0".as_ptr(),
                    f.number as c_uint, f.class as c_uint,
                    f.subclass as c_uint, f.protocol as c_uint);
        }
        ifaces_seen += 1;
    }
    if ifaces_seen == 0 {
        unsafe { kprintf(b"usb: hid: device reported no interfaces at all\n\0".as_ptr()); }
    }

    for i in 0..dev.iface_count {
        let f = &dev.ifaces[i];

        if f.class == 3 && f.subclass == 1 && f.protocol == 1 {
            iface = f.number;
            found = true;
            break;
        }
    }

    if !found {
        // A HID device that is not a *boot* keyboard still types: it is just
        // driven by the report's own keycodes rather than by this table. Saying
        // so is the difference between "attach this differently" and "buy a
        // different keyboard".
        let hid_other = (0..dev.iface_count).any(|i| dev.ifaces[i].class == 3);
        unsafe {
            kprintf(if hid_other {
                b"usb: hid: HID present but not a boot keyboard (protocol != 1)\n\0".as_ptr()
            } else {
                b"usb: hid: no HID interface; not a keyboard\n\0".as_ptr()
            });
        }
        return Ok(false);
    }

    let mut ep_num = 0u8;
    let mut mps = 8u16;
    let mut interval = 10u8;
    let mut have_ep = false;

    for i in 0..dev.ep_count {
        let e = &dev.eps[i];

        if e.attributes & 0x03 == EP_INTERRUPT && e.address & 0x80 != 0 {
            ep_num = e.address & 0x0F;
            mps = e.max_packet;
            interval = e.interval;
            have_ep = true;
            break;
        }
    }

    if !have_ep {
        return Ok(false);
    }

    let ep_idx = (2 * ep_num + 1) as u32;

    control::control_out(x, dev, SET_IDLE, 0, iface as u16, &[])?;
    control::control_out(x, dev, SET_PROTOCOL, 0, iface as u16, &[])?;

    let ring = TransferRing::new(16)?;
    let data = DmaBuf::new(64)?;

    dev.ctx.setup_configure_ep(dev.speed, dev.port, ep_idx, 7,
                               mps, interval as u32, ring.phys());

    x.command(Trb::configure_ep(dev.slot, dev.ctx.input.phys))?;

    let mut kb = HidKeyboard {
        slot: dev.slot,
        ep_idx,
        ring,
        data,
        prev: [0u8; 6],
    };

    submit(x, &mut kb);

    unsafe {
        for slot in KEYS.iter_mut() {
            if slot.is_none() {
                *slot = Some(kb);
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Whether a USB keyboard is attached and driven.
///
/// The input layer uses this to decide what to tell the user, and — more
/// importantly — to know whether PS/2 is the *only* keyboard there is. A USB
/// keyboard is optional hardware: on a machine without one, PS/2 is not a
/// fallback but the sole input path.
pub fn keyboard_attached() -> bool {
    unsafe { KEYS.iter().any(|k| k.is_some()) }
}

pub fn poll(x: &mut Xhci) {
    while let Some(t) = x.ev.pending() {
        let t = t;
        x.ev.pop();
        // ERDP is 64 bits. The old 32-bit `rt_write(.., erdp() as u32)` truncated
        // the physical address of the event ring, which is harmless for a DMA
        // buffer that happens to land below 4 GiB and silently fatal for one that
        // does not — the controller would keep posting events the host reads
        // from a different segment.
        super::super::host::xhci::init::rt_write64(
            &x.regs,
            super::super::host::xhci::init::RT_ERDP,
            x.ev.erdp(),
        );

        if t.typ() != TRB_TRANSFER_EVENT {
            continue;
        }

        let slot = t.slot_id();
        let ep = t.ep_id();

        unsafe {
            for kb in KEYS.iter_mut() {
                let kb = match kb {
                    Some(k) if k.slot == slot && k.ep_idx == ep as u32 => k,
                    _ => continue,
                };

                if let Some(rep) = report::parse_boot_keyboard(
                    core::slice::from_raw_parts(kb.data.virt, 8)) {
                    for key in rep.keys.iter() {
                        if *key == 0 {
                            continue;
                        }

                        if !kb.prev.contains(key) {
                            if let Some(c) = keyboard::key_to_ascii(*key, rep.shift) {
                                keyboard::push_char(c);
                            }
                        }
                    }

                    kb.prev = rep.keys;
                }

                submit(x, kb);
            }
        }
    }
}
