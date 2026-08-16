pub mod blockdev;
pub mod scsi;

use crate::drivers::usb::dma::DmaBuf;
use crate::drivers::usb::host::xhci::control;
use crate::drivers::usb::host::xhci::init::Xhci;
use crate::drivers::usb::host::xhci::ring::TransferRing;
use crate::drivers::usb::host::xhci::trb::*;
use crate::drivers::usb::usbcore::device::UsbDevice;
use crate::drivers::usb::UsbError;

pub const CBW_SIG: u32 = 0x4342_5355;
pub const CSW_SIG: u32 = 0x5342_5355;

static mut XHCI_PTR: *mut Xhci = core::ptr::null_mut();

pub struct UsbMass {
    pub slot: u8,
    pub out_idx: u32,
    pub in_idx: u32,
    pub out_ring: TransferRing,
    pub in_ring: TransferRing,
    pub data: DmaBuf,
    pub cmd: DmaBuf,
    pub tag: u32,
    pub sectors: u32,
    pub block_len: u32,
}

impl UsbMass {
    pub fn bulk_out(&mut self, x: &mut Xhci, phys: u64, len: u32) -> Result<(), UsbError> {
        self.out_ring.enqueue(Trb::normal(phys, len));
        x.regs.doorbell(self.slot as u32, self.out_idx, 0);

        let cc = control::wait_transfer_ep(x, self.slot, self.out_idx as u8)?;

        if cc != CC_SUCCESS {
            return Err(UsbError::Transfer(cc));
        }

        Ok(())
    }

    pub fn bulk_in(&mut self, x: &mut Xhci, phys: u64, len: u32) -> Result<(), UsbError> {
        self.in_ring.enqueue(Trb::normal(phys, len));
        x.regs.doorbell(self.slot as u32, self.in_idx, 0);

        let cc = control::wait_transfer_ep(x, self.slot, self.in_idx as u8)?;

        if cc != CC_SUCCESS {
            return Err(UsbError::Transfer(cc));
        }

        Ok(())
    }

    pub fn scsi_cmd(&mut self, x: &mut Xhci, cdb: &[u8],
                    data_phys: u64, data_len: u32, dir_in: bool) -> Result<(), UsbError> {
        self.tag += 1;

        let mut cbw = [0u8; 31];

        cbw[0..4].copy_from_slice(&CBW_SIG.to_le_bytes());
        cbw[4..8].copy_from_slice(&self.tag.to_le_bytes());
        cbw[8..12].copy_from_slice(&data_len.to_le_bytes());
        cbw[12] = if dir_in { 0x80 } else { 0x00 };
        cbw[13] = 0;
        cbw[14] = cdb.len() as u8;
        cbw[15..15 + cdb.len()].copy_from_slice(cdb);

        unsafe {
            core::ptr::copy_nonoverlapping(cbw.as_ptr(), self.cmd.virt, 31);
        }

        self.bulk_out(x, self.cmd.phys, 31)?;

        if data_len > 0 {
            if dir_in {
                self.bulk_in(x, data_phys, data_len)?;
            } else {
                self.bulk_out(x, data_phys, data_len)?;
            }
        }

        self.bulk_in(x, self.cmd.phys, 13)?;

        unsafe {
            let csw = self.cmd.virt;
            let sig = u32::from_le_bytes([*csw.add(0), *csw.add(1), *csw.add(2), *csw.add(3)]);
            let status = *csw.add(12);

            if sig != CSW_SIG || status != 0 {
                return Err(UsbError::Io2);
            }
        }

        Ok(())
    }
}

pub fn attach(x: &mut Xhci, dev: &mut UsbDevice) -> Result<bool, UsbError> {
    let mut found = false;

    for i in 0..dev.iface_count {
        let f = &dev.ifaces[i];

        if f.class == 8 && f.subclass == 6 && f.protocol == 0x50 {
            found = true;
            break;
        }
    }

    if !found {
        return Ok(false);
    }

    let mut out_ep = 0u8;
    let mut in_ep = 0u8;
    let mut mps_out = 512u16;
    let mut mps_in = 512u16;

    for i in 0..dev.ep_count {
        let e = &dev.eps[i];

        if e.attributes & 0x03 == 2 {
            if e.address & 0x80 != 0 {
                in_ep = e.address & 0x0F;
                mps_in = e.max_packet;
            } else {
                out_ep = e.address & 0x0F;
                mps_out = e.max_packet;
            }
        }
    }

    let out_idx = (2 * out_ep) as u32;
    let in_idx = (2 * in_ep + 1) as u32;

    let out_ring = TransferRing::new(16)?;
    let in_ring = TransferRing::new(16)?;
    let data = DmaBuf::new(0x10000)?;
    let cmd = DmaBuf::new(64)?;

    dev.ctx.setup_configure_bulk_pair(dev.speed, dev.port,
                                      out_idx, in_idx,
                                      mps_out, mps_in,
                                      out_ring.phys(), in_ring.phys());

    x.command(Trb::configure_ep(dev.slot, dev.ctx.input.phys))?;

    unsafe {
        XHCI_PTR = x as *mut Xhci;
    }

    let mut m = UsbMass {
        slot: dev.slot,
        out_idx,
        in_idx,
        out_ring,
        in_ring,
        data,
        cmd,
        tag: 0,
        sectors: 0,
        block_len: 512,
    };

    let (sectors, blen) = scsi::read_capacity(x, &mut m)?;

    m.sectors = sectors;
    m.block_len = blen;

    unsafe {
        MASS0 = Some(m);

        if let Some(d) = MASS0.as_ref() {
            crate::fs::driver::registry::register(d);
        }
    }

    Ok(true)
}

static mut MASS0: Option<UsbMass> = None;

pub fn with_controller<F: FnOnce(&mut Xhci) -> R, R>(f: F) -> R {
    let x = unsafe { &mut *XHCI_PTR };
    f(x)
}