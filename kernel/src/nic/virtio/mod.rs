//! Virtio-net zależne od platformy.
//!
//! `transport` jest jedyną granicą dla MMIO/DMA. `queue` zarządza pulą
//! deskryptorów bez alokacji, a `net` wykonuje sekwencję inicjalizacji.

pub mod descriptor;
pub mod device;
pub mod net;
pub mod queue;
pub mod transport;

pub use net::VirtioNetDevice;
pub use queue::{Descriptor, DescriptorId, DescriptorPool, QueueMemory};
pub use transport::{QueueSetup, VirtioTransport};
