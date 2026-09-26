use core::sync::atomic::{AtomicU64, Ordering};
use kapi_abi::DsMsg;

#[repr(C)]
pub struct RingBuffer {
    head: AtomicU64,
    tail: AtomicU64,
    capacity: u32,
    _pad: u32,
}

impl RingBuffer {
    pub unsafe fn from_ptr(ptr: *mut u8) -> &'static mut Self {
        unsafe { &mut *(ptr as *mut Self) }
    }

    fn slot_ptr(&self, index: u64) -> *mut DsMsg {
        let base = self as *const _ as *const u8;
        let offset = core::mem::size_of::<Self>() 
            + (index as usize % self.capacity as usize) * core::mem::size_of::<DsMsg>();
        unsafe { base.add(offset) as *mut DsMsg }
    }

    pub fn push(&self, msg: &DsMsg) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        
        if head - tail >= self.capacity as u64 {
            return false;
        }

        unsafe {
            self.slot_ptr(head).write(*msg);
        }
        
        self.head.store(head + 1, Ordering::Release);
        true
    }

    pub fn pop(&self) -> Option<DsMsg> {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        
        if tail >= head {
            return None;
        }

        let msg = unsafe { self.slot_ptr(tail).read() };
        self.tail.store(tail + 1, Ordering::Release);
        Some(msg)
    }

    pub fn is_empty(&self) -> bool {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        tail >= head
    }

    /// Number of messages still waiting to be read from the queue.
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        head.saturating_sub(tail) as usize
    }
}