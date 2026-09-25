//! Convenience channel for a layer: a pair of rings plus routing metadata.

use crate::layer::Layer;
use crate::ring::SpscRing;
use crate::wire::CommMsg;

/// A bidirectional channel owned by one layer.
///
/// `tx` carries messages *from* this layer, `rx` carries messages *to* it.
pub struct LayerChannel {
    pub layer: Layer,
    pub tx: SpscRing,
    pub rx: SpscRing,
}

unsafe impl Send for LayerChannel {}
unsafe impl Sync for LayerChannel {}

impl LayerChannel {
    /// # Safety
    /// `tx_base`/`rx_base` must point to shared buffers of at least
    /// `RING_CONTROL_SIZE + slots * COMM_MSG_SIZE` bytes each.
    pub unsafe fn from_raw(
        layer: Layer,
        tx_base: *mut u8,
        tx_slots: u32,
        rx_base: *mut u8,
        rx_slots: u32,
    ) -> Self {
        Self {
            layer,
            tx: SpscRing::from_raw(tx_base, tx_slots),
            rx: SpscRing::from_raw(rx_base, rx_slots),
        }
    }

    /// Enqueue a message toward the peer. `false` when the ring is full.
    pub fn send(&self, msg: &CommMsg) -> bool {
        self.tx.push(msg)
    }

    /// Dequeue the next message for this layer, if any.
    pub fn recv(&self) -> Option<CommMsg> {
        self.rx.pop()
    }

    /// Peek at the next incoming message.
    pub fn peek(&self) -> Option<CommMsg> {
        self.rx.peek()
    }

    /// Number of pending incoming messages.
    pub fn pending(&self) -> u32 {
        self.rx.available()
    }
}
