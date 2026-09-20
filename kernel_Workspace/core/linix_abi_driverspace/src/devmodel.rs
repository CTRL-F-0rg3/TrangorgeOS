use kstd_base::{PhysAddr, VirtAddr, Status};

// Linux Device Model abstraction (maps to native DriverSpace registry)
pub struct LinuxDevice {
    pub name: [u8; 32],
    pub bus_id: [u8; 32],
    pub driver_data: u64,
}

pub trait LinuxDriver {
    fn probe(&mut self, dev: &LinuxDevice) -> Result<(), Status>;
    fn remove(&mut self, dev: &LinuxDevice) -> Result<(), Status>;
}

// Map Linux driver registration to native DriverSpace IPC handshake
#[inline]
pub fn register_linux_driver(_drv: &dyn LinuxDriver) -> Result<(), Status> {
    // TODO: Send DevAttachPayload via ds-ipc to ds-manager
    Err(Status::NotSupported)
}