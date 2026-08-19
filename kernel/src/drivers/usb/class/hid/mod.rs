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

    for i in 0..dev.iface_count {
        let f = &dev.ifaces[i];

        if f.class == 3 && f.subclass == 1 && f.protocol == 1 {
            iface = f.number;
            found = true;
            break;
        }
    }

    if !found {
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

pub fn poll(x: &mut Xhci) {
    while let Some(t) = x.ev.pending() {
        let t = t;
        x.ev.pop();
        x.regs.rt_write(super::super::host::xhci::init::RT_ERDP, x.ev.erdp() as u32);

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