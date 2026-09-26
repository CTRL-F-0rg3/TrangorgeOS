//! `ds-fw-gpu` - the GPU device-class framework.
//!
//! The contract that `AmdGpu-TrangorgeOS` and `IntelGpu-TrangorgeOS` both
//! implement, and the server side that dispatches IPC onto it.
//!
//! ## What lives here
//!
//! * [`types`] - generational handles, so a stale handle cannot drive whatever
//!   took its place.
//! * [`traits::GpuDevice`] - the vendor-neutral *shape* a GPU driver has.
//! * [`service::GpuService`] - the dispatcher, and where capability checks
//!   live.
//! * [`codec`] - payload encoding that rejects a short or over-long message.
//!
//! ## What deliberately does not live here
//!
//! Hardware. There is no register map, no command encoder and no memory
//! allocator in this crate, because there is no such thing as one that is
//! right for both an AMD and an Intel part. That is what the two drivers are
//! for, and why this framework has no knowledge of either.
//!
//! ## The layering
//!
//! ```text
//!   ds-manager  ── CapId check, spawns one driver per matched device
//!        │
//!        ▼
//!   ds-fw-gpu   ← this crate: handles, contract, capability gate, codec
//!        │
//!        ├──────────────┬─────────────────┐
//!        ▼              ▼                 ▼
//!  AmdGpu-...    IntelGpu-...          vgpu-...
//!  (radeonsi)     (iris)             (software)
//! ```
//!
//! Each driver binary is built alone; none of them links the others.

#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod codec;
pub mod mock;
pub mod service;
pub mod traits;
pub mod types;

pub use service::{GpuService, Reply, Request};
pub use traits::{GpuDevice, Vendor};
pub use types::{
    BufferHandle, ContextHandle, FenceValue, GpuIndex, HandleTable, ShaderHandle,
};

/// How many contexts one service will track at once.
///
/// A GPU has a handful of clients at most; the rest are apps sharing one
/// context. The table is fixed so the framework needs no allocator.
pub const MAX_CONTEXTS: usize = 64;
