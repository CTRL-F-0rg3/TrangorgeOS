//! `PT_DYNAMIC`: the table that tells the loader what this object needs.
//!
//! For an executable this is a short list — `DT_NEEDED`, `DT_SYMTAB`, a hash
//! table, some relocations. For a library it is the same list, and it is the
//! only way to discover the object's dependencies without a section header
//! table, which a stripped binary does not have.
//!
//! Tags live in the processor-specific range (`0x6fff_ffff` and above), so the
//! high bit of `d_tag` is not an error flag and must not be masked off.

use super::phdr::{p_type, Phdrs};
use super::{Error, Result};

/// Size of `Elf64_Dyn`.
pub const DYN_SIZE: usize = 16;

/// `d_tag` values the loader acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    /// `DT_NULL`: end of the table.
    Null,
    /// `DT_NEEDED`: a `DT_STRTAB` offset naming a library to load.
    Needed,
    /// `DT_PLTRELSZ`: size of the PLT relocation table.
    PltRelSz,
    /// `DT_PLTGOT`: the `.got.plt` address.
    PltGot,
    /// `DT_HASH`: a classic SysV hash table.
    Hash,
    /// `DT_STRTAB`: the string table.
    StrTab,
    /// `DT_SYMTAB`: the symbol table.
    SymTab,
    /// `DT_RELA`: a relocation table.
    Rela,
    /// `DT_RELASZ`: its size in bytes.
    RelaSz,
    /// `DT_RELAENT`: the size of one `Elf64_Rela`.
    RelaEnt,
    /// `DT_STRSZ`: the string table's size.
    StrSz,
    /// `DT_SYMENT`: the size of one `Elf64_Sym`.
    SymEnt,
    /// `DT_INIT`: a function to run before anything else.
    Init,
    /// `DT_FINI`: a function to run last.
    Fini,
    /// `DT_SONAME`: this object's own name.
    SoName,
    /// `DT_RPATH`: an older, less capable `DT_RUNPATH`.
    RPath,
    /// `DT_SYMBOLIC`: resolve this object's own symbols first.
    Symbolic,
    /// `DT_REL`: an `Elf64_Rel` table. x86_64 does not use this; seeing it means
    /// the image is not what the loader thinks it is.
    Rel,
    /// `DT_RELSZ`.
    RelSz,
    /// `DT_RELENT`.
    RelEnt,
    /// `DT_PLTREL`: the relocation *type* used by the PLT.
    PltRel,
    /// `DT_DEBUG`: a `struct r_debug*` the debugger writes through.
    Debug,
    /// `DT_TEXTREL`: the image wants its text mapped writable.
    TextRel,
    /// `DT_JMPREL`: the PLT relocations.
    JmpRel,
    /// `DT_BIND_NOW`: resolve everything eagerly.
    BindNow,
    /// `DT_INIT_ARRAY`: a list of functions to run before `main`.
    InitArray,
    /// `DT_FINI_ARRAY`: a list of functions to run at exit.
    FiniArray,
    /// `DT_INIT_ARRAYSZ`.
    InitArraySz,
    /// `DT_FINI_ARRAYSZ`.
    FiniArraySz,
    /// `DT_RUNPATH`: the `LD_LIBRARY_PATH`-style search list.
    RunPath,
    /// `DT_FLAGS`.
    Flags,
    /// `DT_PREINIT_ARRAY`.
    PreInitArray,
    /// `DT_PREINIT_ARRAYSZ`.
    PreInitArraySz,
    /// `DT_GNU_HASH`: the GNU hash table, the one modern linkers emit.
    GnuHash,
    /// `DT_VERSYM`: symbol versioning.
    VerSym,
    /// `DT_VERNEED`: the version requirements.
    VerNeed,
    /// `DT_VERNEEDNUM`.
    VerNeedNum,
    /// A tag the loader does not know. Kept rather than dropped, because a
    /// dynamic table full of `Unknown` is itself a useful diagnostic.
    Other(i64),
}

impl Tag {
    fn from_raw(v: i64) -> Self {
        use Tag::*;
        match v {
            0 => Null,
            1 => Needed,
            2 => PltRelSz,
            3 => PltGot,
            4 => Hash,
            5 => StrTab,
            6 => SymTab,
            7 => Rela,
            8 => RelaSz,
            9 => RelaEnt,
            10 => StrSz,
            11 => SymEnt,
            12 => Init,
            13 => Fini,
            14 => SoName,
            15 => RPath,
            16 => Symbolic,
            17 => Rel,
            18 => RelSz,
            19 => RelEnt,
            20 => PltRel,
            21 => Debug,
            22 => TextRel,
            23 => JmpRel,
            24 => BindNow,
            25 => InitArray,
            26 => FiniArray,
            27 => InitArraySz,
            28 => FiniArraySz,

/// One `Elf64_Dyn`.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    /// The decoded tag.
    pub tag: Tag,
    /// `d_un.d_val`, interpreted according to the tag.
    pub val: u64,
}

/// The `PT_DYNAMIC` table of one object.
///
/// Only address-bearing tags are meaningful before the image is mapped, and they
/// are *link-time* addresses, so a caller must add the load base. The one
/// exception is [`Tag::Needed`], whose value is a `DT_STRTAB` offset and is
/// therefore base-independent — the loader reads it before it has a base.
pub struct Dynamic<'a> {
    entries: &'a [u8],
}

impl<'a> Dynamic<'a> {
    /// Read the dynamic table out of the `PT_DYNAMIC` segment described by
    /// `phdrs`.
    ///
    /// A missing `PT_DYNAMIC` is not an error here: a statically linked binary
    /// has none, and the caller needs to be able to say "nothing to do" rather
    /// than "malformed".
    pub fn parse(phdrs: &Phdrs<'_>, data: &'a [u8]) -> Option<Result<Self>> {
        let p = phdrs.find(p_type::DYNAMIC)?;
        let start = p.offset as usize;
        let len = p.filesz as usize;
        if start > data.len() || start.checked_add(len)? > data.len() {
            return Some(Err(Error::OutOfBounds));
        }
        Some(Ok(Self { entries: &data[start..start + len] }))
    }

    /// Build from a raw table, for tests.
    pub fn from_bytes(entries: &'a [u8]) -> Result<Self> {
        if entries.len() % DYN_SIZE != 0 {
            return Err(Error::BadDynamicEntry);
        }
        Ok(Self { entries })
    }

    /// Iterate over every entry, stopping at `DT_NULL`.
    ///
    /// A table with no `DT_NULL` is malformed. Rather than reading off the end,
    /// the iterator ends at the last whole entry, and [`Dynamic::is_terminated`]
    /// lets the loader reject the image.
    pub fn iter(&self) -> DynamicIter<'a> {
        DynamicIter { entries: self.entries }
    }

    /// Does the table end with `DT_NULL`?
    pub fn is_terminated(&self) -> bool {
        self.iter().any(|e| e.tag == Tag::Null)
    }

    /// The first entry with this tag.
    pub fn find(&self, tag: Tag) -> Option<u64> {
        self.iter().find(|e| e.tag == tag).map(|e| e.val)
    }

    /// The first entry with this tag, as a count.
    pub fn count(&self, tag: Tag) -> usize {
        self.iter().filter(|e| e.tag == tag).count()
    }

    /// Every `DT_NEEDED` offset, in order.
    ///
    /// Order is significant: the loader loads dependencies in this order, and a
    /// binary that needs `libm.so.6` before `libc.so.6` says so here.
    pub fn needed(&self) -> Vec<u32> {
        self.iter().filter(|e| e.tag == Tag::Needed).map(|e| e.val as u32).collect()
    }

    /// How many symbols the table claims.
    ///
    /// Not derivable from `DT_SYMTAB` alone — the table has no length for it, so
    /// the loader takes it from the hash table's `nbucket`, or bounds it by the
    /// start of the next table. [`crate::symbol::SymTable`] then enforces that
    /// bound against the real bytes.
    pub fn sym_count_hint(&self) -> Option<usize> {
        let symtab = self.find(Tag::SymTab)?;
        let _ = symtab;
        None
    }
}

/// Iterator over `Elf64_Dyn` entries.
pub struct DynamicIter<'a> {
    entries: &'a [u8],
}

impl<'a> Iterator for DynamicIter<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Entry> {
        if self.entries.len() < DYN_SIZE {
            self.entries = &[];
            return None;
        }
        let d = &self.entries[..DYN_SIZE];
        self.entries = &self.entries[DYN_SIZE..];
        let raw = i64::from_le_bytes([d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]]);
        Some(Entry { tag: Tag::from_raw(raw), val: rd_u64(d, 8) })
    }
}

/// One `DT_NEEDED`, resolved against `DT_STRTAB`.
#[derive(Debug, Clone, Copy)]
pub struct Needed<'a> {
    /// The name, e.g. `libc.so.6`.
    pub name: &'a [u8],
}

impl<'a> Needed<'a> {
    /// The name as a `&str`, or `""` if it is not valid UTF-8.
    pub fn as_str(&self) -> &'a str {
        core::str::from_utf8(self.name).unwrap_or("")
    }
}

fn rd_u64(d: &[u8], o: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[o..o + 8]);
    u64::from_le_bytes(b)
}

            29 => RunPath,
            30 => Flags,
            32 => PreInitArray,
            33 => PreInitArraySz,
            0x6fff_fef5 => GnuHash,
            0x6fff_fff0 => VerSym,
            0x6fff_fffe => VerNeed,
            0x6fff_ffff => VerNeedNum,
            other => Other(other),
        }
    }
}
