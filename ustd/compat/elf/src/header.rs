//! The `Elf64_Ehdr` and the checks that reject an image the loader cannot use.

use super::{Error, Result};

/// Size of the `e_ident` array.
pub const EI_NIDENT: usize = 16;
const EI_CLASS: usize = 4;
const EI_DATA: usize = 5;
const EI_VERSION: usize = 6;
const EHDR_SIZE: usize = 64;

/// `EI_CLASS`: an address width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// `ELFCLASS32`.
    Bits32,
    /// `ELFCLASS64`.
    Bits64,
}

/// `EI_DATA`: byte order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    /// `ELFDATA2LSB`.
    Little,
    /// `ELFDATA2MSB`.
    Big,
}

/// `e_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    /// `ET_EXEC`: a fixed-address executable. Loadable, but no ASLR.
    Exec,
    /// `ET_DYN`: a shared object *or* a PIE executable. This is the interesting
    /// case, because the loader has to choose a base and relocate against it.
    Dyn,
    /// `ET_CORE`.
    Core,
    /// `ET_REL`: an unlinked object. The loader has nothing to do with one.
    Rel,
    /// Anything else.
    Other(u16),
}

/// `e_machine`.
pub type Machine = u16;

/// `EM_X86_64`.
pub const EM_X86_64: Machine = 62;

/// The first 16 bytes of the file, decoded far enough to reject a 32-bit or
/// big-endian image before any pointer arithmetic happens.
#[derive(Debug, Clone, Copy)]
pub struct Ident {
    /// `EI_CLASS`.
    pub class: Class,
    /// `EI_DATA`.
    pub endian: Endian,
    /// `EI_VERSION`.
    pub version: u8,
}

/// A parsed `Elf64_Ehdr`.
///
/// Fields are read with explicit little-endian loads rather than by transmuting
/// a byte slice. That is slower and it matters: the alternative makes every
/// downstream `&Phdr`/`&Sym` a pointer into unaligned memory, and on x86_64 that
/// is fine until the image is 32-bit or a struct field straddles a boundary.
#[derive(Debug, Clone, Copy)]
pub struct Ehdr {
    /// `EI_CLASS` and friends.
    pub ident: Ident,
    /// `e_type`.
    pub kind: Type,
    /// `e_machine`.
    pub machine: Machine,
    /// `e_entry`: the entry point for an executable, or `0` for a library.
    pub entry: u64,
    /// `e_phoff`: the program header table's file offset.
    pub phoff: u64,
    /// `e_phnum`: how many program headers there are.
    pub phnum: u16,
    /// `e_phentsize`: the size of one program header.
    pub phentsize: u16,
}

impl Ehdr {
    /// Parse the header at the start of `data`.
    ///
    /// `size` is the length of the image, so that a truncated file reports
    /// [`Error::Truncated`] instead of being read past. The `data` bound is
    /// checked against the fixed header size only; everything variable-length is
    /// checked by the parser that owns it ([`Phdrs`](super::Phdrs)).
    pub fn parse(data: &[u8], size: usize) -> Result<Self> {
        if size < EI_NIDENT || data.len() < EI_NIDENT {
            return Err(Error::Truncated);
        }
        if &data[0..4] != b"\x7fELF" {
            return Err(Error::NotElf);
        }

        // A 32-bit image is rejected on the x86_64-ABI grounds, not because the
        // magic is unfamiliar: the reason it cannot be loaded is that its
        // relocation set and its `struct stat` are different ones.
        let class = match data[EI_CLASS] {
            2 => Class::Bits64,
            _ => return Err(Error::Not64),
        };
        let endian = match data[EI_DATA] {
            1 => Endian::Little,
            _ => return Err(Error::NotLittleEndian),
        };
        let version = data[EI_VERSION];
        if version != 1 {
            return Err(Error::BadVersion(version));
        }
        if size < EHDR_SIZE || data.len() < EHDR_SIZE {
            return Err(Error::Truncated);
        }

        let machine = u16::from_le_bytes([data[18], data[19]]);
        if machine != EM_X86_64 {
            return Err(Error::WrongMachine(machine));
        }

        // `ET_REL` and `ET_NONE` are both 0 on the wire; calling 0 `Rel` is
        // right for a `.o`, and the loader rejects `Rel` regardless.
        let kind = match u16::from_le_bytes([data[16], data[17]]) {
            0 => Type::Rel,
            1 | 2 => Type::Exec,
            3 => Type::Dyn,
            4 => Type::Core,
            other => Type::Other(other),
        };

        Ok(Ehdr {
            ident: Ident { class, endian, version },
            kind,
            machine,
            entry: read_u64(data, 24),
            phoff: read_u64(data, 32),
            phentsize: u16::from_le_bytes([data[54], data[55]]),
            phnum: u16::from_le_bytes([data[56], data[57]]),
        })
    }

    /// Is this an image whose addresses the loader may relocate?
    ///
    /// True for `ET_DYN`, which covers both PIEs and shared objects.
    pub fn needs_base(&self) -> bool {
        matches!(self.kind, Type::Dyn)
    }
}

fn read_u64(data: &[u8], off: usize) -> u64 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[off..off + 8]);
    u64::from_le_bytes(buf)
}
