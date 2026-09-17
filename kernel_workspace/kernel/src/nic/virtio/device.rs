use crate::nic::error::NetworkError;
use crate::nic::protocols::MacAddress;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub checksum_offload: bool,
    pub multi_queue: bool,
    pub jumbo_frames: bool,
}

pub trait NetworkDevice {
    fn mac_address(&self) -> MacAddress;
    fn mtu(&self) -> usize;
    fn capabilities(&self) -> Capabilities;
    fn transmit(&mut self, packet: &[u8]) -> Result<(), NetworkError>;
    fn receive(&mut self) -> Option<&[u8]>;
}
