use crate::arch::x86_64::{inl, outl};

const PCI_CONFIG_ADDR: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

#[derive(Debug, Clone, Copy)]
pub struct PciAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl PciAddress {
    pub const fn new(bus: u8, device: u8, function: u8) -> Self {
        Self { bus, device, function }
    }

    fn config_addr(self, offset: u8) -> u32 {
        0x8000_0000 
            | ((self.bus as u32) << 16) 
            | ((self.device as u32 & 0x1F) << 11) 
            | ((self.function as u32 & 0x07) << 8) 
            | (offset as u32 & 0xFC)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub addr: PciAddress,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
}

pub fn read_config32(addr: PciAddress, offset: u8) -> u32 {
    unsafe {
        outl(PCI_CONFIG_ADDR, addr.config_addr(offset));
        inl(PCI_CONFIG_DATA)
    }
}

pub fn write_config32(addr: PciAddress, offset: u8, value: u32) {
    unsafe {
        outl(PCI_CONFIG_ADDR, addr.config_addr(offset));
        outl(PCI_CONFIG_DATA, value);
    }
}

pub fn scan_bus() -> kstd_data::Vec<PciDevice> {
    let mut devices = kstd_data::Vec::new();
    for bus in 0..=255 {
        for dev in 0..32 {
            let addr = PciAddress::new(bus, dev, 0);
            let id = read_config32(addr, 0);
            let vendor = id as u16;
            if vendor == 0xFFFF { continue; }
            
            devices.push(PciDevice {
                addr,
                vendor_id: vendor,
                device_id: (id >> 16) as u16,
                class_code: ((id >> 24) & 0xFF) as u8, // Simplified, usually at offset 0x08
                subclass: 0,
            });
        }
    }
    devices
}