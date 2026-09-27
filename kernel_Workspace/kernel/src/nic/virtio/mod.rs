//! The virtio transports.
//!
//! Two drivers, because there are two incompatible devices:
//!
//! * [`pci_legacy`] — virtio 0.9 (`0x1AF4:0x1000`), registers in an I/O BAR.
//! * [`pci_modern`] — virtio 1.0 (`0x1AF4:0x1041`), registers in a memory BAR
//!   reached through a PCI capability. This is what a current hypervisor
//!   hands out by default, and it has no I/O port at all.
//!
//! They share [`queue`] and nothing else: different register layout, different
//! feature negotiation, different queue setup. `runtime` picks between them.
//!
//! The packet layer above — `crate::nic::stack`, `ethernet`, `ipv4`, `icmp` —
//! is common to both and lives one level up.

pub mod pci_legacy;
pub mod pci_modern;
pub mod queue;

pub use pci_legacy::VirtioPciLegacyNetDevice;
pub use pci_modern::VirtioModernNet;
pub use queue::Descriptor;
