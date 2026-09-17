use super::abi::*;

pub fn ring_bytes(cap: u64) -> usize {
    DS_RING_HDR_SIZE + cap as usize * DS_MSG_SIZE
}

pub struct RingView {
    pub base: *mut u8,
}

impl RingView {
    pub unsafe fn new(base: *mut u8) -> Self {
        Self { base }
    }

    fn hdr(&self) -> *mut DsRing {
        self.base as *mut DsRing
    }

    pub fn init(&self, cap: u64) {
        unsafe {
            (*self.hdr()).head = 0;
            (*self.hdr()).tail = 0;
            (*self.hdr()).cap = cap;
        }
    }

    pub fn push(&self, msg: &DsMsg) -> bool {
        unsafe {
            let h = &mut *self.hdr();

            let next = (h.head + 1) % h.cap;

            if next == h.tail {
                return false;
            }

            let slot = self.base.add(DS_RING_HDR_SIZE + h.head as usize * DS_MSG_SIZE)
                as *mut DsMsg;

            slot.write_volatile(*msg);

            h.head = next;

            true
        }
    }

    pub fn pop(&self) -> Option<DsMsg> {
        unsafe {
            let h = &mut *self.hdr();

            if h.tail == h.head {
                return None;
            }

            let slot = self.base.add(DS_RING_HDR_SIZE + h.tail as usize * DS_MSG_SIZE)
                as *const DsMsg;

            let m = slot.read_volatile();

            h.tail = (h.tail + 1) % h.cap;

            Some(m)
        }
    }
}
