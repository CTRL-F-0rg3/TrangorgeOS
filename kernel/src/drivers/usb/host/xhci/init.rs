use super::regs::*;
use super::ring::{CmdRing, EventRing};
use super::trb::*;
use crate::drivers::usb::dma::DmaBuf;
use crate::drivers::usb::UsbError;

extern "C" {
    fn kprintf(fmt: *const u8, ...);
}

pub const RT_IMAN: usize = 0x00;
pub const RT_IMOD: usize = 0x04;
pub const RT_ERSTSZ: usize = 0x08;
pub const RT_ERSTBA: usize = 0x10;
pub const RT_ERDP: usize = 0x18;

pub struct Xhci {
    pub regs: XhciRegs,
    pub cmd: CmdRing,
    pub ev: EventRing,
    pub dcbaa: DmaBuf,
    pub ctx_size: usize,
    pub slots: u32,
    pub ports: u32,
}

fn spin_wait<F: Fn() -> bool>(f: F) -> Result<(), UsbError> {
    for _ in 0..2_000_000 {
        if f() {
            return Ok(());
        }

        core::hint::spin_loop();
    }

    Err(UsbError::Timeout)
}

fn op_write64(regs: &XhciRegs, off: usize, v: u64) {
    regs.op_write(off, (v & 0xFFFF_FFFF) as u32);
    regs.op_write(off + 4, (v >> 32) as u32);
}

pub(super) fn rt_write64(regs: &XhciRegs, off: usize, v: u64) {
    regs.rt_write(off, (v & 0xFFFF_FFFF) as u32);
    regs.rt_write(off + 4, (v >> 32) as u32);
}

pub fn init(regs: XhciRegs) -> Result<Xhci, UsbError> {
    let cmd0 = regs.op_read(OP_USBCMD);

    if cmd0 & CMD_RS != 0 {
        regs.op_write(OP_USBCMD, cmd0 & !CMD_RS);
        spin_wait(|| regs.op_read(OP_USBSTS) & STS_HCH != 0)?;
    }

    regs.op_write(OP_USBCMD, CMD_HCRST);

    spin_wait(|| regs.op_read(OP_USBCMD) & CMD_HCRST == 0)?;
    spin_wait(|| regs.op_read(OP_USBSTS) & STS_CNR == 0)?;

    if regs.op_read(OP_PAGESIZE) & 1 == 0 {
        return Err(UsbError::Invalid);
    }

    let slots = regs.max_slots.min(64);
    let ports = regs.max_ports;

    let mut dcbaa = DmaBuf::new(64 * 8)?;
    dcbaa.zero();

    let cmd = CmdRing::new(64)?;
    let ev = EventRing::new(256)?;

    regs.op_write(OP_CONFIG, slots & 0xFF);
    op_write64(&regs, OP_DCBAAP, dcbaa.phys);
    op_write64(&regs, OP_CRCR, cmd.phys() | 1);

    regs.rt_write(RT_ERSTSZ, 1);
    regs.rt_write(RT_IMOD, 0);
    regs.rt_write(RT_IMAN, 0);
    rt_write64(&regs, RT_ERSTBA, ev.erst_phys());
    rt_write64(&regs, RT_ERDP, ev.erdp());

    regs.op_write(OP_USBCMD, CMD_RS);

    spin_wait(|| regs.op_read(OP_USBSTS) & STS_HCH == 0)?;

    let ctx_size = if regs.csz64 { 64 } else { 32 };

    Ok(Xhci {
        regs,
        cmd,
        ev,
        dcbaa,
        ctx_size,
        slots,
        ports,
    })
}

impl Xhci {
    pub fn command(&mut self, trb: Trb) -> Result<Trb, UsbError> {
        self.cmd.enqueue(trb);
        self.regs.doorbell(0, 0, 0);

        for _ in 0..2_000_000 {
            if let Some(t) = self.ev.pending() {
                let t = t;
                self.ev.pop();
                rt_write64(&self.regs, RT_ERDP, self.ev.erdp());

                if t.typ() == TRB_CMD_COMPLETION {
                    if t.completion_code() == CC_SUCCESS {
                        return Ok(t);
                    }

                    return Err(UsbError::Transfer(t.completion_code()));
                }

                continue;
            }

            core::hint::spin_loop();
        }

        Err(UsbError::Timeout)
    }

    pub fn scan_ports(&mut self) {
        for p in 1..=self.ports {
            let sc = self.regs.port_sc(p);

            if sc & PORTSC_CCS == 0 {
                continue;
            }

            let speed = (sc & PORTSC_SPEED) >> 10;

            unsafe {
                kprintf(b"usb: port %d connected, speed=%d\n\0".as_ptr(), p);
            }

            if sc & PORTSC_PED == 0 && speed <= 3 {
                self.regs.port_sc_write(p, sc | PORTSC_PR);

                let ok = spin_wait(|| {
                    self.regs.port_sc(p) & PORTSC_PRC != 0
                }).is_ok();

                if ok {
                    self.regs.port_sc_write(p, PORTSC_PRC | PORTSC_CSC);
                }
            }

            let sc2 = self.regs.port_sc(p);

            unsafe {
                kprintf(b"usb: port %d enabled=%d\n\0".as_ptr(),
                        p,
                        (sc2 & PORTSC_PED != 0) as u32);
            }
        }
    }
}
