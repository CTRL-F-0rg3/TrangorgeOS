use super::device::UsbDevice;
use super::descriptor;
use super::request::*;
use super::speed::default_ep0_mps;
use crate::drivers::usb::host::xhci::control;
use crate::drivers::usb::host::xhci::init::Xhci;
use crate::drivers::usb::host::xhci::trb::Trb;
use crate::drivers::usb::UsbError;

extern "C" {
    fn kprintf(fmt: *const u8, ...);
}

/// Run one step of enumeration, naming it if it fails.
///
/// USB enumeration is a chain of a dozen control transfers, and every one of
/// them returns the same opaque `UsbError`. Without this, a failure prints
/// "enumeration failed" and names no step at all, which is not enough to tell a
/// broken address-device command from a device that does not answer its first
/// descriptor request. The label is a byte string rather than a `&str` because
/// `kprintf` is a C variadic and wants a NUL-terminated literal.
macro_rules! step {
    ($label:literal, $expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => {
                unsafe {
                    kprintf(concat!("usb: enumerate: ", $label, " failed\n\0").as_ptr());
                }
                return Err(e);
            }
        }
    };
}

pub fn enumerate(x: &mut Xhci, port: u32) -> Result<UsbDevice, UsbError> {
    let slot = step!("enable_slot", x.command(Trb::enable_slot())).slot_id();
    let speed = x.regs.port_speed(port);

    let mut dev = UsbDevice::new(slot, speed, x.ctx_size)?;

    dev.port = port;

    let mps = default_ep0_mps(speed);
    dev.ep0_mps = mps;

    dev.ctx.setup_address_device(speed, port, mps, dev.ep0.phys());

    unsafe {
        (x.dcbaa.virt as *mut u64)
            .add(slot as usize)
            .write_volatile(dev.ctx.output.phys);
    }

    step!("address_device", x.command(Trb::address_device(slot, dev.ctx.input.phys)));

    let mut buf = [0u8; 256];

    step!("get_descriptor device 8",
          control::control_in(x, &mut dev, GET_DESCRIPTOR,
                              (DESC_DEVICE as u16) << 8, 0,
                              &mut buf[..8]));

    let real_mps = buf[7];

    if real_mps != 0 && real_mps as u16 != mps {
        dev.ep0_mps = real_mps as u16;
        dev.ctx.setup_evaluate_mps(real_mps as u16);
        step!("evaluate_ctx",
              x.command(Trb::evaluate_ctx(slot, dev.ctx.input.phys)));
    }

    step!("get_descriptor device 18",
          control::control_in(x, &mut dev, GET_DESCRIPTOR,
                              (DESC_DEVICE as u16) << 8, 0,
                              &mut buf[..18]));

    // The two `parse_*` helpers answer with an `Option`, not a `Result`, so they
    // do not go through `step!` — there is no error value to name, only the
    // absence of a parsed descriptor.
    let dd = descriptor::parse_device(&buf[..18]).ok_or(UsbError::BadDescriptor)?;

    dev.desc = dd;

    step!("get_descriptor config 9",
          control::control_in(x, &mut dev, GET_DESCRIPTOR,
                              (DESC_CONFIG as u16) << 8, 0,
                              &mut buf[..9]));

    let total = u16::from_le_bytes([buf[2], buf[3]]).min(256) as usize;

    step!("get_descriptor config total",
          control::control_in(x, &mut dev, GET_DESCRIPTOR,
                              (DESC_CONFIG as u16) << 8, 0,
                              &mut buf[..total]));

    let (cfg, _t, ni, ne) =
        descriptor::parse_config(&buf[..total], &mut dev.ifaces, &mut dev.eps)
            .ok_or(UsbError::BadDescriptor)?;

    dev.iface_count = ni;
    dev.ep_count = ne;
    dev.config_value = cfg;

    step!("set_configuration",
          control::control_out(x, &mut dev, SET_CONFIGURATION, cfg as u16, 0, &[]));

    unsafe {
        kprintf(b"usb: device slot=%d vendor=%x product=%d class=%d ifaces=%d\n\0".as_ptr(),
                slot as u32,
                dd.vendor as u32,
                dd.product as u32,
                dd.class as u32,
                ni as u32);
    }

    Ok(dev)
}