use tg_ipc_abi::wire::{MsgHeader, MsgKind, IpcError};
use tg_ipc_ring::SpscRing;
use tg_cap_core::ObjectId;

pub struct ChannelEnd {
    pub endpoint_id: ObjectId,
    tx: SpscRing,
    rx: SpscRing,
}

unsafe impl Send for ChannelEnd {}

impl ChannelEnd {
    pub unsafe fn from_raw(
        endpoint_id: ObjectId,
        tx_base: *mut u8,
        tx_size: usize,
        rx_base: *mut u8,
        rx_size: usize,
    ) -> Self {
        Self {
            endpoint_id,
            tx: SpscRing::from_raw(tx_base, tx_size),
            rx: SpscRing::from_raw(rx_base, rx_size),
        }
    }

    pub fn send(&self, header: &MsgHeader, payload: &[u8]) -> Result<(), IpcError> {
        if self.tx.push(header, payload) {
            Ok(())
        } else {
            Err(IpcError::NoSpace)
        }
    }

    pub fn recv(&self, out: &mut [u8]) -> Result<MsgHeader, IpcError> {
        self.rx.pop(out).ok_or(IpcError::NoData)
    }

    pub fn try_recv(&self, out: &mut [u8]) -> Option<MsgHeader> {
        self.rx.pop(out)
    }

    pub fn pending(&self) -> usize {
        self.rx.available()
    }
}

pub struct ChannelPair {
    pub a: ChannelEnd,
    pub b: ChannelEnd,
}

impl ChannelPair {
    pub unsafe fn from_raw(
        id_a: ObjectId,
        id_b: ObjectId,
        ring_a_to_b: *mut u8,
        ring_b_to_a: *mut u8,
        ring_size: usize,
    ) -> Self {
        Self {
            a: ChannelEnd::from_raw(id_a, ring_a_to_b, ring_size, ring_b_to_a, ring_size),
            b: ChannelEnd::from_raw(id_b, ring_b_to_a, ring_size, ring_a_to_b, ring_size),
        }
    }
}