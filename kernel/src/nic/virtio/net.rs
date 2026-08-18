use crate::nic::{
    error::NetworkError,
    types::MacAddress,
    virtio::{
        queue::QueueMemory,
        transport::{QueueSetup, VirtioTransport},
    },
};

pub const STATUS_ACKNOWLEDGE: u8 = 1;
pub const STATUS_DRIVER: u8 = 2;
pub const STATUS_DRIVER_OK: u8 = 4;
pub const STATUS_FEATURES_OK: u8 = 8;
pub const STATUS_FAILED: u8 = 128;

pub const VIRTIO_NET_F_MAC: u64 = 1 << 5;
pub const VIRTIO_NET_F_STATUS: u64 = 1 << 16;
pub const VIRTIO_NET_F_MRG_RXBUF: u64 = 1 << 15;

pub const RX_QUEUE: u16 = 0;
pub const TX_QUEUE: u16 = 1;
pub const DEFAULT_MTU: usize = 1500;

/// Sterownik inicjalizacji virtio-net niezależny od MMIO/PCI.
///
/// Celowo nie aktywuje checksum offload ani merged RX buffers. Taka decyzja
/// redukuje złożoność i jest bezpieczniejsza dla pierwszej implementacji.
pub struct VirtioNetDriver<T> {
    transport: T,
    mac: MacAddress,
    ready: bool,
    negotiated_features: u64,
    mtu: usize,
}

impl<T: VirtioTransport> VirtioNetDriver<T> {
    pub const fn new(transport: T) -> Self {
        Self {
            transport,
            mac: MacAddress::ZERO,
            ready: false,
            negotiated_features: 0,
            mtu: DEFAULT_MTU,
        }
    }

    #[inline]
    pub const fn is_ready(&self) -> bool {
        self.ready
    }

    #[inline]
    pub const fn mac_address(&self) -> MacAddress {
        self.mac
    }

    #[inline]
    pub const fn mtu(&self) -> usize {
        self.mtu
    }

    #[inline]
    pub const fn negotiated_features(&self) -> u64 {
        self.negotiated_features
    }

    pub fn initialize(&mut self, rx: QueueMemory, tx: QueueMemory) -> Result<(), NetworkError> {
        self.ready = false;
        self.transport.reset()?;
        if self.transport.status() != 0 {
            return Err(NetworkError::DeviceNeedsReset);
        }

        self.add_status(STATUS_ACKNOWLEDGE);
        self.add_status(STATUS_DRIVER);

        let offered = self.transport.device_features();
        // Pierwsza wersja negocjuje wyłącznie bezpieczne do zaimplementowania MAC/status.
        let requested = offered & (VIRTIO_NET_F_MAC | VIRTIO_NET_F_STATUS);
        self.transport.set_driver_features(requested);
        self.add_status(STATUS_FEATURES_OK);
        if self.transport.status() & STATUS_FEATURES_OK == 0 {
            self.fail();
            return Err(NetworkError::UnsupportedFeatures { offered, requested });
        }

        self.validate_queue(RX_QUEUE, rx)?;
        self.validate_queue(TX_QUEUE, tx)?;
        self.transport.configure_queue(RX_QUEUE, as_setup(rx))?;
        self.transport.configure_queue(TX_QUEUE, as_setup(tx))?;

        if requested & VIRTIO_NET_F_MAC != 0 {
            let mut mac = [0u8; 6];
            self.transport.read_config(0, &mut mac)?;
            self.mac = MacAddress(mac);
            if self.mac.is_zero() || self.mac.is_multicast() {
                self.fail();
                return Err(NetworkError::ReceiveFailed);
            }
        }

        self.negotiated_features = requested;
        self.add_status(STATUS_DRIVER_OK);
        self.ready = true;
        Ok(())
    }

    /// Publikuje nową pracę w virtqueue po uzupełnieniu pamięci kolejki przez adapter.
    pub fn notify_queue(&mut self, queue_index: u16) -> Result<(), NetworkError> {
        if !self.ready {
            return Err(NetworkError::DeviceNotReady);
        }
        self.transport.notify_queue(queue_index);
        Ok(())
    }

    pub fn into_inner(self) -> T {
        self.transport
    }

    fn validate_queue(&self, index: u16, memory: QueueMemory) -> Result<(), NetworkError> {
        let max = self.transport.queue_max_size(index);
        if memory.size == 0 || memory.size > max || !memory.size.is_power_of_two() {
            return Err(NetworkError::InvalidQueueSize);
        }
        if memory.descriptor_phys == 0 || memory.driver_phys == 0 || memory.device_phys == 0 {
            return Err(NetworkError::DmaAddressUnavailable);
        }
        Ok(())
    }

    fn add_status(&mut self, bit: u8) {
        self.transport.set_status(self.transport.status() | bit);
    }

    fn fail(&mut self) {
        self.transport
            .set_status(self.transport.status() | STATUS_FAILED);
        self.ready = false;
    }
}

#[inline]
const fn as_setup(memory: QueueMemory) -> QueueSetup {
    QueueSetup {
        size: memory.size,
        descriptor_phys: memory.descriptor_phys,
        driver_phys: memory.driver_phys,
        device_phys: memory.device_phys,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeTransport {
        status: u8,
        features: u64,
        configured: u8,
    }

    impl VirtioTransport for FakeTransport {
        fn reset(&mut self) -> Result<(), NetworkError> {
            self.status = 0;
            Ok(())
        }
        fn status(&self) -> u8 {
            self.status
        }
        fn set_status(&mut self, status: u8) {
            self.status = status;
        }
        fn device_features(&self) -> u64 {
            self.features
        }
        fn set_driver_features(&mut self, _features: u64) {}
        fn queue_max_size(&self, _queue_index: u16) -> u16 {
            8
        }
        fn configure_queue(
            &mut self,
            _queue_index: u16,
            _setup: QueueSetup,
        ) -> Result<(), NetworkError> {
            self.configured += 1;
            Ok(())
        }
        fn notify_queue(&mut self, _queue_index: u16) {}
        fn read_config(&self, _offset: u16, out: &mut [u8]) -> Result<(), NetworkError> {
            out.copy_from_slice(&[2, 0, 0, 0, 0, 1]);
            Ok(())
        }
    }

    fn memory() -> QueueMemory {
        QueueMemory {
            descriptor_phys: 0x1000,
            driver_phys: 0x2000,
            device_phys: 0x3000,
            size: 8,
        }
    }

    #[test]
    fn driver_follows_basic_status_sequence() {
        let mut driver = VirtioNetDriver::new(FakeTransport {
            features: VIRTIO_NET_F_MAC,
            ..FakeTransport::default()
        });
        driver.initialize(memory(), memory()).unwrap();
        assert!(driver.is_ready());
        assert_eq!(driver.mac_address(), MacAddress([2, 0, 0, 0, 0, 1]));
        assert_eq!(driver.into_inner().configured, 2);
    }
}
