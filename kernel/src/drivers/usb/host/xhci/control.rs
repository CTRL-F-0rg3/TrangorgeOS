use super::init::Xhci;
use super::trb::*;
use crate::drivers::usb::usbcore::device::UsbDevice;
use crate::drivers::usb::UsbError;

pub fn wait_transfer(x: &mut Xhci, slot: u8) -> Result<u8, UsbError> {
    for _ in 0..2_000_000 {
        if let Some(t) = x.ev.pending() {
            let t = t;
            x.ev.pop();
            x.regs.rt_write(super::init::RT_ERDP, x.ev.erdp());

            if t.typ() == TRB_TRANSFER_EVENT && t.slot_id() == slot {
                return Ok(t.completion_code());
            }

            continue;
        }

        core::hint::spin_loop();
    }

    Err(UsbError::Timeout)
}

pub fn wait_transfer_ep(x: &mut Xhci, slot: u8, ep: u8) -> Result<u8, UsbError> {
    for _ in 0..2_000_000 {
        if let Some(t) = x.ev.pending() {
            let t = t;
            x.ev.pop();
            x.regs.rt_write(super::init::RT_ERDP, x.ev.erdp());

            if t.typ() == TRB_TRANSFER_EVENT && t.slot_id() == slot && t.ep_id() == ep {
                return Ok(t.completion_code());
            }

            continue;
        }

        core::hint::spin_loop();
    }

    Err(UsbError::Timeout)
}

pub fn control(x: &mut Xhci,
               dev: &mut UsbDevice,
               setup: u64,
               trt: u32,
               data: Option<(&mut [u8], usize)>,
               out_len: usize) -> Result<usize, UsbError> {
    dev.ep0.enqueue(Trb::setup_stage(setup, trt));

    let (dir_in, len) = match &data {
        Some((_, l)) => (trt == 3, *l),
        None => (false, 0),
    };

    if let Some((buf, l)) = data {
        if !dir_in && l > 0 {
            unsafe {
                core::ptr::copy_nonoverlapping(buf.as_ptr(), dev.data.virt, l);
            }
        }

        dev.ep0.enqueue(Trb::data_stage(dev.data.phys, l as u32, dir_in));
    }

    let _ = out_len;

    dev.ep0.enqueue(Trb::status_stage(!dir_in));

    x.regs.doorbell(dev.slot as u32, 1, 0);

    let cc = wait_transfer(x, dev.slot)?;

    if cc != CC_SUCCESS {
        return Err(UsbError::Transfer(cc));
    }

    if let Some((buf, l)) = data {
        if dir_in {
            unsafe {
                core::ptr::copy_nonoverlapping(dev.data.virt, buf.as_mut_ptr(), l);
            }
        }

        return Ok(l);
    }

    Ok(0)
}

pub fn control_in(x: &mut Xhci, dev: &mut UsbDevice,
                  req: u8, value: u16, index: u16,
                  buf: &mut [u8]) -> Result<usize, UsbError> {
    let setup = pack_setup(0x80, req, value, index, buf.len() as u16);
    control(x, dev, setup, 3, Some((buf, buf.len())), 0)
}

pub fn control_out(x: &mut Xhci, dev: &mut UsbDevice,
                   req: u8, value: u16, index: u16,
                   data: &[u8]) -> Result<usize, UsbError> {
    let setup = pack_setup(0x00, req, value, index, data.len() as u16);

    if data.is_empty() {
        control(x, dev, setup, 0, None, 0)
    } else {
        let mut tmp = [0u8; 512];
        tmp[..data.len()].copy_from_slice(data);
        control(x, dev, setup, 2, Some((&mut tmp, data.len())), 0)
    }
}