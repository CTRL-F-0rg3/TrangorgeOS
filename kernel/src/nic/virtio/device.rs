use crate::nic::device::{Capabilities, NetworkDevice};
use crate::nic::error::NetworkError;
use crate::nic::protocols::MacAddress;
use crate::nic::virtio::net::VirtioNetDevice;

impl NetworkDevice for VirtioNetDevice {
    fn mac_address(&self) -> MacAddress {
        self.mac
    }

    fn mtu(&self) -> usize {
        1500
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            checksum_offload: true,
            multi_queue: false,
            jumbo_frames: false,
        }
    }

    fn transmit(&mut self, _packet: &[u8]) -> Result<(), NetworkError> {
        if !self.ready {
            return Err(NetworkError::DeviceNotReady);
        }
        Err(NetworkError::Unsupported)
    }

    fn receive(&mut self) -> Option<&[u8]> {
        None
    }
}
