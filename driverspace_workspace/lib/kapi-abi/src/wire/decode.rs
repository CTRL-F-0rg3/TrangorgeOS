#![no_std]

use super::{DsMsg, DS_MAGIC, DS_VERSION};

pub struct Decoder<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    #[inline]
    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pos >= self.buf.len() { return None; }
        let v = self.buf[self.pos];
        self.pos += 1;
        Some(v)
    }

    #[inline]
    pub fn read_u16(&mut self) -> Option<u16> {
        if self.pos + 2 > self.buf.len() { return None; }
        let v = u16::from_le_bytes([self.buf[self.pos], self.buf[self.pos + 1]]);
        self.pos += 2;
        Some(v)
    }

    #[inline]
    pub fn read_u32(&mut self) -> Option<u32> {
        if self.pos + 4 > self.buf.len() { return None; }
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.buf[self.pos..self.pos + 4]);
        self.pos += 4;
        Some(u32::from_le_bytes(bytes))
    }

    #[inline]
    pub fn read_u64(&mut self) -> Option<u64> {
        if self.pos + 8 > self.buf.len() { return None; }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.buf[self.pos..self.pos + 8]);
        self.pos += 8;
        Some(u64::from_le_bytes(bytes))
    }

    pub fn read_msg(&mut self) -> Option<DsMsg> {
        if self.remaining() < super::MSG_SIZE { return None; }
        let magic = self.read_u32()?;
        if magic != DS_MAGIC { return None; }
        let version = self.read_u16()?;
        if version != DS_VERSION as u16 { return None; }
        let cmd = self.read_u16()?;
        let id = self.read_u64()?;
        let flags = self.read_u32()?;
        let status = self.read_u32()? as i32;
        let arg0 = self.read_u64()?;
        let arg1 = self.read_u64()?;
        let arg2 = self.read_u64()?;
        Some(DsMsg {
            magic,
            version,
            cmd,
            id,
            flags,
            status,
            arg0,
            arg1,
            arg2,
        })
    }

    pub fn read_bytes(&mut self, len: usize) -> Option<&'a [u8]> {
        if self.pos + len > self.buf.len() { return None; }
        let slice = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        Some(slice)
    }
}