use alloc::vec;
use alloc::vec::Vec;

use spin::Mutex;

use crate::nic::command::NetworkCommandRunner;
use crate::nic::device::{NetworkDevice, PollResult, RxFrame, TxFrame};
use crate::nic::error::NetworkError;
use crate::nic::ping::PingResult;
use crate::nic::stack::NetworkConfig;
use crate::nic::types::{Ipv4Address, MacAddress};
use crate::nic::virtio::pci_legacy::VirtioPciLegacyNetDevice;
use crate::nic::virtio::pci_modern::VirtioModernNet;
use crate::pci::{
    self, PciDevice, VIRTIO_NET_LEGACY_DEVICE, VIRTIO_NET_MODERN_DEVICE, VIRTIO_VENDOR,
};
use crate::println;

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

/// A received frame held for the caller to read once.
///
/// The modern driver hands its DMA buffer back to the device the instant the
/// frame is taken, so a borrow into that buffer would be reading memory the
/// device may already be writing. `NetworkDevice::take_rx` has to return a
/// borrow, so the copy lives here and the caller is expected to call
/// `take_pending` rather than `take_rx` on a modern device.
pub struct PendingFrame {
    pub bytes: Vec<u8>,
}

impl Nic {
    /// Which transport this is, for the boot log.
    pub fn transport(&self) -> Transport {
        match self {
            Nic::Legacy(_) => Transport::Legacy,
            Nic::Modern(_) => Transport::Modern,
        }
    }

    /// Take a received frame from a modern device, copied out of its buffer.
    ///
    /// This is the modern counterpart to `NetworkDevice::take_rx`, and the one a
    /// caller should use: it hands back an owned `Vec` rather than a borrow, so
    /// nothing keeps pointing into memory the device has been given back.
    pub fn take_pending(&mut self) -> Option<PendingFrame> {
        match self {
            Nic::Legacy(d) => d.take_rx().map(|f| PendingFrame {
                bytes: f.bytes.to_vec(),
            }),
            Nic::Modern(d) => d.take_frame().map(|bytes| PendingFrame { bytes }),
        }
    }
}

impl NetworkDevice for Nic {
    fn init(&mut self) -> Result<(), NetworkError> {
        match self {
            Nic::Legacy(d) => d.init(),
            // Brought up by `open_nic`, which holds the PCI address the
            // constructor needs. Re-initialising here would re-map the BAR.
            Nic::Modern(_) => Ok(()),
        }
    }

    fn mac_address(&self) -> MacAddress {
        match self {
            Nic::Legacy(d) => d.mac_address(),
            // `MacAddress` is those same six bytes.
            Nic::Modern(d) => MacAddress(d.mac_address()),
        }
    }

    fn mtu(&self) -> usize {
        // 1500 bytes of payload, which is what both drivers' buffers hold.
        1500
    }

    fn submit_tx(&mut self, frame: TxFrame<'_>) -> Result<(), NetworkError> {
        match self {
            Nic::Legacy(d) => d.submit_tx(frame),
            Nic::Modern(d) => d.send(frame.bytes),
        }
    }

    fn poll(&mut self) -> Result<PollResult, NetworkError> {
        match self {
            Nic::Legacy(d) => d.poll(),
            // The modern driver has no interrupt status to report; `take_pending`
            // is how a frame is collected, not how one is waited for. Reporting a
            // reset only when the device says so keeps `poll` free of the
            // side effect of draining the receive ring.
            Nic::Modern(d) => Ok(PollResult {
                tx_completed: 0,
                rx_available: 0,
                device_needs_reset: d.in_failed_state(),
            }),
        }
    }

    /// Only meaningful for a legacy device.
    ///
    /// A modern device cannot return a borrow into its DMA buffer, because the
    /// buffer is already back with the device by the time this is called. Use
    /// [`Nic::take_pending`] there; this returns `None` rather than a dangling
    /// borrow so the mistake is a missing frame and not undefined behaviour.
    fn take_rx(&mut self) -> Option<RxFrame<'_>> {
        match self {
            Nic::Legacy(d) => d.take_rx(),
            Nic::Modern(_) => None,
        }
    }

    fn recycle_rx(&mut self, buffer_id: u16) -> Result<(), NetworkError> {
        match self {
            Nic::Legacy(d) => d.recycle_rx(buffer_id),
            // The modern driver refills its receive ring inside `take_frame`.
            Nic::Modern(_) => Ok(()),
        }
    }
}

/// The device owns a single mapped BAR and its own buffers; it is only ever
/// reachable through the `RUNTIME` mutex below.
///
/// Raw device pointers are not `Send`, and that is correct — moving one across
/// threads is meaningless without the mapping it points at. Here the mapping is
/// process-wide and created before the value is ever shared, so the assertion
/// holds; it is asserted rather than assumed because that is the whole claim.
unsafe impl Send for Nic {}

impl Nic {
    pub fn mac_address(&self) -> MacAddress {
        match self {
            Nic::Legacy(d) => d.mac_address(),
            // The modern driver reads the address out of the device's config
            // blob as raw bytes; `MacAddress` is that same six bytes.
            Nic::Modern(d) => MacAddress(d.mac_address()),
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
        "[nic] virtio-net {:?} up, mac {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}, link {}",
        transport,
        nic.mac_address().0[0],
        nic.mac_address().0[1],
        nic.mac_address().0[2],
        nic.mac_address().0[3],
        nic.mac_address().0[4],
        nic.mac_address().0[5],
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
