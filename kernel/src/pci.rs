use x86_64::instructions::port::Port;

const CONFIG_ADDRESS: u16 = 0xCF8;
const CONFIG_DATA: u16 = 0xCFC;

pub const VIRTIO_VENDOR_ID: u16 = 0x1AF4;
pub const VIRTIO_NET_LEGACY_DEVICE_ID: u16 = 0x1000;
pub const VIRTIO_NET_MODERN_DEVICE_ID: u16 = 0x1041;

pub const MAX_FOUND_DEVICES: usize = 32;

crate::test_module!({
    let mut found: [Option<PciDevice>; MAX_FOUND_DEVICES] = [None; MAX_FOUND_DEVICES];
    let count = scan_bus0(&mut found);
    if count == 0 {
        return Err("PCI bus 0 scan found no devices at all");
    }
    let host_bridge_present = found[..count]
        .iter()
        .flatten()
        .any(|dev| dev.bus == 0 && dev.device == 0 && dev.function == 0);
    if !host_bridge_present {
        return Err("PCI bus 0 scan did not find the expected host bridge at 0:0:0");
    }
    Ok("PCI config space access verified via host bridge at 0:0:0")
});

#[derive(Debug, Clone, Copy)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub header_type: u8,
}

impl PciDevice {
    pub fn bar(&self, index: u8) -> u32 {
        config_read_u32(self.bus, self.device, self.function, 0x10 + index * 4)
    }

    pub fn is_virtio_net(&self) -> bool {
        self.vendor_id == VIRTIO_VENDOR_ID
            && (self.device_id == VIRTIO_NET_LEGACY_DEVICE_ID
                || self.device_id == VIRTIO_NET_MODERN_DEVICE_ID)
    }
}

pub fn config_read_u32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address: u32 = (1 << 31)
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);

    let mut addr_port: Port<u32> = Port::new(CONFIG_ADDRESS);
    let mut data_port: Port<u32> = Port::new(CONFIG_DATA);

    unsafe {
        addr_port.write(address);
        data_port.read()
    }
}

pub fn config_read_u16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let value = config_read_u32(bus, device, function, offset & 0xFC);
    let shift = (offset as u32 & 2) * 8;
    ((value >> shift) & 0xFFFF) as u16
}

pub fn config_read_u8(bus: u8, device: u8, function: u8, offset: u8) -> u8 {
    let value = config_read_u32(bus, device, function, offset & 0xFC);
    let shift = (offset as u32 & 3) * 8;
    ((value >> shift) & 0xFF) as u8
}

fn probe_function(bus: u8, device: u8, function: u8) -> Option<PciDevice> {
    let vendor_id = config_read_u16(bus, device, function, 0x00);
    if vendor_id == 0xFFFF {
        return None;
    }
    let device_id = config_read_u16(bus, device, function, 0x02);
    let class = config_read_u8(bus, device, function, 0x0B);
    let subclass = config_read_u8(bus, device, function, 0x0A);
    let prog_if = config_read_u8(bus, device, function, 0x09);
    let header_type = config_read_u8(bus, device, function, 0x0E);

    Some(PciDevice {
        bus,
        device,
        function,
        vendor_id,
        device_id,
        class,
        subclass,
        prog_if,
        header_type,
    })
}

pub fn scan_bus0(found: &mut [Option<PciDevice>; MAX_FOUND_DEVICES]) -> usize {
    let mut count = 0;
    for device in 0..32u8 {
        let Some(dev) = probe_function(0, device, 0) else {
            continue;
        };
        let multi_function = dev.header_type & 0x80 != 0;
        if count < MAX_FOUND_DEVICES {
            found[count] = Some(dev);
            count += 1;
        }
        if multi_function {
            for function in 1..8u8 {
                if let Some(dev) = probe_function(0, device, function) {
                    if count < MAX_FOUND_DEVICES {
                        found[count] = Some(dev);
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

pub fn find_virtio_net(
    found: &[Option<PciDevice>; MAX_FOUND_DEVICES],
    count: usize,
) -> Option<PciDevice> {
    found[..count].iter().flatten().find(|dev| dev.is_virtio_net()).copied()
}
