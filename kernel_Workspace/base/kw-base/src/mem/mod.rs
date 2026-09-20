pub mod frame;
pub mod paging;
pub mod heap;
pub mod slab;
pub mod buddy;

pub use frame::Frame;
pub use buddy::ContiguousFrames;
pub use slab::SlabCache;