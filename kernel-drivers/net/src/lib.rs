//! `net` — the network driver layer for TrangorgeOS.
//!
//! This crate is the completed form of `kernel_Workspace/kernel/src/nic`. The
//! original had the right shape - a `NetworkDevice` trait with `submit_tx` and
//! a polled `poll()` - and a protocol stack above it, but the device layer
//! itself was missing the three things that decide whether a network driver
//! works under load or livelocks the machine.
//!
//! | Missing before | Where it is now |
//! |----------------|-----------------|
//! | `poll()` with no bound: a full ring loops forever inside the poll | [`napi::Napi`] — a packet budget, and a reschedule signal |
//! | no link state, so a cable pull looks like a silent device | [`device::LinkState`] |
//! | no counters, so a bug is indistinguishable from a quiet network | [`device::Stats`] |
//!
//! # Why the budget is the headline
//!
//! The original `poll()` returns only when the device says it is done. On a
//! 1 Gbit link with small buffers, the device finishes a batch faster than the
//! stack drains it, so `poll` never returns: the softirq runs forever and the
//! machine stops scheduling anything else. This is not a hypothetical, it is
//! why NAPI exists in Linux at all - the change from "an interrupt per packet"
//! to "an interrupt, then a bounded poll".
//!
//! [`napi::Napi`] is that change. A poll stops after `budget` packets and sets
//! [`PollOutcome::reschedule`], and the caller comes back. The bound is the
//! whole mechanism: without it, either the driver starves the system or it
//! drops packets, and which of those happens is a property of timing rather
//! than of the code.
//!
//! # Layers
//!
//! ```text
//!   protocol stack   (arp, ipv4, icmp — in the kernel, not here)
//!          │
//!   napi::Napi       ← budgeted polling, reschedule, and drop accounting
//!   ring::RxRing/TxRing   ← buffers, and who owns recycling one
//!   device::NetworkDevice ← the contract a card implements
//!          │
//!   virtio-net / e1000 / r8169
//! ```
//!
//! # The hardware layer
//!
//! [`virtio`] is that bottom layer, ported from Linux rather than guessed at. It
//! begins with [`virtio::spec`], the wire format: feature bit numbers, the status
//! word, and the two frame headers. It exists as a separate, pure, testable
//! module on purpose, because every one of those numbers is an ABI, and an ABI is
//! exactly the kind of thing you verify before you point DMA at it.
//!
//! The clearest example is in [`virtio::spec::MrgRxBufHeader`]. The old
//! `kernel_Workspace` driver hardcoded a ten-byte header; with
//! `VIRTIO_NET_F_MRG_RXBUF` negotiated the device writes twelve, and the two
//! extra bytes land on the first byte of the frame.
//!
//! # The reference
//!
//! `dirtycode/linux-kernel/drivers/net/` is a sparse clone of the Linux tree,
//! kept for exactly one reason: to read how upstream solved these problems
//! before reimplementing them. It is not compiled and not vendored.

#![no_std]

// `alloc` is not linked into the kernel build, but the test harness needs a heap
// to stand in for the DMA allocator: the ring's index logic is independent of
// where the memory comes from, so a `Vec` backing proves it just as well as a
// physically contiguous frame, and is the only way to exercise it off-hardware.
extern crate alloc;

pub mod checksum;
pub mod device;
pub mod error;
pub mod napi;
pub mod ring;
pub mod types;
pub mod virtio;

pub use device::{
    Duplex, Features, LinkState, NetworkDevice, PollResult, RxFrame, Speed, Stats, TxFrame,
};
pub use error::{NetworkError, PacketError};
pub use napi::{Napi, PollOutcome};
pub use ring::{BufferId, RxRing, TxRing};
pub use types::{Ethertype, Ipv4Address, MacAddress, MAX_FRAME, MIN_FRAME, MTU_DEFAULT};
