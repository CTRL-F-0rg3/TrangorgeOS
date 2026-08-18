use crate::nic::error::NetworkError;

/// Adresy fizyczne trzech obszarów split virtqueue.
///
/// Adapter platformy odpowiada za to, aby pamięć była poprawnie wyrównana,
/// widoczna dla urządzenia i stabilna przez cały okres aktywności kolejki.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueSetup {
    pub size: u16,
    pub descriptor_phys: u64,
    pub driver_phys: u64,
    pub device_phys: u64,
}

/// Minimalna abstrakcja nad transportem virtio.
///
/// Implementacja dla MMIO albo PCI może korzystać z `read_volatile` /
/// `write_volatile` i barier właściwych dla architektury. Dzięki temu parsery
/// pakietów i gospodarka deskryptorami nie wymagają `unsafe`.
pub trait VirtioTransport {
    /// Przywraca stan urządzenia do zera.
    fn reset(&mut self) -> Result<(), NetworkError>;
    fn status(&self) -> u8;
    fn set_status(&mut self, status: u8);

    fn device_features(&self) -> u64;
    fn set_driver_features(&mut self, features: u64);

    fn queue_max_size(&self, queue_index: u16) -> u16;
    fn configure_queue(&mut self, queue_index: u16, setup: QueueSetup) -> Result<(), NetworkError>;

    /// Powiadamia urządzenie po opublikowaniu nowego wpisu available.
    fn notify_queue(&mut self, queue_index: u16);

    /// Odczyt konfiguracji urządzenia, np. MAC z virtio-net config space.
    fn read_config(&self, offset: u16, out: &mut [u8]) -> Result<(), NetworkError>;
}
