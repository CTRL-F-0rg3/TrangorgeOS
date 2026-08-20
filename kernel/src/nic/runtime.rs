use spin::Mutex;

use crate::nic::command::NetworkCommandRunner;
use crate::nic::device::NetworkDevice;
use crate::nic::error::NetworkError;
use crate::nic::ping::PingResult;
use crate::nic::stack::NetworkConfig;
use crate::nic::types::Ipv4Address;
use crate::nic::virtio::pci_legacy::VirtioPciLegacyNetDevice;
use crate::pci::{self, PciDevice, VIRTIO_NET_LEGACY_DEVICE, VIRTIO_VENDOR};

const ARP_ENTRIES: usize = 4;
const IDENTIFIER: u16 = 0x5452;

pub const DEFAULT_CONFIG: NetworkConfig = NetworkConfig {
    ipv4: Ipv4Address::new(10, 0, 2, 15),
    netmask: Ipv4Address::new(255, 255, 255, 0),
    gateway: Ipv4Address::new(10, 0, 2, 2),
    ttl: 64,
    arp_ttl_ms: 30_000,
};

struct NetworkRuntime {
    device: VirtioPciLegacyNetDevice,
    commands: NetworkCommandRunner<ARP_ENTRIES>,
}

static RUNTIME: Mutex<Option<NetworkRuntime>> = Mutex::new(None);

pub fn init() -> Result<(), NetworkError> {
    let mut runtime = RUNTIME.lock();
    if runtime.is_some() {
        return Ok(());
    }
    let pci_device = legacy_virtio_device().ok_or(NetworkError::DeviceNotReady)?;
    let io_base = pci::io_base_from_bar(pci::bar(pci_device.address, 0))
        .ok_or(NetworkError::DeviceNotReady)?;
    pci::enable_bus_mastering(pci_device.address);
    let mut device = VirtioPciLegacyNetDevice::new(io_base);
    device.init()?;
    let commands = NetworkCommandRunner::new(DEFAULT_CONFIG, device.mac_address(), IDENTIFIER);
    *runtime = Some(NetworkRuntime { device, commands });
    Ok(())
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
