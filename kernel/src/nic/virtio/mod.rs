pub mod descriptor;
pub mod device;
pub mod net;
pub mod pci_legacy;
pub mod queue;
pub mod transport;

pub use net::VirtioNetDevice;
pub use pci_legacy::VirtioPciLegacyNetDevice;
pub use queue::{Descriptor, DescriptorId, DescriptorPool, QueueMemory};
pub use transport::{QueueSetup, VirtioTransport};
