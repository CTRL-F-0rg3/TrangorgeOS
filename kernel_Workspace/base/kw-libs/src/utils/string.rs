// Safe wrappers for C-string operations, heavily used in FS paths and device names.

pub unsafe fn cstr_len(ptr: *const u8) -> usize {
    let mut len = 0;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    len
}

pub unsafe fn cstr_to_slice<'a>(ptr: *const u8) -> &'a [u8] {
    core::slice::from_raw_parts(ptr, cstr_len(ptr))
}

pub unsafe fn cstr_to_str<'a>(ptr: *const u8) -> Option<&'a str> {
    core::str::from_utf8(cstr_to_slice(ptr)).ok()
}

// Simple fixed-capacity string buffer for formatting paths/logs without heap allocation.
pub struct StrBuffer<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> StrBuffer<N> {
    pub const fn new() -> Self {
        Self { buf: [0; N], len: 0 }
    }

    pub fn push_str(&mut self, s: &str) -> bool {
        let bytes = s.as_bytes();
        if self.len + bytes.len() > N { return false; }
        self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        true
    }

    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }
    
    pub fn as_cstr_ptr(&mut self) -> *const u8 {
        if self.len < N {
            self.buf[self.len] = 0;
        }
        self.buf.as_ptr()
    }
}