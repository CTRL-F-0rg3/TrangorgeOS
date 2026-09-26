//! The `linux_dirent64` record, which is what `getdents64` produces and what
//! `std::fs::read_dir` parses.
//!
//! `std` does not call `readdir`/`readdir64`: `library/std/src/sys/fs/unix.rs`
//! opens the directory and reads `struct linux_dirent64` records straight out of
//! the `getdents64` buffer itself, walking the `d_reclen` fields. So the exact
//! byte layout below *is* the contract — a one-byte mistake in `d_reclen` makes
//! `read_dir` yield garbage names rather than fail loudly.

#[cfg(not(test))]
use alloc::vec::Vec;
#[cfg(test)]
use std::vec::Vec;

/// The fixed part of a record: everything before the name.
///
/// `packed` is not optional. `struct linux_dirent64` really is 19 bytes — the
/// `d_name` array starts at offset 19, not at the next 8-byte boundary — while
/// the *records* are 8-byte aligned relative to the start of the buffer. A plain
/// `#[repr(C)]` would round the struct up to 24 and every name would come back
/// with four bytes of padding in front of it.
#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// Inode number, or 0 when the filesystem does not have one.
    pub d_ino: u64,
    /// Offset of the *next* record, for resuming a `read_dir` iteration.
    pub d_off: i64,
    /// The length of this whole record, name and its NUL included.
    pub d_reclen: u16,
    /// The entry type, one of the [`kind`] values.
    pub d_type: u8,
}

impl Header {
    /// A zeroed header, for writing a record from scratch.
    pub const fn zeroed() -> Self {
        Self {
            d_ino: 0,
            d_off: 0,
            d_reclen: 0,
            d_type: 0,
        }
    }

    /// Inode number, read out of a possibly unaligned header.
    pub fn ino(&self) -> u64 {
        self.d_ino
    }

    /// Next-record offset, read out of a possibly unaligned header.
    pub fn off(&self) -> i64 {
        self.d_off
    }

    /// Record length, read out of a possibly unaligned header.
    pub fn reclen(&self) -> u16 {
        self.d_reclen
    }

    /// Entry type, read out of a possibly unaligned header.
    pub fn type_(&self) -> u8 {
        self.d_type
    }
}

// Hand-written because the derived one would take a reference to a packed field.
impl core::fmt::Debug for Header {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Header")
            .field("d_ino", &self.ino())
            .field("d_off", &self.off())
            .field("d_reclen", &self.reclen())
            .field("d_type", &self.type_())
            .finish()
    }
}

/// The `d_type` values of a `linux_dirent64`.
pub mod kind {
    /// Unknown; the reader must `lstat` to find out.
    pub const UNKNOWN: u8 = 0;
    /// A named pipe.
    pub const FIFO: u8 = 1;
    /// A character device.
    pub const CHR: u8 = 2;
    /// A directory.
    pub const DIR: u8 = 4;
    /// A block device.
    pub const BLK: u8 = 6;
    /// A regular file.
    pub const REG: u8 = 8;
    /// A symbolic link.
    pub const LNK: u8 = 10;
    /// A socket.
    pub const SOCK: u8 = 12;
}

/// The longest name a record can carry, which is the filesystem `NAME_MAX`.
pub const MAX_NAME: usize = 255;

/// The largest record this crate will write: the 19-byte header plus a
/// 255-byte name and its NUL, rounded up to the kernel's 8-byte record
/// alignment. The kernel is allowed to use a larger value, and `decode` accepts
/// anything up to the buffer length, so this is a cap on writing, not reading.
pub const MAX_RECLEN: usize = 280;

/// The size of the fixed part of a record.
pub const HEADER_LEN: usize = core::mem::size_of::<Header>();

/// One directory entry, decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Inode number, or 0.
    pub ino: u64,
    /// Offset to resume from.
    pub off: i64,
    /// One of the [`kind`] values.
    pub type_: u8,
    /// The entry name, without the terminating NUL.
    pub name: Vec<u8>,
}

impl Entry {
    /// A regular-file entry.
    pub fn file(name: &str) -> Self {
        Self::new(name, kind::REG, 0, 0)
    }

    /// A directory entry.
    pub fn dir(name: &str) -> Self {
        Self::new(name, kind::DIR, 0, 0)
    }

    /// An entry with every field spelled out.
    pub fn new(name: &str, type_: u8, ino: u64, off: i64) -> Self {
        Self {
            ino,
            off,
            type_,
            name: name.as_bytes().to_vec(),
        }
    }

    /// Whether the entry is a directory.
    pub fn is_dir(&self) -> bool {
        self.type_ == kind::DIR
    }

    /// Whether the entry is a regular file.
    pub fn is_file(&self) -> bool {
        self.type_ == kind::REG
    }

    /// The name, if it is valid UTF-8.
    pub fn name_str(&self) -> Option<&str> {
        core::str::from_utf8(&self.name).ok()
    }
}

/// How many bytes a record for a name of `name_len` bytes occupies.
///
/// The name is NUL-terminated and the whole record is rounded up to 8 bytes,
/// because the kernel places the next record at an 8-byte boundary and `std`
/// relies on that when it walks the buffer.
pub const fn reclen_for(name_len: usize) -> usize {
    let raw = HEADER_LEN + name_len + 1;
    (raw + 7) & !7
}

/// The longest name a record can carry.
pub const fn max_name_len() -> usize {
    MAX_NAME
}

/// Write one record into the front of `buf`, returning the bytes written.
///
/// Returns `None` when the record does not fit, or when the name is longer than
/// [`MAX_NAME`]. The first means "retry with a bigger buffer"; the second means
/// the kernel could not have produced this record either.
///
/// # Safety
///
/// `buf` must be writable for `buf.len()` bytes and must not be shared with
/// anything else, since the whole record is overwritten.
pub unsafe fn encode(buf: &mut [u8], entry: &Entry) -> Option<usize> {
    if entry.name.len() > MAX_NAME {
        return None;
    }
    let len = reclen_for(entry.name.len());
    if len > buf.len() {
        return None;
    }

    let h = Header {
        d_ino: entry.ino,
        d_off: entry.off,
        d_reclen: len as u16,
        d_type: entry.type_,
    };

    // SAFETY: `reclen_for` guarantees `len >= HEADER_LEN` and
    // `HEADER_LEN + name.len() + 1 <= len`, and the caller promised `buf` is
    // writable for at least `len` bytes. `Header` is `packed`, so writing it
    // through a possibly-unaligned pointer is well defined. The name copy and
    // the NUL stay inside `buf` by the same inequality; the padding after the
    // NUL is left alone, which is what the kernel's own reader expects.
    unsafe {
        core::ptr::write_unaligned(buf.as_mut_ptr().cast::<Header>(), h);
        core::ptr::copy_nonoverlapping(
            entry.name.as_ptr(),
            buf.as_mut_ptr().add(HEADER_LEN),
            entry.name.len(),
        );
        *buf.get_unchecked_mut(HEADER_LEN + entry.name.len()) = 0;
    }
    Some(len)
}

/// Decode the record at the front of `buf`.
///
/// Returns `None` when the buffer holds no whole record, which is how a
/// `read_dir` loop knows it has reached the end.
///
/// # Safety
///
/// `buf` must be a buffer `getdents64` actually filled, containing whole
/// records written by the kernel.
pub unsafe fn decode(buf: &[u8]) -> Option<Entry> {
    if buf.len() < HEADER_LEN {
        return None;
    }

    // SAFETY: `buf.len() >= HEADER_LEN`, so a `Header` can be read from the
    // start; `read_unaligned` is required and sufficient because the buffer
    // comes from the kernel and is only 8-byte aligned, not 19. `d_reclen` is
    // validated against the buffer length before any name byte is touched.
    let h = unsafe { core::ptr::read_unaligned(buf.as_ptr().cast::<Header>()) };
    let len = h.reclen() as usize;
    if len < HEADER_LEN || len > buf.len() {
        return None;
    }

    // The name runs to the first NUL. The kernel guarantees one inside the
    // record, so this cannot read past `len`; the `unwrap_or` only covers a
    // buffer that was not produced by a kernel.
    let name_buf = &buf[HEADER_LEN..len];
    let name_len = name_buf
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(name_buf.len());

    Some(Entry {
        ino: h.ino(),
        off: h.off(),
        type_: h.type_(),
        name: name_buf[..name_len].to_vec(),
    })
}

/// Every entry in a `getdents64` buffer, in order.
///
/// A truncated or self-inconsistent record ends the walk instead of running off
/// the end of the buffer, so a buggy kernel yields a short directory listing
/// rather than a crash.
pub fn decode_all(buf: &[u8]) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut at = 0usize;

    while let Some(entry) = read_at(buf, at) {
        let (entry, len) = entry;
        at += len;
        out.push(entry);
    }

    out
}

/// The entry at byte offset `at`, with the record length it declared.
fn read_at(buf: &[u8], at: usize) -> Option<(Entry, usize)> {
    if at.checked_add(HEADER_LEN)? > buf.len() {
        return None;
    }
    // SAFETY: the check above put the whole header inside `buf`, and `decode`
    // re-validates `d_reclen` against `buf.len() - at` before reading a name.
    let entry = unsafe { decode(&buf[at..]) }?;
    let len = reclen_of(&buf, at);
    if len == 0 {
        return None;
    }
    Some((entry, len))
}

/// The record length stored at byte offset `at`, or 0 if the header is not
/// there at all.
fn reclen_of(buf: &[u8], at: usize) -> usize {
    if at.checked_add(HEADER_LEN).is_none_or(|end| end > buf.len()) {
        return 0;
    }
    // SAFETY: the check above puts the header inside `buf`. Reading it as a
    // packed, unaligned value is well defined.
    unsafe { core::ptr::read_unaligned(buf.as_ptr().add(at).cast::<Header>()).reclen() as usize }
}

/// How many records `encode` would fit into a buffer of `len` bytes.
pub fn count_for(entries: &[Entry], len: usize) -> usize {
    let mut used = 0usize;
    let mut n = 0usize;
    for e in entries {
        let want = reclen_for(e.name.len());
        if used + want > len {
            break;
        }
        used += want;
        n += 1;
    }
    n
}
