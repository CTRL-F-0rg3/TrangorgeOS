//! No-std string formatting into a fixed-size buffer.

use core::fmt::{self, Write};

pub const MAX_LOG_LINE_LEN: usize = 256;

pub struct LogBuffer {
    buf: [u8; MAX_LOG_LINE_LEN],
    pos: usize,
}

impl LogBuffer {
    pub const fn new() -> Self {
        Self {
            buf: [0; MAX_LOG_LINE_LEN],
            pos: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.pos]
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }
}

impl Write for LogBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - self.pos;
        let copy_len = if bytes.len() > remaining { remaining } else { bytes.len() };
        
        self.buf[self.pos..self.pos + copy_len].copy_from_slice(&bytes[..copy_len]);
        self.pos += copy_len;
        Ok(())
    }
}