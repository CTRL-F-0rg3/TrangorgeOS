//! # `tg-comm` — the strict shared-memory communication protocol of TrangorgeOS.
//!
//! This crate is the **single, independent** definition of how the layers of
//! TrangorgeOS (kernel, driverspace, userspace, manager, user driver space)
//! talk to each other through shared memory, and — critically — of *what they
//! are allowed to ask for*.
//!
//! ## Layers
//!
//! A message carries a source [`Layer`] and a target [`Layer`]. The routing
//! matrix in [`Layer::may_target`] is the first gate: an illegal route is
//! rejected before any capability is even consulted.
//!
//! ## Wire format
//!
//! Every message is a fixed 64-byte [`CommMsg`] (`#[repr(C)]`), mirrored
//! byte-for-byte by `lib-C`, `lib-odin` and `lib-ada`. Messages travel over a
//! shared-memory [`SpscRing`].
//!
//! ## Capability model & the authorization gate
//!
//! Each request names a capability handle ([`CapId`]). The kernel-side
//! [`authorize`] function checks, in order:
//!
//! 1. structural validity (magic, version, layers),
//! 2. routing policy,
//! 3. that the opcode is known and non-reserved,
//! 4. that the capability exists, carries the rights the opcode requires, and
//!    refers to an object of the required type.
//!
//! Only `Forwarded` requests reach the kernel. This is the invariant proven in
//! `lib-ada` (`Tg_Comm_Security`).
//!
//! ## Cross-language boundary
//!
//! [`ffi`] exposes the stable `extern "C"` ABI that C, Odin and Ada call. They
//! build messages and reserve storage, but the table, ring and gate logic live
//! here in Rust — consumers never re-implement the protocol.

#![cfg_attr(not(test), no_std)]

pub mod consts;
pub mod layer;
pub mod kind;
pub mod rights;
pub mod opcodes;
pub mod caps;
pub mod wire;
pub mod ring;
pub mod region;
pub mod filter;
pub mod channel;
pub mod manager;
pub mod ffi;

pub use consts::*;
pub use layer::Layer;
pub use kind::MsgKind;
pub use rights::Rights;
pub use opcodes::{opcode, OpClass, Opcode, Requirement};
pub use caps::{CapEntry, CapId, CapTable, ObjectType};
pub use wire::CommMsg;
pub use ring::{RingControl, SpscRing};
pub use region::{ShmemMapping, ShmemRegion};
pub use filter::{authorize, AuthorizeResult, Gate, KernelHandler};
pub use channel::LayerChannel;
pub use manager::Manager;

#[cfg(test)]
mod tests;

// A panic handler for the `staticlib`/`cdylib` variants. The library itself is
// panic-free, but the handler makes the archive self-contained for C/Ada/Odin
// final links. Excluded when compiling the test harness (std provides its own).
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
