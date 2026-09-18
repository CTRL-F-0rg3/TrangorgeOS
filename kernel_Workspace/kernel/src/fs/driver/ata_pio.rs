use super::block::{BlockDevice, DriverError};
use super::lock::IrqGuard;
use super::port;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

const ATA_TIMEOUT: u32 = 1_000_000;

const ATA_REG_DATA: u16 = 0;
const ATA_REG_ERROR: u16 = 1;
const ATA_REG_SECT: u16 = 2;
const ATA_REG_LBA_LO: u16 = 3;
const ATA_REG_LBA_MID: u16 = 4;
const ATA_REG_LBA_HI: u16 = 5;
const ATA_REG_DRIVE: u16 = 6;
const ATA_REG_STATUS: u16 = 7;

const ATA_STATUS_ERR: u8 = 1 << 0;
const ATA_STATUS_DRQ: u8 = 1 << 3;
const ATA_STATUS_BSY: u8 = 1 << 7;

const ATA_CMD_IDENTIFY: u8 = 0xEC;
const ATA_CMD_READ28: u8 = 0x20;
const ATA_CMD_WRITE28: u8 = 0x30;
const ATA_CMD_FLUSH: u8 = 0xE7;

pub struct AtaPio {
    base: u16,
    ctrl: u16,
    slave: bool,
    present: AtomicBool,
    sectors: AtomicU64,
}

pub static ATA0: AtaPio = AtaPio::master(0x1F0, 0x3F6);
pub static ATA1: AtaPio = AtaPio::slave(0x1F0, 0x3F6);

impl AtaPio {
    pub const fn master(base: u16, ctrl: u16) -> Self {
        Self {
            base,
            ctrl,
            slave: false,
            present: AtomicBool::new(false),
            sectors: AtomicU64::new(0),
        }
    }

    pub const fn slave(base: u16, ctrl: u16) -> Self {
        Self {
            base,
            ctrl,
            slave: true,
            present: AtomicBool::new(false),
            sectors: AtomicU64::new(0),
        }
    }

    fn drive_lba(&self) -> u8 {
        if self.slave { 0xF0 } else { 0xE0 }
    }

    fn drive_identify(&self) -> u8 {
        if self.slave { 0xB0 } else { 0xA0 }
    }

    pub fn is_present(&self) -> bool {
        self.present.load(Ordering::Relaxed)
    }

    fn disable_irq(&self) {
        unsafe { port::outb(self.ctrl, 0x02) };
    }

    fn status(&self) -> u8 {
        unsafe { port::inb(self.base + ATA_REG_STATUS) }
    }

    fn wait_not_bsy(&self) -> Result<(), DriverError> {
        for _ in 0..ATA_TIMEOUT {
            if self.status() & ATA_STATUS_BSY == 0 {
                return Ok(());
            }
        }

        Err(DriverError::Timeout)
    }

    fn wait_drq(&self) -> Result<(), DriverError> {
        for _ in 0..ATA_TIMEOUT {
            let st = self.status();

            if st & ATA_STATUS_ERR != 0 {
                return Err(DriverError::Io);
            }

            if st & ATA_STATUS_DRQ != 0 {
                return Ok(());
            }
        }

        Err(DriverError::Timeout)
    }

    fn wait_ready(&self) -> Result<(), DriverError> {
        for _ in 0..ATA_TIMEOUT {
            let st = self.status();

            if st & ATA_STATUS_BSY == 0 && st & ATA_STATUS_DRQ == 0 {
                return Ok(());
            }
        }

        Err(DriverError::Timeout)
    }

    fn setup_lba28(&self, lba: u32, count: u8, cmd: u8) -> Result<(), DriverError> {
        unsafe {
            port::outb(self.base + ATA_REG_DRIVE, self.drive_lba() | ((lba >> 24) & 0x0F) as u8);
            port::outb(self.base + ATA_REG_SECT, count);
            port::outb(self.base + ATA_REG_LBA_LO, (lba & 0xFF) as u8);
            port::outb(self.base + ATA_REG_LBA_MID, ((lba >> 8) & 0xFF) as u8);
            port::outb(self.base + ATA_REG_LBA_HI, ((lba >> 16) & 0xFF) as u8);
            port::outb(self.base + ATA_REG_STATUS, cmd);
        }

        Ok(())
    }

    pub fn identify(&self) -> Result<[u16; 256], DriverError> {
        let _g = IrqGuard::lock();

        unsafe {
            port::outb(self.base + ATA_REG_DRIVE, self.drive_identify());
            port::outb(self.base + ATA_REG_SECT, 0);
            port::outb(self.base + ATA_REG_LBA_LO, 0);
            port::outb(self.base + ATA_REG_LBA_MID, 0);
            port::outb(self.base + ATA_REG_LBA_HI, 0);
            port::outb(self.base + ATA_REG_STATUS, ATA_CMD_IDENTIFY);

            if self.status() == 0 {
                return Err(DriverError::NoDevice);
            }

            self.wait_not_bsy()?;

            if self.status() & ATA_STATUS_ERR != 0 {
                return Err(DriverError::NoDevice);
            }

            self.wait_drq()?;

            let mut id = [0u16; 256];

            for i in 0..256 {
                id[i] = port::inw(self.base + ATA_REG_DATA);
            }

            Ok(id)
        }
    }

    fn read_sector(&self, lba: u32, buf: &mut [u8]) -> Result<(), DriverError> {
        let _g = IrqGuard::lock();

        self.setup_lba28(lba, 1, ATA_CMD_READ28)?;
        self.wait_drq()?;

        for i in 0..256 {
            let w = unsafe { port::inw(self.base + ATA_REG_DATA) };
            buf[i * 2] = (w & 0xFF) as u8;
            buf[i * 2 + 1] = (w >> 8) as u8;
        }

        Ok(())
    }

    fn write_sector(&self, lba: u32, buf: &[u8]) -> Result<(), DriverError> {
        let _g = IrqGuard::lock();

        self.setup_lba28(lba, 1, ATA_CMD_WRITE28)?;
        self.wait_drq()?;

        for i in 0..256 {
            let w = ((buf[i * 2 + 1] as u16) << 8) | buf[i * 2] as u16;
            unsafe { port::outw(self.base + ATA_REG_DATA, w) };
        }

        unsafe { port::outb(self.base + ATA_REG_STATUS, ATA_CMD_FLUSH) };
        self.wait_ready()?;

        Ok(())
    }
}

impl BlockDevice for AtaPio {
    fn name(&self) -> &'static str {
        if self.slave { "ata1" } else { "ata0" }
    }

    fn block_size(&self) -> usize {
        512
    }

    fn block_count(&self) -> u64 {
        self.sectors.load(Ordering::Relaxed)
    }

    fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError> {
        if !self.present.load(Ordering::Relaxed) {
            return Err(DriverError::NoDevice);
        }

        if buf.len() < 512 {
            return Err(DriverError::InvalidLength);
        }

        if block >= self.block_count() || block > 0x0FFF_FFFF {
            return Err(DriverError::InvalidBlock);
        }

        self.read_sector(block as u32, buf)
    }

    fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError> {
        if !self.present.load(Ordering::Relaxed) {
            return Err(DriverError::NoDevice);
        }

        if buf.len() < 512 {
            return Err(DriverError::InvalidLength);
        }

        if block >= self.block_count() || block > 0x0FFF_FFFF {
            return Err(DriverError::InvalidBlock);
        }

        self.write_sector(block as u32, buf)
    }
}

fn probe_dev(dev: &AtaPio) -> bool {
    match dev.identify() {
        Ok(id) => {
            let sectors = (id[60] as u64) | ((id[61] as u64) << 16);
            dev.sectors.store(sectors, Ordering::Relaxed);
            dev.present.store(true, Ordering::Relaxed);
            true
        }
        Err(_) => false,
    }
}

pub fn probe() -> usize {
    ATA0.disable_irq();
    ATA1.disable_irq();

    let mut found = 0;
    if probe_dev(&ATA0) {
        found += 1;
    }
    if probe_dev(&ATA1) {
        found += 1;
    }
    found
}
