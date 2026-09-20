#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};
use kapi_abi::wire::{DsMsg, DsRing, RING_SLOTS, MSG_SIZE};

pub struct Channel {
    k2d_ring: *mut DsRing,
    d2k_ring: *mut DsRing,
    k2d_msgs: *mut DsMsg,
    d2k_msgs: *mut DsMsg,
    next_id: AtomicU64,
}

unsafe impl Send for Channel {}
unsafe impl Sync for Channel {}

impl Channel {
    pub unsafe fn new(
        k2d_ring: *mut DsRing,
        d2k_ring: *mut DsRing,
        k2d_msgs: *mut DsMsg,
        d2k_msgs: *mut DsMsg,
    ) -> Self {
        Self {
            k2d_ring,
            d2k_ring,
            k2d_msgs,
            d2k_msgs,
            next_id: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    pub unsafe fn send(&self, cmd: u16, arg0: u64, arg1: u64, arg2: u64) -> u64 {
        let ring = &*self.d2k_ring;
        if ring.is_full() {
            return 0;
        }

        let id = self.next_id();
        let slot = (ring.head % ring.cap) as usize;
        let msg_ptr = self.d2k_msgs.add(slot);

        let msg = DsMsg {
            magic: kapi_abi::wire::DS_MAGIC,
            version: kapi_abi::wire::DS_VERSION as u16,
            cmd,
            id,
            flags: 0,
            status: 0,
            arg0,
            arg1,
            arg2,
        };

        core::ptr::write_volatile(msg_ptr, msg);
        core::sync::atomic::fence(Ordering::Release);
        (*self.d2k_ring).head = ring.head.wrapping_add(1);

        id
    }

    pub unsafe fn recv(&self) -> Option<DsMsg> {
        let ring = &*self.k2d_ring;
        if ring.is_empty() {
            return None;
        }

        let slot = (ring.tail % ring.cap) as usize;
        let msg_ptr = self.k2k_msgs.add(slot);
        let msg = core::ptr::read_volatile(msg_ptr);

        if !msg.is_valid() {
            return None;
        }

        core::sync::atomic::fence(Ordering::Acquire);
        (*self.k2d_ring).tail = ring.tail.wrapping_add(1);

        Some(msg)
    }

    pub unsafe fn wait_for_reply(&self, id: u64, max_spins: u32) -> Option<DsMsg> {
        let mut spins = 0;
        while spins < max_spins {
            if let Some(msg) = self.recv() {
                if msg.id == id {
                    return Some(msg);
                }
            }
            core::hint::spin_loop();
            spins += 1;
        }
        None
    }
}