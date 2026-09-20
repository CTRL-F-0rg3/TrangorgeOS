#![no_std]

use super::{DsMsg, DS_MAGIC, DS_VERSION};

pub struct Encoder<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> Encoder<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    #[inline]
    pub fn write_u8(&mut self, v: u8) -> bool {
        if self.pos >= self.buf.len() { return false; }
        self.buf[self.pos] = v;
        self.pos += 1;
        true
    }

    #[inline]
    pub fn write_u16(&mut self, v: u16) -> bool {
        if self.pos + 2 > self.buf.len() { return false; }
        let bytes = v.to_le_bytes();
        self.buf[self.pos] = bytes[0];
        self.buf[self.pos + 1] = bytes[1];
        self.pos += 2;
        true
    }

    #[inline]
    pub fn write_u32(&mut self, v: u32) -> bool {
        if self.pos + 4 > self.buf.len() { return false; }
        let bytes = v.to_le_bytes();
        self.buf[self.pos..self.pos + 4].copy_from_slice(&bytes);
        self.pos += 4;
        true
    }

    #[inline]
    pub fn write_u64(&mut self, v: u64) -> bool {
        if self.pos + 8 > self.buf.len() { return false; }
        let bytes = v.to_le_bytes();
        self.buf[self.pos..self.pos + 8].copy_from_slice(&bytes);
        self.pos += 8;
        true
    }

    pub fn write_msg(&mut self, msg: &DsMsg) -> bool {
        if self.remaining() < super::MSG_SIZE { return false; }
        self.write_u32(msg.magic);
        self.write_u16(msg.version);
        self.write_u16(msg.cmd);
        self.write_u64(msg.id);
        self.write_u32(msg.flags);
        self.write_u32(msg.status as u32);
        self.write_u64(msg.arg0);
        self.write_u64(msg.arg1);
        self.write_u64(msg.arg2);
        true
    }

    pub fn write_bytes(&mut self, data: &[u8]) -> bool {
        if self.pos + data.len() > self.buf.len() { return false; }
        self.buf[self.pos..self.pos + data.len()].copy_from_slice(data);
        self.pos += data.len();
        true
    }

    pub fn pos(&self) -> usize {
        self.pos
    }
}