//! `DT_SYMTAB` and `DT_STRTAB`, plus the two hash tables.
//!
//! The symbol table turns an offset in a relocation into an address in some
//! object, so this is the piece the loader leans on hardest. It is also the
//! piece that differs most between binaries: a stripped `/bin/ls` has only
//! `DT_GNU_HASH` and no `DT_HASH` at all, while a `libc.so.6` has both.

use super::{Error, Result};

/// Size of `Elf64_Sym`.
pub const SYM_SIZE: usize = 24;

/// `st_shndx` when the symbol is undefined.
pub const SHN_UNDEF: u16 = 0;
/// `st_shndx` when the symbol is absolute, i.e. not relocatable.
pub const SHN_ABS: u16 = 0xfff1;

/// `STB_*`: the linkage of a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StBind {
    /// `STB_LOCAL`: invisible to every other object.
    Local,
    /// `STB_GLOBAL`: visible to every object in the link.
    Global,
    /// `STB_WEAK`: a definition another object may override.
    Weak,
    /// Anything else.
    Other(u8),
}

/// `STT_*`: what kind of thing a symbol names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StType {
    /// `STT_NOTYPE`.
    NoType,
    /// `STT_OBJECT`.
    Object,
    /// `STT_FUNC`.
    Func,
    /// `STT_SECTION`.
    Section,
    /// `STT_FILE`.
    File,
    /// `STT_TLS`: a thread-local. Relocates differently from everything else,
    /// and is the single easiest thing in this format to get wrong.
    Tls,
    /// Anything else.
    Other(u8),
}

/// `st_other`'s visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Default: exported.
    Default,
    /// Internal: not visible to other objects.
    Internal,
    /// Hidden: still in the dynamic table, but not exported.
    Hidden,
    /// Protected.
    Protected,
}

/// One `Elf64_Sym`.
#[derive(Debug, Clone, Copy)]
pub struct Sym {
    /// `st_name`: offset into `DT_STRTAB`.
    pub name_off: u32,
    /// `st_info >> 4`.
    pub bind: StBind,
    /// `st_info & 0xf`.
    pub kind: StType,
    /// `st_other & 3`.
    pub visibility: Visibility,
    /// `st_shndx`.
    pub shndx: u16,
    /// `st_value`, before any load base is added.
    pub value: u64,
    /// `st_size`.
    pub size: u64,
}

impl Sym {
    /// Is this symbol defined *here*?
    ///
    /// `STB_WEAK` and `STB_GLOBAL` both count. The distinction between them
    /// matters during lookup, not here.
    pub fn is_defined(&self) -> bool {
        self.shndx != SHN_UNDEF
    }

    /// Is this a weak definition?
    pub fn is_weak(&self) -> bool {
        self.bind == StBind::Weak
    }

    /// Is this symbol a thread-local?
    pub fn is_tls(&self) -> bool {
        self.kind == StType::Tls
    }
}

/// A section index, as stored in `st_shndx`.
pub type Section = u16;


/// The symbol table plus the string table it indexes into.
///
/// Both addresses are *link-time* addresses: the caller adds the load base when
/// it wants runtime ones. Keeping that split explicit is what lets the same table
/// code serve both the relocation pass and the lookup pass.
pub struct SymTable<'a> {
    syms: &'a [u8],
    strs: &'a [u8],
    count: usize,
}

impl<'a> SymTable<'a> {
    /// Build a view over `count` entries of `Elf64_Sym` and the string table `strs`.
    pub fn new(syms: &'a [u8], strs: &'a [u8], count: usize) -> Result<Self> {
        let need = count.checked_mul(SYM_SIZE).ok_or(Error::OutOfBounds)?;
        if need > syms.len() {
            return Err(Error::BadOffset);
        }
        Ok(Self { syms: &syms[..need], strs, count })
    }

    /// How many symbols.
    pub fn len(&self) -> usize {
        self.count
    }

    /// True when the table is empty: the object exports nothing, so nothing can
    /// link against it.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// The `i`th symbol, or `None` if out of range.
    pub fn get(&self, i: usize) -> Option<Sym> {
        if i >= self.count {
            return None;
        }
        let o = i * SYM_SIZE;
        let d = &self.syms[o..o + SYM_SIZE];
        let info = d[4];
        Some(Sym {
            name_off: u32::from_le_bytes([d[0], d[1], d[2], d[3]]),
            bind: match info >> 4 {
                0 => StBind::Local,
                1 => StBind::Global,
                2 => StBind::Weak,
                other => StBind::Other(other),
            },
            kind: match info & 0xf {
                0 => StType::NoType,
                1 => StType::Object,
                2 => StType::Func,
                3 => StType::Section,
                4 => StType::File,
                6 => StType::Tls,
                other => StType::Other(other),
            },
            visibility: match d[5] & 0x3 {
                0 => Visibility::Default,
                1 => Visibility::Internal,
                2 => Visibility::Hidden,
                _ => Visibility::Protected,
            },
            shndx: u16::from_le_bytes([d[6], d[7]]),
            value: rd_u64(d, 8),
            size: rd_u64(d, 16),
        })
    }

    /// The name of the `i`th symbol, up to the NUL.
    pub fn name(&self, i: usize) -> Option<&'a [u8]> {
        self.string(self.get(i)?.name_off)
    }

    /// The name of the `i`th symbol, or `""` if absent or not UTF-8.
    pub fn name_str(&self, i: usize) -> &'a str {
        match self.name(i) {
            Some(b) => core::str::from_utf8(b).unwrap_or(""),
            None => "",
        }
    }

    /// The string at `off` in `DT_STRTAB`, up to the NUL.
    pub fn string(&self, off: u32) -> Option<&'a [u8]> {
        let bytes = self.strs.get(off as usize..)?;
        let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        Some(&bytes[..len])
    }

    /// Iterate over `(index, symbol)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (usize, Sym)> + '_ {
        (0..self.count).filter_map(|i| self.get(i).map(|s| (i, s)))
    }

    /// Find a defined symbol with this name, by linear scan.
    ///
    /// `DT_GNU_HASH` is faster and the loader will use it once symbol resolution
    /// exists; the scan is what comes first because it is the version you can
    /// check by eye. It skips `STB_LOCAL`, which must never satisfy another
    /// object's reference — a lookup that returns a local symbol yields a loader
    /// that works until two objects happen to share a name.
    pub fn lookup(&self, name: &[u8]) -> Option<Sym> {
        self.iter()
            .find(|(_, s)| {
                s.is_defined() && s.bind != StBind::Local && self.string(s.name_off) == Some(name)
            })
            .map(|(_, s)| s)
    }

    /// The index of the first defined symbol with this name.
    ///
    /// The loader needs the index, not the symbol: `r_info` addresses symbols by
    /// position, and a relocator that looks a symbol up by name to find its
    /// index has already thrown away the only thing that was unambiguous.
    pub fn lookup_index(&self, name: &[u8]) -> Option<usize> {
        self.iter()
            .find(|(_, s)| {
                s.is_defined() && s.bind != StBind::Local && self.string(s.name_off) == Some(name)
            })
            .map(|(i, _)| i)
    }
}

fn rd_u64(d: &[u8], o: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[o..o + 8]);
    u64::from_le_bytes(b)
}
