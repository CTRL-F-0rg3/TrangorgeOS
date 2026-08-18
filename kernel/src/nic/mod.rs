//! Lekki stos sieciowy zintegrowany z crate'em kernel.
//!
//! Zasada: `virtio` obsługuje urządzenie i kolejki, natomiast pozostałe
//! moduły parsują oraz budują pakiety bez zależności od MMIO/DMA.

pub mod arp;
pub mod checksum;
pub mod device;
pub mod driver;
pub mod error;
pub mod ethernet;
pub mod icmp;
pub mod ipv4;
pub mod packet;
pub mod protocols;
pub mod stack;
pub mod types;
pub mod virtio;

pub use device::{NetworkDevice, PollResult, RxFrame, TxFrame};
pub use error::{NetworkError, PacketError};
pub use stack::{NetworkConfig, NetworkStack, PingRequest, StackEvent};
pub use types::{Ipv4Address, MacAddress};
