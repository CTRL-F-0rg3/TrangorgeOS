use crate::nic::{error::NetworkError, types::MacAddress};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PollResult {
    pub tx_completed: u16,
    pub rx_available: u16,
    pub device_needs_reset: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TxFrame<'a> {
    pub bytes: &'a [u8],
}

impl<'a> TxFrame<'a> {
    #[inline]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

#[derive(Debug)]
pub struct RxFrame<'a> {
    pub buffer_id: u16,
    pub bytes: &'a [u8],
}

pub trait NetworkDevice {
    fn init(&mut self) -> Result<(), NetworkError>;
    fn mac_address(&self) -> MacAddress;
    fn mtu(&self) -> usize;
    fn submit_tx(&mut self, frame: TxFrame<'_>) -> Result<(), NetworkError>;
    fn poll(&mut self) -> Result<PollResult, NetworkError>;
    fn take_rx(&mut self) -> Option<RxFrame<'_>>;
    fn recycle_rx(&mut self, buffer_id: u16) -> Result<(), NetworkError>;
}
