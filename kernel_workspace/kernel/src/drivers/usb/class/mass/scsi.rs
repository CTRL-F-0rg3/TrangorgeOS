use super::UsbMass;
use crate::drivers::usb::host::xhci::init::Xhci;
use crate::drivers::usb::UsbError;

pub fn read_capacity(x: &mut Xhci, m: &mut UsbMass) -> Result<(u32, u32), UsbError> {
    let cdb = [0x25u8, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    m.scsi_cmd(x, &cdb, m.data.phys, 8, true)?;

    unsafe {
        let p = m.data.virt;

        let last = u32::from_be_bytes([*p.add(0), *p.add(1), *p.add(2), *p.add(3)]);
        let blen = u32::from_be_bytes([*p.add(4), *p.add(5), *p.add(6), *p.add(7)]);

        Ok((last + 1, blen))
    }
}

pub fn read10(x: &mut Xhci, m: &mut UsbMass,
              lba: u32, count: u16, phys: u64) -> Result<(), UsbError> {
    let cdb = [
        0x28, 0,
        (lba >> 24) as u8, (lba >> 16) as u8, (lba >> 8) as u8, lba as u8,
        0,
        (count >> 8) as u8, count as u8,
        0,
    ];

    m.scsi_cmd(x, &cdb, phys, count as u32 * m.block_len, true)
}

pub fn write10(x: &mut Xhci, m: &mut UsbMass,
               lba: u32, count: u16, phys: u64) -> Result<(), UsbError> {
    let cdb = [
        0x2A, 0,
        (lba >> 24) as u8, (lba >> 16) as u8, (lba >> 8) as u8, lba as u8,
        0,
        (count >> 8) as u8, count as u8,
        0,
    ];

    m.scsi_cmd(x, &cdb, phys, count as u32 * m.block_len, false)
}
