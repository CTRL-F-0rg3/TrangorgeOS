//! Finding Intel GPUs and working out what they are.
//!
//! Nothing here touches hardware. Probe reads PCI ids and returns a
//! classification; that separation is what lets the rules be tested against
//! every id in the tree on a machine with no Intel GPU at all.

pub mod device;
pub mod genkind;
pub mod names;

pub use device::{
    DEFAULT_GTT_SIZE, INTEL_VENDOR_ID, IntelDevice, is_supported, memory_model, probe,
};
pub use genkind::GenKind;
pub use names::name_for;
