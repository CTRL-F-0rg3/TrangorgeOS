//! The virtio-net hardware layer, ported from Linux's `drivers/net/virtio_net.c`.
//!
//! # Where this sits
//!
//! Linux's `virtio_net.c` is a stack of layers. Two of them are here so far:
//!
//! ```text
//!   spec.rs   ← the wire format: feature bits, status, frame headers
//!   ring.rs   ← the split virtqueue, in shared memory
//! ```
//!
//! Both are deliberately free of MMIO, DMA allocation and interrupts, so they
//! can be tested exhaustively on the host. Everything that needs real hardware
//! — [`pci_legacy`]'s port I/O and MMIO transports, the buffer allocator, the
//! `NetworkDevice` implementation — is layered on top of these two and is the
//! part that has to be exercised in the emulator.
//!
//! # What Linux got right that the old code did not
//!
//! Each of these is a bug the port fixes, not a style preference:
//!
//! - **A conditional frame header.** Ten bytes without `MRG_RXBUF`, twelve with
//!   it. The old code hardcoded ten. See [`spec::MrgRxBufHeader`].
//! - **`FEATURES_OK`.** The old status sequence is `ACKNOWLEDGE → DRIVER →
//!   DRIVER_OK`, and modern devices expect the fourth step.
//! - **A real queue depth.** The old driver used four descriptors per virtqueue.
//!   Four is not a buffer pool; it is one packet and three retries.
//! - **No hardcoded 1536.** Frame size follows the negotiated MTU.
//! - **Bounds-checked config.** `virtio_net_config` is variable-length, and the
//!   optional fields are individually gated on their feature bits.
//!
//! # The ring, specifically
//!
//! The split virtqueue is a shared-memory protocol, and the only subtle part is
//! that the driver and the device update *different* indices in *shared* words.
//! The memory-ordering rules around that are not obvious and are not optional, so
//! [`ring`] states them at each store rather than leaving them to be inferred —
//! see `Driver::publish` and `Device::publish`.

pub mod ring;
pub mod spec;

pub use ring::{
    index_delta, layout, layout_size, need_event, validate_size, Descriptor, RingError, RingLayout,
    SplitQueue, UsedElem,
};
pub use spec::{
    has, link_is_up, link_tx_blocked, net_header_len, parse_config, u16_le, MrgRxBufHeader,
    NetConfig, NetHeader,
};
