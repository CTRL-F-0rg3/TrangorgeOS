//! A generic shared-memory client for any layer (driverspace, userspace, ...).
//!
//! This is the "wide API" for consumers: attach to two shared rings and talk
//! to the kernel without knowing the ring/capability internals.

use crate::wire::CommMsg;
use crate::kind::MsgKind;
use crate::layer::Layer;
use crate::ring::SpscRing;

/// A bidirectional shared-memory client endpoint.
///
/// `tx` carries layer -> kernel requests, `rx` carries kernel -> layer replies.
pub struct Client {
    layer: Layer,
    tx: SpscRing,
    rx: SpscRing,
    next_id: u32,
    default_cap: u32,
}

impl Client {
    /// # Safety
    /// `tx_base`/`rx_base` must point to already-initialized shared rings.
    pub unsafe fn attach(
        layer: Layer,
        tx_base: *mut u8,
        rx_base: *mut u8,
        default_cap: u32,
    ) -> Self {
        Self {
            layer,
            tx: SpscRing::attach(tx_base),
            rx: SpscRing::attach(rx_base),
            next_id: 1,
            default_cap,
        }
    }

    /// Post a request to the kernel; returns the sequence id (0 on failure).
    pub fn request(&mut self, opcode: u32, a0: u64, a1: u64, a2: u64) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);

        let mut msg = CommMsg::new(MsgKind::Request, self.layer, Layer::Kernel, opcode);
        msg.seq = id;
        msg.cap = self.default_cap;
        msg.a0 = a0;
        msg.a1 = a1;
        msg.a2 = a2;

        if self.tx.push(&msg) {
            id
        } else {
            0
        }
    }

    /// Receive the next reply, if any.
    pub fn recv(&self) -> Option<CommMsg> {
        self.rx.pop()
    }

    /// Number of pending replies.
    pub fn pending(&self) -> u32 {
        self.rx.available()
    }
}
