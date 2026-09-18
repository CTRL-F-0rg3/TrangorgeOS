use crate::println;
use crate::testing::TestResult;
use alloc::vec::Vec;
use spin::Mutex;
use x86_64::instructions::port::Port;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciAddress {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub address: PciAddress,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub header_type: u8,
}

fn config_address(addr: PciAddress, offset: u8) -> u32 {
    (1u32 << 31)
        | ((addr.bus as u32) << 16)
        | ((addr.device as u32) << 11)
        | ((addr.function as u32) << 8)
        | ((offset as u32) & 0xFC)
}

pub fn config_read_u32(addr: PciAddress, offset: u8) -> u32 {
    let mut addr_port: Port<u32> = Port::new(CONFIG_ADDRESS);
    let mut data_port: Port<u32> = Port::new(CONFIG_DATA);
    unsafe {
        addr_port.write(config_address(addr, offset));
        data_port.read()
    }
}

pub fn config_write_u32(addr: PciAddress, offset: u8, value: u32) {
    let mut addr_port: Port<u32> = Port::new(CONFIG_ADDRESS);
    let mut data_port: Port<u32> = Port::new(CONFIG_DATA);
    unsafe {
        addr_port.write(config_address(addr, offset));
        data_port.write(value);
    }
}

pub fn config_read_u16(addr: PciAddress, offset: u8) -> u16 {
    let value = config_read_u32(addr, offset & 0xFC);
    let shift = ((offset & 2) as u32) * 8;
    ((value >> shift) & 0xFFFF) as u16
}

pub fn config_read_u8(addr: PciAddress, offset: u8) -> u8 {
    let value = config_read_u32(addr, offset & 0xFC);
    let shift = ((offset & 3) as u32) * 8;
    ((value >> shift) & 0xFF) as u8
}

fn probe_function(bus: u8, device: u8, function: u8) -> Option<PciDevice> {
    let addr = PciAddress { bus, device, function };
    let vendor_id = config_read_u16(addr, 0x00);
    if vendor_id == 0xFFFF {
        return None;
    }
    let device_id = config_read_u16(addr, 0x02);
    let class_code = config_read_u8(addr, 0x0B);
    let subclass = config_read_u8(addr, 0x0A);
    let prog_if = config_read_u8(addr, 0x09);
    let header_type = config_read_u8(addr, 0x0E);
    Some(PciDevice {
        address: addr,
        vendor_id,
        device_id,
        class_code,
        subclass,
        prog_if,
        header_type,
    })
}

pub fn enumerate() -> Vec<PciDevice> {
    let mut devices = Vec::new();
    for bus in 0u8..=255 {
        for device in 0u8..32 {
            if let Some(dev0) = probe_function(bus, device, 0) {
                let multifunction = dev0.header_type & 0x80 != 0;
                devices.push(dev0);
                if multifunction {
                    for function in 1u8..8 {
                        if let Some(devn) = probe_function(bus, device, function) {
                            devices.push(devn);
                        }
                    }
                }
            }
            if bus == 255 && device == 31 {
                break;
            }
        }
        if bus == 255 {
            break;
        }
    }
    devices
}

pub const RTL8139_VENDOR: u16 = 0x10EC;
pub const RTL8139_DEVICE: u16 = 0x8139;
pub const VIRTIO_VENDOR: u16 = 0x1AF4;
pub const VIRTIO_NET_LEGACY_DEVICE: u16 = 0x1000;
pub const VIRTIO_NET_MODERN_DEVICE: u16 = 0x1041;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NicKind {
    Rtl8139,
    VirtioNet,
}

pub fn find_nic(devices: &[PciDevice]) -> Option<(PciDevice, NicKind)> {
    if let Some(dev) = devices
        .iter()
        .find(|d| d.vendor_id == RTL8139_VENDOR && d.device_id == RTL8139_DEVICE)
    {
        return Some((*dev, NicKind::Rtl8139));
    }
    devices
        .iter()
        .find(|d| {
            d.vendor_id == VIRTIO_VENDOR
                && (d.device_id == VIRTIO_NET_LEGACY_DEVICE || d.device_id == VIRTIO_NET_MODERN_DEVICE)
        })
        .map(|d| (*d, NicKind::VirtioNet))
}

pub fn bar(addr: PciAddress, bar_index: u8) -> u32 {
    let offset = 0x10 + bar_index * 4;
    config_read_u32(addr, offset)
}

pub fn io_base_from_bar(bar_value: u32) -> Option<u16> {
    if bar_value & 0x1 == 1 {
        Some((bar_value & 0xFFFC) as u16)
    } else {
        None
    }
}

pub fn mem_base_from_bar(bar_value: u32) -> Option<u32> {
    if bar_value & 0x1 == 0 {
        Some(bar_value & 0xFFFFFFF0)
    } else {
        None
    }
}

pub fn enable_bus_mastering(addr: PciAddress) {
    let full = config_read_u32(addr, 0x04);
    let command = (full & 0xFFFF) | 0x0004 | 0x0001;
    let new_full = (full & 0xFFFF0000) | command;
    config_write_u32(addr, 0x04, new_full);
}

pub static PCI_DEVICES: Mutex<Vec<PciDevice>> = Mutex::new(Vec::new());

pub fn init() {
    let devices = enumerate();
    println!("PCI: found {} device(s)", devices.len());
    if let Some((dev, kind)) = find_nic(&devices) {
        println!(
            "PCI: NIC found -> {:?} at bus {} device {} function {}",
            kind, dev.address.bus, dev.address.device, dev.address.function
        );
    } else {
        println!("PCI: no supported NIC found");
    }
    *PCI_DEVICES.lock() = devices;
}

pub fn self_test() -> TestResult {
    let devices = enumerate();
    if devices.is_empty() {
        return Err("no PCI devices found");
    }
    if devices.iter().any(|d| d.vendor_id == 0xFFFF) {
        return Err("invalid vendor id present in scan results");
    }
    Ok("PCI bus scan completed")
}
