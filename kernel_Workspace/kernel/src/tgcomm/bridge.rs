//! A generic bidirectional shared-memory bridge for one layer.

use tg_comm::{authorize, AuthorizeResult, CapTable, CommMsg, Layer, MsgKind, SpscRing};

/// A shared-memory channel between the kernel and a single layer.
///
/// `tx` carries kernel -> layer messages, `rx` carries layer -> kernel
/// requests. Every inbound request is passed through the `tg_comm`
/// authorization gate before it is dispatched; denied requests get an error
/// reply and never reach the kernel handler.
pub struct LayerBridge {
    pub layer: Layer,
    pub tx: SpscRing,
    pub rx: SpscRing,
    pub table: CapTable,
}

impl LayerBridge {
    /// # Safety
    /// `tx_base`/`rx_base` must point to shared buffers of at least
    /// `RING_CONTROL_SIZE + slots * COMM_MSG_SIZE` bytes each. `tx_base`
    /// (kernel -> layer) and `rx_base` (layer -> kernel) are initialized here.
    pub unsafe fn new(
        layer: Layer,
        tx_base: *mut u8,
        tx_slots: u32,
        rx_base: *mut u8,
        rx_slots: u32,
        table: CapTable,
    ) -> Self {
        Self {
            layer,
            tx: SpscRing::from_raw(tx_base, tx_slots),
            rx: SpscRing::from_raw(rx_base, rx_slots),
            table,
        }
    }

    /// Drain inbound requests, authorize each one, and dispatch only the
    /// authorized ones through `dispatch`. The handler fills the reply; the
    /// bridge posts it back to `tx`.
    pub fn poll<F>(&self, mut dispatch: F)
    where
        F: FnMut(&CommMsg, &mut CommMsg),
    {
        while let Some(msg) = self.rx.pop() {
            // The wire stores layers as u8; recover the enum for `CommMsg::new`.
            let src = Layer::from_u8(msg.layer).unwrap_or(Layer::Kernel);
            let tgt = Layer::from_u8(msg.target).unwrap_or(Layer::Kernel);

            match authorize(&self.table, &msg) {
                AuthorizeResult::Forwarded => {
                    let mut reply = CommMsg::new(MsgKind::Reply, tgt, src, msg.opcode);
                    reply.seq = msg.seq;
                    reply.cap = msg.cap;
                    dispatch(&msg, &mut reply);
                    let _ = self.tx.push(&reply);
                }
                other => {
                    let mut reply = CommMsg::new(MsgKind::Reply, tgt, src, msg.opcode);
                    reply.seq = msg.seq;
                    reply.cap = msg.cap;
                    reply.status = -(other as i32);
                    let _ = self.tx.push(&reply);
                }
            }
        }
    }

    /// Post an asynchronous event (kernel -> layer).
    pub fn post_event(&self, opcode: u32, a0: u64, a1: u64) {
        let mut msg = CommMsg::new(MsgKind::Event, Layer::Kernel, self.layer, opcode);
        msg.a0 = a0;
        msg.a1 = a1;
        let _ = self.tx.push(&msg);
    }
}
