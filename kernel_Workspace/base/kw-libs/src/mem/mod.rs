// kw-libs/src/mem/mod.rs

pub mod physical;
pub mod virtual;
pub mod allocators;

pub use physical::buddy::BuddyAllocator;
pub use virtual::aspace::AddressSpace;
pub use virtual::vma::{Vma, VmaType};
pub use allocators::slab::SlabCache;