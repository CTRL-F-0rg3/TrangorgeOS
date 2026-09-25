//! Fixed-slot single-producer/single-consumer ring over shared memory.

use core::sync::atomic::{AtomicU32, Ordering};

use crate::consts::{COMM_MSG_SIZE, RING_CONTROL_SIZE};
use crate::wire::CommMsg;

/// Ring control header placed at the start of the shared ring buffer.
///
/// `head` is the producer write index and `tail` the consumer read index,
/// both measured in *message slots*. `capacity` is the number of slots.
/// The layout is `#[repr(C)]` (16 bytes).
#[repr(C)]
pub struct RingControl {
    pub head: AtomicU32,
    pub tail: AtomicU32,
    pub capacity: u32,
    pub _pad: u32,
}

/// A shared-memory SPSC ring of fixed-size [`CommMsg`] slots.
pub struct SpscRing {
    base: *mut u8,
}

// SAFETY: all accesses go through `AtomicU32` for the control block and
// volatile reads/writes for slots; the ring is safe to share across threads.
unsafe impl Send for SpscRing {}
unsafe impl Sync for SpscRing {}

impl SpscRing {
    /// Attach to an already-reserved shared buffer of `slots` message slots.
    ///
    /// # Safety
    /// `base` must point to at least `RING_CONTROL_SIZE + slots * COMM_MSG_SIZE`
    /// bytes of shared, writable memory, and must not alias another live ring.
    pub unsafe fn from_raw(base: *mut u8, slots: u32) -> Self {
        let ctrl = &mut *(base as *mut RingControl);
        ctrl.head = AtomicU32::new(0);
        ctrl.tail = AtomicU32::new(0);
        ctrl.capacity = slots;
        ctrl._pad = 0;
        Self { base }
    }

    /// Wrap an already-initialized ring buffer **without** resetting its
    /// indices. Used by the C ABI to operate on a ring created by
    /// [`SpscRing::from_raw`].
    ///
    /// # Safety
    /// `base` must point to a buffer previously initialized by
    /// [`SpscRing::from_raw`] with the same capacity.
    pub unsafe fn attach(base: *mut u8) -> Self {
        Self { base }
    }

    #[inline]
    fn ctrl(&self) -> &RingControl {
        // SAFETY: `base` was validated by `from_raw` (or by the FFI wrapper).
        unsafe { &*(self.base as *const RingControl) }
    }

    #[inline]
    fn slot_ptr(&self, index: u32) -> *mut CommMsg {
        // SAFETY: `index` is kept within `[0, capacity)` by callers.
        unsafe { self.base.add(RING_CONTROL_SIZE + index as usize * COMM_MSG_SIZE) as *mut CommMsg }
    }

    /// Number of messages currently available to consume.
    pub fn available(&self) -> u32 {
        let ctrl = self.ctrl();
        let head = ctrl.head.load(Ordering::Acquire);
        let tail = ctrl.tail.load(Ordering::Acquire);
        let cap = ctrl.capacity;
        if cap == 0 {
            return 0;
        }
        if head >= tail {
            head - tail
        } else {
            cap - tail + head
        }
    }

    /// Number of free slots (one slot is always reserved to disambiguate
    /// "full" from "empty").
    pub fn free_space(&self) -> u32 {
        let ctrl = self.ctrl();
        let cap = ctrl.capacity;
        cap.saturating_sub(1).saturating_sub(self.available())
    }

    /// Push one message. Returns `false` when the ring is full.
    pub fn push(&self, msg: &CommMsg) -> bool {
        let ctrl = self.ctrl();
        let head = ctrl.head.load(Ordering::Acquire);
        let tail = ctrl.tail.load(Ordering::Acquire);
        let cap = ctrl.capacity;
        if cap == 0 {
            return false;
        }
        if (head.wrapping_add(1)) % cap == tail {
            return false;
        }
        // SAFETY: `slot_ptr(head)` is within the ring.
        unsafe {
            core::ptr::write_volatile(self.slot_ptr(head), *msg);
        }
        ctrl.head.store((head + 1) % cap, Ordering::Release);
        true
    }

    /// Pop one message, if any.
    pub fn pop(&self) -> Option<CommMsg> {
        let ctrl = self.ctrl();
        let head = ctrl.head.load(Ordering::Acquire);
        let tail = ctrl.tail.load(Ordering::Acquire);
        let cap = ctrl.capacity;
        if cap == 0 || head == tail {
            return None;
        }
        // SAFETY: `slot_ptr(tail)` is within the ring.
        let msg = unsafe { core::ptr::read_volatile(self.slot_ptr(tail)) };
        ctrl.tail.store((tail + 1) % cap, Ordering::Release);
        Some(msg)
    }

    /// Peek at the next message without consuming it.
    pub fn peek(&self) -> Option<CommMsg> {
        let ctrl = self.ctrl();
        let head = ctrl.head.load(Ordering::Acquire);
        let tail = ctrl.tail.load(Ordering::Acquire);
        if ctrl.capacity == 0 || head == tail {
            return None;
        }
        // SAFETY: `slot_ptr(tail)` is within the ring.
        Some(unsafe { core::ptr::read_volatile(self.slot_ptr(tail)) })
    }
}
