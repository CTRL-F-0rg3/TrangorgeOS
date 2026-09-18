use crate::primitives::PAGE_SIZE;

pub trait PageAlign {
    fn align_up(self) -> Self;
    fn align_down(self) -> Self;
    fn is_aligned(self) -> bool;
}

impl PageAlign for u64 {
    #[inline]
    fn align_up(self) -> Self {
        (self + PAGE_SIZE as u64 - 1) & !(PAGE_SIZE as u64 - 1)
    }

    #[inline]
    fn align_down(self) -> Self {
        self & !(PAGE_SIZE as u64 - 1)
    }

    #[inline]
    fn is_aligned(self) -> bool {
        self & (PAGE_SIZE as u64 - 1) == 0
    }
}

impl PageAlign for usize {
    #[inline]
    fn align_up(self) -> Self {
        (self + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
    }

    #[inline]
    fn align_down(self) -> Self {
        self & !(PAGE_SIZE - 1)
    }

    #[inline]
    fn is_aligned(self) -> bool {
        self & (PAGE_SIZE - 1) == 0
    }
}

pub trait IoPort {
    fn read_u8(&self, port: u16) -> u8;
    fn write_u8(&self, port: u16, val: u8);
    fn read_u16(&self, port: u16) -> u16;
    fn write_u16(&self, port: u16, val: u16);
    fn read_u32(&self, port: u16) -> u32;
    fn write_u32(&self, port: u16, val: u32);
}

pub trait MmioRegion {
    fn read_volatile_u32(&self, offset: usize) -> u32;
    fn write_volatile_u32(&self, offset: usize, val: u32);
    fn read_volatile_u64(&self, offset: usize) -> u64;
    fn write_volatile_u64(&self, offset: usize, val: u64);
}