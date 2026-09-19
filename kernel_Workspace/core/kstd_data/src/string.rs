use crate::vec::Vec;
use core::fmt;

pub struct String { vec: Vec<u8> }

impl String {
    pub const fn new() -> Self { Self { vec: Vec::new() } }
    
    pub fn from_str(s: &str) -> Self {
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s.as_bytes());
        Self { vec: v }
    }

    pub fn push_str(&mut self, s: &str) {
        self.vec.extend_from_slice(s.as_bytes());
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.vec.as_slice()) }
    }
    
    pub fn len(&self) -> usize { self.vec.len() }
    pub fn is_empty(&self) -> bool { self.vec.is_empty() }
}

impl core::ops::Deref for String {
    type Target = str;
    fn deref(&self) -> &str { self.as_str() }
}

impl fmt::Write for String {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}