//! Finding AMD GPUs and working out what they are.
//!
//! Nothing here touches hardware. Probe reads PCI ids and returns a
//! classification; that separation is what lets the rules be tested against
//! every id in the table on a machine with no AMD GPU at all.

pub mod arch;
pub mod device;
pub mod families;
pub mod names;

pub use arch::Arch;
pub use device::{
    AMD_VENDOR_ID, APU_FAMILIES, AmdDevice, DEFAULT_APERTURE, is_supported, memory_model, probe,
};
pub use families::family_for;
pub use names::name_for;
