use core::sync::atomic::{AtomicU64, Ordering};
use tg_ipc_abi::wire::MsgHeader;

#[repr(C)]
pub struct RingControl {
    pub head: AtomicU64,
    pub tail: AtomicU64,
    pub capacity: u64,
    pub _pad: [u64; 5],
}

impl RingControl {
    pub const SIZE: usize = core::mem::size_of::<Self>();
}

pub struct SpscRing {
    ctrl: *mut RingControl,
    data: *mut u8,
}

unsafe impl Send for SpscRing {}
unsafe impl Sync for SpscRing {}

impl SpscRing {
    pub unsafe fn from_raw(base: *mut u8, total_size: usize) -> Self {
        let ctrl = base as *mut RingControl;
        let data = base.add(RingControl::SIZE);
        let data_size = total_size - RingControl::SIZE;
        (*ctrl).capacity = data_size as u64;
        (*ctrl).head = AtomicU64::new(0);
        (*ctrl).tail = AtomicU64::new(0);
        Self { ctrl, data }
    }

    #[inline]
    fn cap(&self) -> u64 {
        unsafe { (*self.ctrl).capacity }
    }

    #[inline]
    fn head(&self) -> u64 {
        unsafe { (*self.ctrl).head.load(Ordering::Acquire) }
    }

    #[inline]
    fn tail(&self) -> u64 {
        unsafe { (*self.ctrl).tail.load(Ordering::Acquire) }
    }

    pub fn free_space(&self) -> usize {
        let h = self.head();
        let t = self.tail();
        let c = self.cap();
        if h >= t {
            (c - h + t - 1) as usize
        } else {
            (t - h - 1) as usize
        }
    }

    pub fn available(&self) -> usize {
        let h = self.head();
        let t = self.tail();
        let c = self.cap();
        if t >= h {
            (t - h) as usize
        } else {
            (c - h + t) as usize
        }
    }

    pub fn push(&self, header: &MsgHeader, payload: &[u8]) -> bool {
        let total = header.total_len();
        if self.free_space() < total {
            return false;
        }
        let h = self.head();
        let c = self.cap();

        unsafe {
            let hdr_bytes = core::slice::from_raw_parts(
                header as *const MsgHeader as *const u8,
                MsgHeader::SIZE,
            );
            self.write_wrapping(h, hdr_bytes, c);
            self.write_wrapping(h + MsgHeader::SIZE as u64, payload, c);
            (*self.ctrl).head.store(
                (h + total as u64) % c,
                Ordering::Release,
            );
        }
        true
    }

    pub fn peek_header(&self) -> Option<MsgHeader> {
        if self.available() < MsgHeader::SIZE {
            return None;
        }
        let t = self.tail();
        let c = self.cap();
        let mut hdr = MsgHeader::new(tg_ipc_abi::wire::MsgKind::Data, 0, 0);
        unsafe {
            let hdr_bytes = core::slice::from_raw_parts_mut(
                &mut hdr as *mut MsgHeader as *mut u8,
                MsgHeader::SIZE,
            );
            self.read_wrapping(t, hdr_bytes, c);
        }
        if self.available() < hdr.total_len() {
            return None;
        }
        Some(hdr)
    }

    pub fn pop(&self, out_payload: &mut [u8]) -> Option<MsgHeader> {
        let hdr = self.peek_header()?;
        let total = hdr.total_len();
        let t = self.tail();
        let c = self.cap();

        let copy_len = hdr.payload_len as usize;
        if copy_len > out_payload.len() {
            return None;
        }

        unsafe {
            self.read_wrapping(
                t + MsgHeader::SIZE as u64,
                &mut out_payload[..copy_len],
                c,
            );
            (*self.ctrl).tail.store(
                (t + total as u64) % c,
                Ordering::Release,
            );
        }
        Some(hdr)
    }

    unsafe fn write_wrapping(&self, pos: u64, data: &[u8], cap: u64) {
        let mut p = pos % cap;
        for &b in data {
            self.data.add(p as usize).write_volatile(b);
            p = (p + 1) % cap;
        }
    }

    unsafe fn read_wrapping(&self, pos: u64, out: &mut [u8], cap: u64) {
        let mut p = pos % cap;
        for b in out.iter_mut() {
            *b = self.data.add(p as usize).read_volatile();
            p = (p + 1) % cap;
        }
    }
}