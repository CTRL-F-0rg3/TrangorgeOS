//! Program headers: what to map, and what to run.

use super::header::Ehdr;
use super::{Error, Result};
use alloc::vec::Vec;

/// Size of `Elf64_Phdr`.
pub const PHDR_SIZE: usize = 56;

/// `p_type` values.
///
/// Not every one is used yet, and that is the point: they are the vocabulary the
/// loader needs before it can act. The `dead_code` allowance is for the ones the
/// kernel-side work will need, so their presence is not mistaken for dead code.
#[allow(dead_code)]
pub mod p_type {
    /// `PT_NULL`.
    pub const NULL: u32 = 0;
    /// `PT_LOAD`: the only kind that allocates anything.
    pub const LOAD: u32 = 1;
    /// `PT_DYNAMIC`: the `.dynamic` table that describes the rest.
    pub const DYNAMIC: u32 = 2;
    /// `PT_INTERP`: the path to the dynamic linker, for an executable.
    pub const INTERP: u32 = 3;
    /// `PT_NOTE`.
    pub const NOTE: u32 = 4;
    /// `PT_PHDR`: where this very table was placed in memory.
    pub const PHDR: u32 = 6;
    /// `PT_TLS`: the thread-local template, the block `fs_base` points into.
    pub const TLS: u32 = 7;
    /// `PT_GNU_STACK`: whether the main thread's stack is executable.
    pub const GNU_STACK: u32 = 0x6474_e551;
    /// `PT_GNU_RELRO`: becomes read-only after relocation.
    pub const GNU_RELRO: u32 = 0x6474_e552;
}

/// `p_flags`, as a permission set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flags(pub u32);

impl Flags {
    /// `PF_R`.
    pub const R: u32 = 4;
    /// `PF_W`.
    pub const W: u32 = 2;
    /// `PF_X`.
    pub const X: u32 = 1;

    /// Whether the segment is readable.
    pub fn readable(self) -> bool {
        self.0 & Self::R != 0
    }
    /// Whether the segment is writable.
    pub fn writable(self) -> bool {
        self.0 & Self::W != 0
    }
    /// Whether the segment is executable.
    pub fn executable(self) -> bool {
        self.0 & Self::X != 0
    }
}

/// One program header, decoded.
#[derive(Debug, Clone, Copy)]
pub struct Phdr {
    /// `p_type`.
    pub kind: u32,
    /// `p_flags`.
    pub flags: Flags,
    /// `p_offset` in the file.
    pub offset: u64,
    /// `p_vaddr`, the address *before* a base is chosen.
    pub vaddr: u64,
    /// `p_filesz`: bytes present in the file.
    pub filesz: u64,
    /// `p_memsz`: bytes occupied in memory. `memsz > filesz` is the `.bss` tail.
    pub memsz: u64,
    /// `p_align`.
    pub align: u64,
}

impl Phdr {
    /// Is this a `PT_LOAD`?
    pub fn is_load(&self) -> bool {
        self.kind == p_type::LOAD
    }

    /// The end of this segment in the address space, exclusive.
    pub fn vaddr_end(&self) -> u64 {
        self.vaddr.saturating_add(self.memsz)
    }

    /// The end of this segment in the file, exclusive.
    pub fn file_end(&self) -> u64 {
        self.offset.saturating_add(self.filesz)
    }

    /// The page this segment starts on.
    ///
    /// The loader maps whole pages, so this is the address the mapping must be
    /// created for, not `p_vaddr`.
    pub fn page_start(&self) -> u64 {
        self.vaddr & !0xfff
    }
}

/// A checked view over the program header table.
///
/// Owns nothing: it borrows the header table's bytes. The bounds are checked
/// once, in [`Phdrs::parse`], so that [`Phdrs::get`] is infallible and the
/// loader's inner loop needs no error handling.
pub struct Phdrs<'a> {
    table: &'a [u8],
    count: usize,
}

impl<'a> Phdrs<'a> {
    /// Read the program header table described by `ehdr` out of `data`.
    ///
    /// `size` is the image length, so a header claiming a table past the end of
    /// the file is rejected here rather than producing a read past the buffer.
    pub fn parse(data: &'a [u8], size: usize, ehdr: &Ehdr) -> Result<Self> {
        if ehdr.phentsize as usize != PHDR_SIZE {
            return Err(Error::BadPhdrSize(ehdr.phentsize));
        }
        let count = ehdr.phnum as usize;
        let start = ehdr.phoff as usize;
        let len = count.checked_mul(PHDR_SIZE).ok_or(Error::OutOfBounds)?;
        let end = start.checked_add(len).ok_or(Error::OutOfBounds)?;
        if end > size || end > data.len() {
            return Err(Error::OutOfBounds);
        }
        Ok(Self { table: &data[start..end], count })
    }

    /// How many program headers there are.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True when there are no program headers at all, which is a broken image.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// The `i`th program header, or `None` if out of range.
    pub fn get(&self, i: usize) -> Option<Phdr> {
        if i >= self.count {
            return None;
        }
        let o = i * PHDR_SIZE;
        let d = &self.table[o..o + PHDR_SIZE];
        Some(Phdr {
            kind: rd_u32(d, 0),
            flags: Flags(rd_u32(d, 4)),
            offset: rd_u64(d, 8),
            vaddr: rd_u64(d, 16),
            filesz: rd_u64(d, 32),
            memsz: rd_u64(d, 40),
            align: rd_u64(d, 48),
        })
    }

    /// Iterate over every program header.
    pub fn iter(&self) -> impl Iterator<Item = Phdr> + '_ {
        (0..self.count).filter_map(|i| self.get(i))
    }

    /// The first program header with the given `p_type`.
    pub fn find(&self, kind: u32) -> Option<Phdr> {
        self.iter().find(|p| p.kind == kind)
    }

    /// Every `PT_LOAD` worth mapping, in file order.
    ///
    /// Two properties are checked here because the loader depends on both and
    /// neither is cheap to discover later:
    ///
    /// * `p_filesz <= p_memsz`. Otherwise the loader would have to invent the
    ///   tail of a segment, and "invent" means a page of zeroes the linker never
    ///   asked for.
    /// * no two segments overlap. Overlap is not theoretical: it means the *load
    ///   order* decides which mapping wins, so a silent overlap shows up as
    ///   corrupted globals under one base and not under another.
    pub fn loadables(&self) -> Result<Vec<Phdr>> {
        let mut out = Vec::new();
        for p in self.iter().filter(Phdr::is_load) {
            if p.filesz > p.memsz {
                return Err(Error::BadSegmentSize);
            }
            if p.memsz == 0 {
                continue;
            }
            out.push(p);
        }
        if out.is_empty() {
            return Err(Error::NoLoadableSegment);
        }
        let mut sorted = out.clone();
        sorted.sort_by_key(|p| p.vaddr);
        for w in sorted.windows(2) {
            if w[0].vaddr_end() > w[1].vaddr {
                return Err(Error::OverlappingSegments);
            }
        }
        Ok(out)
    }

    /// The bytes of the `PT_INTERP` segment, with the trailing NUL removed.
    ///
    /// For `/bin/ls` this is `/lib64/ld-linux-x86-64.so.2`. Surfacing it before
    /// anything is mapped is what lets the loader reject an unrunnable image
    /// while the failure still has a useful message.
    pub fn interp<'b>(&self, data: &'b [u8]) -> Option<&'b [u8]> {
        let p = self.find(p_type::INTERP)?;
        let start = p.offset as usize;
        let end = p.file_end() as usize;
        if start > data.len() || start > end || end > data.len() {
            return None;
        }
        let bytes = &data[start..end];
        let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        Some(&bytes[..len])
    }
}

fn rd_u32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

fn rd_u64(d: &[u8], o: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[o..o + 8]);
    u64::from_le_bytes(b)
}
