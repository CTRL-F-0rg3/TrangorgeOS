use spin::Mutex;

use crate::nic::command::NetworkCommandRunner;
use crate::nic::device::NetworkDevice;
use crate::nic::error::NetworkError;
use crate::nic::ping::PingResult;
use crate::nic::stack::NetworkConfig;
use crate::nic::types::Ipv4Address;
use crate::nic::virtio::pci_legacy::VirtioPciLegacyNetDevice;
use crate::nic::virtio::pci_modern::VirtioModernNet;
use crate::pci::{
    self, PciDevice, VIRTIO_NET_LEGACY_DEVICE, VIRTIO_NET_MODERN_DEVICE, VIRTIO_VENDOR,
};

const ARP_ENTRIES: usize = 4;
const IDENTIFIER: u16 = 0x5452;

pub const DEFAULT_CONFIG: NetworkConfig = NetworkConfig {
    ipv4: Ipv4Address::new(10, 0, 2, 15),
    netmask: Ipv4Address::new(255, 255, 255, 0),
    gateway: Ipv4Address::new(10, 0, 2, 2),
    ttl: 64,
    arp_ttl_ms: 30_000,
};

/// Which virtio-net transport the device speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    /// virtio 0.9: registers in the I/O BAR.
    Legacy,
    /// virtio 1.0: registers in a memory BAR behind a PCI capability.
    Modern,
}

/// A NIC, whichever transport it speaks.
///
/// The two drivers share the packet layer above them but nothing below, so
/// `runtime` holds one of these rather than pushing the choice into every call.
pub enum Nic {
    Legacy(VirtioPciLegacyNetDevice),
    Modern(VirtioModernNet),
}

impl Nic {
    pub fn mac_address(&self) -> [u8; 6] {
        match self {
            Nic::Legacy(d) => d.mac_address(),
            Nic::Modern(d) => d.mac_address(),
        }
    }

    /// Whether the device reports an active link.
    pub fn link_up(&self) -> bool {
        match self {
            Nic::Legacy(_) => true,
            Nic::Modern(d) => d.link_up(),
        }
    }
}

struct NetworkRuntime {
    device: Nic,
    commands: NetworkCommandRunner<ARP_ENTRIES>,
}

static RUNTIME: Mutex<Option<NetworkRuntime>> = Mutex::new(None);

pub fn init() -> Result<(), NetworkError> {
    let mut runtime = RUNTIME.lock();
    if runtime.is_some() {
        return Ok(());
    }
    let device = open_nic()?;
    let commands = NetworkCommandRunner::new(DEFAULT_CONFIG, device.mac_address(), IDENTIFIER);
    *runtime = Some(NetworkRuntime { device, commands });
    Ok(())
}

/// Find a virtio-net NIC and bring it up, whatever transport it uses.
///
/// Legacy is tried first only because it is the older, simpler path; a machine
/// with a modern device takes the modern path, and a machine with neither gets
/// `DeviceNotReady` rather than a half-configured device.
fn open_nic() -> Result<Nic, NetworkError> {
    let (pci_device, transport) = find_nic().ok_or(NetworkError::DeviceNotReady)?;

    let nic = match transport {
        Transport::Legacy => {
            let io_base = pci::io_base_from_bar(pci::bar(pci_device.address, 0))
                .ok_or(NetworkError::DeviceNotReady)?;
            pci::enable_bus_mastering(pci_device.address);
            let mut device = VirtioPciLegacyNetDevice::new(io_base);
            device.init()?;
            Nic::Legacy(device)
        }
        Transport::Modern => {
            // The modern driver maps the BAR itself and enables bus mastering as
            // part of `init`, so neither is duplicated here.
            Nic::Modern(VirtioModernNet::init(&pci_device)?)
        }
    };

    println!(
        "[nic] virtio-net {:?} up, mac {:02x}:{:02x}:{:02x}:{:02x}:{:02x}, link {}",
        transport,
        nic.mac_address()[0],
        nic.mac_address()[1],
        nic.mac_address()[2],
        nic.mac_address()[3],
        nic.mac_address()[4],
        nic.mac_address()[5],
        if nic.link_up() { "up" } else { "down" }
    );
    Ok(nic)
}

/// The first virtio-net device on the bus, and which transport it speaks.
fn find_nic() -> Option<(PciDevice, Transport)> {
    let devices = pci::PCI_DEVICES.lock();
    devices.iter().copied().find_map(|device| {
        if device.vendor_id != VIRTIO_VENDOR {
            return None;
        }
        match device.device_id {
            VIRTIO_NET_LEGACY_DEVICE => Some((device, Transport::Legacy)),
            VIRTIO_NET_MODERN_DEVICE => Some((device, Transport::Modern)),
            _ => None,
        }
    })
}

pub fn start_ping(destination: Ipv4Address, now_ms: u64) -> Result<PingResult, NetworkError> {
    let mut runtime = RUNTIME.lock();
    let runtime = runtime.as_mut().ok_or(NetworkError::DeviceNotReady)?;
    runtime
        .commands
        .start_ping(&mut runtime.device, now_ms, destination)
}

pub fn poll(now_ms: u64) -> Result<Option<PingResult>, NetworkError> {
    let mut runtime = RUNTIME.lock();
    let runtime = match runtime.as_mut() {
        Some(value) => value,
        None => return Ok(None),
    };
    runtime.commands.poll(&mut runtime.device, now_ms)
}

pub fn is_ready() -> bool {
    RUNTIME.lock().is_some()
}

fn legacy_virtio_device() -> Option<PciDevice> {
    let devices = pci::PCI_DEVICES.lock();
    devices
        .iter()
        .copied()
        .find(|device| device.vendor_id == VIRTIO_VENDOR && device.device_id == VIRTIO_NET_LEGACY_DEVICE)
}
