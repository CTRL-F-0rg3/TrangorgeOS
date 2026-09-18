use crate::nic::error::NetworkError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueSetup {
    pub size: u16,
    pub descriptor_phys: u64,
    pub driver_phys: u64,
    pub device_phys: u64,
}

pub trait VirtioTransport {
    fn reset(&mut self) -> Result<(), NetworkError>;
    fn status(&self) -> u8;
    fn set_status(&mut self, status: u8);

    fn device_features(&self) -> u64;
    fn set_driver_features(&mut self, features: u64);

    fn queue_max_size(&self, queue_index: u16) -> u16;
    fn configure_queue(&mut self, queue_index: u16, setup: QueueSetup) -> Result<(), NetworkError>;

    fn notify_queue(&mut self, queue_index: u16);

    fn read_config(&self, offset: u16, out: &mut [u8]) -> Result<(), NetworkError>;
}
