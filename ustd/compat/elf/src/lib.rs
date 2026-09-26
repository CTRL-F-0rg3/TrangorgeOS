//! `tgs-elf` — ELF64 parsing for the Linux compatibility layer.
//!
//! A PIE executable and a shared object are the same format, so one parser
//! covers `/bin/ls`, `libc.so.6` and anything else the loader is asked to load.
//! Nothing here maps memory or resolves a symbol; this crate only turns bytes
//! into typed views over them, which is what makes it testable against a real
//! stripped binary instead of a fixture someone wrote to match the parser.
//!
//! # Zero-copy on purpose
//!
//! Every view borrows from the file image. A loader that copies a symbol table it
//! is about to walk twice is a loader that has to trust its own arithmetic, and
//! that arithmetic is the part which is easy to get wrong on a 32-bit offset in a
//! 64-bit address space. The views below are all checked, and a malformed image
//! produces an `Err` rather than a wild pointer.
//!
//! # What is *not* here
//!
//! [`reloc`] is separated out because it is the part with its own failure modes,
//! and because it needs the loader's memory to already be mapped. [`dynamic`]
//! walks `PT_DYNAMIC`, which is what tells the loader what to do in the first
//! place.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

use core::fmt;

pub mod dynamic;
pub mod reloc;

mod header;
mod phdr;
mod symbol;

pub use dynamic::{Dynamic, Needed, Tag};
pub use header::{Class, Ehdr, Endian, Ident, Machine, Type};
pub use phdr::{Flags, Phdr, Phdrs};
pub use reloc::{Rela, RelaIter, RelocType};
pub use symbol::{Section, Sym, SymIter, StBind, StType, Visibility};

/// Everything that can be wrong with an image.
///
/// One enum rather than a `Result<_, &'static str>` per field, so a caller can
/// report *which* check failed. A loader that says "bad elf" is a loader whose
/// users cannot debug anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Fewer than `EI_NIDENT` bytes.
    Truncated,
    /// The first four bytes are not `\x7fELF`.
    NotElf,
    /// A 32-bit image. The x86_64-ABI is 64-bit only, and a 32-bit Linux binary
    /// wants a different relocation set and a different `struct stat`.
    Not64,
    /// Big-endian. Same reasoning: the x86_64-ABI is little-endian only.
    NotLittleEndian,
    /// An ELF version other than 1.
    BadVersion(u8),
    /// `e_machine` is not `EM_X86_64`.
    WrongMachine(u16),
    /// A header field points outside the image.
    OutOfBounds,
    /// A program header is not `Elf64_Phdr` bytes long.
    BadPhdrSize(u16),
    /// No `PT_LOAD`, so there is nothing to map.
    NoLoadableSegment,
    /// A `PT_LOAD`'s `p_filesz` exceeds its `p_memsz`, which would mean the
    /// loader has to invent the tail.
    BadSegmentSize,
    /// Two segments overlap in the address space. Valid images do not do this,
    /// and an overlap means the load order decides the winner.
    OverlappingSegments,
    /// A dynamic entry with a tag the parser does not know how to size.
    BadDynamicEntry,
    /// A symbol or string offset outside the image.
    BadOffset,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Truncated => f.write_str("file is shorter than an ELF ident"),
            Error::NotElf => f.write_str("no ELF magic"),
            Error::Not64 => f.write_str("not a 64-bit ELF"),
            Error::NotLittleEndian => f.write_str("not little-endian"),
            Error::BadVersion(v) => write!(f, "ELF version {v}"),
            Error::WrongMachine(m) => write!(f, "e_machine {m:#06x} is not EM_X86_64"),
            Error::OutOfBounds => f.write_str("a header field points outside the image"),
            Error::BadPhdrSize(n) => write!(f, "e_phentsize {n} is not 56"),
            Error::NoLoadableSegment => f.write_str("no PT_LOAD segment"),
            Error::BadSegmentSize => f.write_str("p_filesz exceeds p_memsz"),
            Error::OverlappingSegments => f.write_str("two PT_LOAD segments overlap"),
            Error::BadDynamicEntry => f.write_str("a PT_DYNAMIC entry is misaligned"),
            Error::BadOffset => f.write_str("an offset points outside the image"),
        }
    }
}

/// The result of parsing.
pub type Result<T> = core::result::Result<T, Error>;

