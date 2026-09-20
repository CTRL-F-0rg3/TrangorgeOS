#![no_std]

pub mod encode;
pub mod decode;
pub mod endian;

pub const DS_MAGIC: u32 = 0x4453_4D53;
pub const DS_VERSION: u32 = 1;
pub const MSG_SIZE: usize = 64;
pub const RING_SLOTS: usize = 256;
pub const RING_SIZE: usize = RING_SLOTS * MSG_SIZE;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DsMsg {
    pub magic: u32,
    pub version: u16,
    pub cmd: u16,
    pub id: u64,
    pub flags: u32,
    pub status: i32,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
}

impl DsMsg {
    pub const fn new(cmd: u16) -> Self {
        Self {
            magic: DS_MAGIC,
            version: DS_VERSION as u16,
            cmd,
            id: 0,
            flags: 0,
            status: 0,
            arg0: 0,
            arg1: 0,
            arg2: 0,
        }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.magic == DS_MAGIC && self.version == DS_VERSION as u16
    }

    #[inline]
    pub fn is_ok(&self) -> bool {
        self.status == 0
    }

    #[inline]
    pub fn error(&self) -> super::Status {
        super::Status::from_i32(self.status)
    }
}

#[repr(C)]
pub struct DsRing {
    pub head: u64,
    pub tail: u64,
    pub cap: u64,
    pub _pad: u64,
}

impl DsRing {
    pub const fn new(cap: u64) -> Self {
        Self { head: 0, tail: 0, cap, _pad: 0 }
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.head.wrapping_sub(self.tail) >= self.cap
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    #[inline]
    pub fn available(&self) -> u64 {
        self.head.wrapping_sub(self.tail)
    }
}