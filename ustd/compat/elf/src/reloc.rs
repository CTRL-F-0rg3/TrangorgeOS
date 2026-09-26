//! `Elf64_Rela` and the x86-64 relocation types.
//!
//! This is where a PIE either works or jumps into page zero. A `ET_DYN` image is
//! full of `R_X86_64_RELATIVE` entries, one per pointer-sized global that has no
//! name — a vtable, a `.init_array` entry, a pointer to a string. Until the loader
//! writes `base + addend` into each of them, the image contains only the
//! addends, which are small numbers. Skipping this pass produces a program that
//! starts, crashes on the first function call, and gives no clue why.

use alloc::vec::Vec;
use super::phdr::Phdr;
use super::symbol::{StType, Sym};
use super::{Error, RelocKindHint, Result};

/// Size of `Elf64_Rela`.
pub const RELA_SIZE: usize = 24;

/// `R_X86_64_*` relocation types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelocType {
    /// `R_X86_64_NONE`: padding.
    None,
    /// `R_X86_64_64`: `S + A`.
    D64,
    /// `R_X86_64_PC32`: `S + A - P`, truncated to 32 bits.
    Pc32,
    /// `R_X86_64_PLT32`: `L + A - P`, where `L` is the PLT entry.
    Plt32,
    /// `R_X86_64_GLOB_DAT`: a resolved address in a GOT slot.
    GlobDat,
    /// `R_X86_64_JUMP_SLOT`: a resolved function address in a GOT slot.
    ///
    /// The difference from `GlobDat` is entirely about timing: a `JumpSlot` is
    /// written the first time the function is called, by the PLT stub.
    JumpSlot,
    /// `R_X86_64_RELATIVE`: `B + A`. No symbol, no lookup — the common case.
    Relative,
    /// `R_X86_64_COPY`: an executable's copy of a shared object's data, so the
    /// loader has to *make* a copy rather than point at one.
    Copy,
    /// `R_X86_64_TPOFF64`: an offset from `fs_base`. Needs TLS to be set up.
    Tpoff64,
    /// `R_X86_64_DTPOFF64`: an offset into the dynamic TLS block.
    Dtpoff64,
    /// `R_X86_64_IRELATIVE`: a resolver's return value is the address. This is
    /// how C++ static initialisers run.
    IRelative,
    /// `R_X86_64_TLSDESC`: a thread-local descriptor call.
    TlsDesc,
    /// `R_X86_64_GOTPCREL`: a 32-bit GOT offset.
    GotPcRel,
    /// `R_X86_64_SIZE32`/`SIZE64`.
    Size,
    /// A type the loader does not implement.
    Other(u32),
}

impl RelocType {
    /// Decode the low 32 bits of `r_info`.
    pub fn from_raw(v: u32) -> Self {
        use RelocType::*;
        match v {
            0 => None,
            1 => D64,
            2 => Pc32,
            4 => Plt32,
            5 => Copy,
            6 => GlobDat,
            7 => JumpSlot,
            8 => Relative,
            9 => GotPcRel,
            17 => Dtpoff64,
            18 => Tpoff64,
            32 | 33 => Size,
            35 | 36 => TlsDesc,
            37 => IRelative,
            other => Other(other),
        }
    }

    /// The raw value.
    pub fn raw(self) -> u32 {
        use RelocType::*;
        match self {
            None => 0,
            D64 => 1,
            Pc32 => 2,
            Plt32 => 4,
            Copy => 5,
            GlobDat => 6,
            JumpSlot => 7,
            Relative => 8,
            GotPcRel => 9,
            Dtpoff64 => 17,
            Tpoff64 => 18,
            Size => 32,
            TlsDesc => 35,
            IRelative => 37,
            Other(v) => v,
        }
    }
}


/// One `Elf64_Rela`.
#[derive(Debug, Clone, Copy)]
pub struct Rela {
    /// `r_offset`: where to write, as a *link-time* address.
    pub offset: u64,
    /// `r_info`: type in the low 32 bits, symbol index in the high 32.
    pub info: u64,
    /// `r_addend`, a signed offset.
    pub addend: i64,
}

impl Rela {
    /// The relocation type.
    pub fn kind(&self) -> RelocType {
        RelocType::from_raw(self.info as u32)
    }

    /// The symbol index, or 0 for a symbol-less relocation like `Relative`.
    pub fn sym_index(&self) -> u32 {
        (self.info >> 32) as u32
    }

    /// Build a relocation from a type and symbol index, for tests.
    pub fn new(offset: u64, kind: RelocType, sym: u32, addend: i64) -> Self {
        Self { offset, info: (u64::from(sym) << 32) | u64::from(kind.raw()), addend }
    }
}

/// Iterator over a `RELA` table.
pub struct RelaIter<'a> {
    entries: &'a [u8],
}

impl<'a> RelaIter<'a> {
    /// Iterate over `bytes`, which must be a whole number of `Elf64_Rela`.
    pub fn new(bytes: &'a [u8]) -> Result<Self> {
        if bytes.len() % RELA_SIZE != 0 {
            return Err(Error::BadOffset);
        }
        Ok(Self { entries: bytes })
    }

    /// Iterate using the table's declared `DT_RELAENT`.
    ///
    /// Checking `entsize` here rather than assuming it is what makes a table
    /// with a surprising `DT_RELAENT` visible instead of silently misread.
    pub fn with_ent(bytes: &'a [u8], entsize: usize) -> Result<Self> {
        if entsize != RELA_SIZE {
            return Err(Error::BadOffset);
        }
        Self::new(bytes)
    }
}

impl<'a> Iterator for RelaIter<'a> {
    type Item = Rela;

    fn next(&mut self) -> Option<Rela> {
        if self.entries.len() < RELA_SIZE {
            self.entries = &[];
            return None;
        }
        let d = &self.entries[..RELA_SIZE];
        self.entries = &self.entries[RELA_SIZE..];
        Some(Rela {
            offset: rd_u64(d, 0),
            info: rd_u64(d, 8),
            addend: rd_u64(d, 16) as i64,
        })
    }
}

/// What the loader resolved a symbol to, ready to be written.
#[derive(Debug, Clone, Copy)]
pub struct Resolved {
    /// The runtime address of the definition.
    pub value: u64,
    /// Whether the definition is weak.
    pub weak: bool,
    /// Whether the symbol is a thread-local.
    pub tls: bool,
}

/// The value a relocation computes, given its inputs.
///
/// The arithmetic is factored out of any mapped image so it can be tested
/// directly. `base` is the load base, `p` the link-time `r_offset`, and `sym`
/// the resolved symbol — `None` for a relocation that has none, which is every
/// `R_X86_64_RELATIVE` in the binary.
///
/// # Why the 32-bit types look wrong
///
/// `Pc32` and friends return `(S + A - P) as u32 as u64`. The cast is not
/// laziness. A PIE's `S + A - P` is negative — it is the distance from the slot
/// back to something in the same image — and the 32-bit truncation is what makes
/// that distance survive. The `base` is added to `P` and to `S` before the
/// subtraction precisely so the two cancel; leaving the base out of one side is
/// the classic way to produce a binary that relocates cleanly and then jumps
/// four gigabytes away.
pub fn compute(
    kind: RelocType,
    base: u64,
    p: u64,
    addend: i64,
    sym: Option<Resolved>,
) -> Result<u64> {
    let s = || sym.map(|r| r.value).ok_or(Error::MissingInput(RelocKindHint::UnresolvedSymbol));
    Ok(match kind {
        RelocType::None => 0,
        // The common case: no symbol, no lookup, just the base.
        RelocType::Relative => base.wrapping_add(addend as u64),
        RelocType::GlobDat | RelocType::JumpSlot | RelocType::D64 => s()?.wrapping_add(addend as u64),
        RelocType::Pc32 | RelocType::Plt32 | RelocType::GotPcRel | RelocType::Size => {
            let v = (s()?.wrapping_add(addend as u64) as i64)
                .wrapping_sub(base.wrapping_add(p) as i64);
            v as u32 as u64
        }
        // `Copy` is a whole-object operation the loader performs *before*
        // relocation, not a per-slot write.
        RelocType::Copy => return Err(Error::MissingInput(RelocKindHint::Unsupported)),
        // The resolver is at `S` in this image, so its runtime address is
        // `S + base`, with the addend applied on top.
        RelocType::IRelative => s()?.wrapping_add(addend as u64).wrapping_add(base),
        // These need `fs_base` and the TLS block layout, neither of which this
        // crate knows. The error says so, because "it crashed" does not.
        RelocType::Tpoff64 | RelocType::Dtpoff64 | RelocType::TlsDesc => {
            return Err(Error::MissingInput(RelocKindHint::NeedsTls))
        }
        RelocType::Other(_) => return Err(Error::MissingInput(RelocKindHint::Unsupported)),
    })
}

/// How many bytes this relocation writes into the slot.
///
/// Getting this wrong corrupts the neighbouring bytes, which is why it is a
/// function and not a `match` repeated at each call site.
pub fn slot_size(kind: RelocType) -> Option<usize> {
    // Every arm is fully qualified. `use RelocType::*` would bring a variant
    // named `None` into scope, and the resulting shadowing of `Option::None` is
    // exactly the kind of mistake that reads as a type error pointing nowhere.
    match kind {
        RelocType::Pc32 | RelocType::Plt32 | RelocType::GotPcRel | RelocType::Size => Some(4),
        RelocType::D64
        | RelocType::GlobDat
        | RelocType::JumpSlot
        | RelocType::Relative
        | RelocType::Copy
        | RelocType::Tpoff64
        | RelocType::Dtpoff64
        | RelocType::IRelative
        | RelocType::TlsDesc => Some(8),
        // `R_X86_64_NONE` writes nothing, which is what makes skipping it safe
        // rather than an error. An unknown type is not safe to guess at.
        RelocType::None | RelocType::Other(_) => None,
    }
}

/// Is this a thread-local relocation?
pub fn is_tls_reloc(kind: RelocType) -> bool {
    use RelocType::*;
    matches!(kind, Tpoff64 | Dtpoff64 | TlsDesc)
}

/// A TLS symbol's `st_value` is a *template offset*, not a runtime address.
///
/// This check exists to catch a loader that treats `R_X86_64_TPOFF64` as if it
/// were `R_X86_64_64`, which produces a program that runs and then reads the
/// wrong address the first time it touches a `thread_local!`.
pub fn tls_symbol_is_offset(sym: &Sym) -> bool {
    sym.kind == StType::Tls && sym.is_defined()
}

/// The `PT_LOAD` segment containing `addr`, if any.
///
/// The loader uses this to check that a relocation target is somewhere it has
/// actually mapped, which is a cheap way to catch a relocation applied against
/// the wrong object.
pub fn segment_containing(segments: &[Phdr], addr: u64) -> Option<&Phdr> {
    segments.iter().find(|p| p.is_load() && addr >= p.vaddr && addr < p.vaddr_end())
}

/// The high bit of a `DT_RELR` entry. Public because a test that builds a RELR
/// table by hand needs to set it, and a constant a test cannot reach is a
/// constant the test will not cover.
pub const RELR_BITMAP: u64 = 0x8000_0000_0000_0000;

/// Decodes a `DT_RELR` table into the relocation addresses it stands for.
///
/// # Why this exists at all
///
/// Since glibc 2.36 a `-z relr` binary stores its `R_X86_64_RELATIVE`
/// relocations *run-length encoded*, in a table of 8-byte words rather than
/// 24-byte `Elf64_Rela` entries. Every current `/bin/ls` has one; there is no
/// `DT_JMPREL` at all, because the binary is also linked `-z now`. A loader that
/// only understands `DT_RELA` will parse a modern binary, find no PLT
/// relocations, and then relocate nothing at all — the image keeps its link-time
/// addends and jumps into page zero on the first global access.
///
/// The encoding, in one place, since it is easier to state than to remember:
///
/// * Bit 63 set: the *first* entry only. Its low 63 bits are the address of the
///   first relocation, which is how the loader derives `DT_RELACOUNT`.
/// * Otherwise, entries alternate. An *even* entry is the address of a
///   relocation. The *odd* entry after it is a bitmap relative to that address,
///   where bit `i` means "also a relocation at `address + (i + 1) * 8`".
///
/// `base` is the load base, and every emitted address already includes it, so
/// the caller writes the result into the image without further arithmetic.
///
/// # What is verified, and what is not
///
/// **Treat this as unconfirmed beyond the first group.** Two tests' worth of
/// correction led here. One assumed the textbook `DT_RELA`/`DT_JMPREL` layout
/// and failed, because a current `/bin/ls` is linked `-z now` and has no
/// `DT_JMPREL`. The next asserted a hardcoded address list for the first RELR
/// group, written from memory rather than read from the binary; it failed too,
/// and the diagnostic it forced showed the cause: **the first entry of a real
/// table does not carry bit 63**, which is the opposite of what the
/// specification's wording suggests, and which desynchronised this function's
/// alternation.
///
/// What the tests do establish is narrow: the base is folded into every emitted
/// address, and an all-zero bitmap is not mistaken for an address. Whether a
/// whole table decodes correctly is unknown — on this host `DT_RELRSZ` runs past
/// the end of `.rela.dyn`, so the words after the used entries are not
/// necessarily RELR data at all.
///
/// The bit-63 branch below is therefore a *defensive* reading, not a verified
/// one: a word with bit 63 set cannot be an address in a 47-bit user space, so
/// treating it as a group restart is the safe interpretation. That is an
/// argument, not evidence. Use `cargo test -p tgs-elf -- --ignored --nocapture`
/// to dump a real table and confirm the rule before relying on this to relocate
/// a whole image.
pub fn relr_decode(entries: &[u64], base: u64) -> Vec<u64> {
    let mut out = Vec::new();
    let mut last = 0u64;
    let mut expect_address = true;
    for &e in entries {
        if e & RELR_BITMAP != 0 {
            // The first entry doubles as the address and as the run-length
            // marker. Later entries with the bit set are not something the
            // format produces; treating them as an address keeps the decoder
            // total rather than panicking on a hostile table.
            last = base.wrapping_add(e & !RELR_BITMAP);
            out.push(last);
            expect_address = false;
            continue;
        }
        if expect_address {
            last = base.wrapping_add(e);
            out.push(last);
            expect_address = false;
        } else {
            let mut bit = 0u32;
            let mut word = e;
            while word != 0 {
                if word & 1 != 0 {
                    out.push(last.wrapping_add(u64::from(bit + 1) * 8));
                }
                word >>= 1;
                bit += 1;
            }
            expect_address = true;
        }
    }
    out
}

/// Read a `DT_RELR` table of `size` bytes out of an image.
pub fn relr_table(data: &[u8], addr: u64, size: u64) -> Result<Vec<u64>> {
    if size % 8 != 0 {
        return Err(Error::BadOffset);
    }
    let start = addr as usize;
    let end = start
        .checked_add(size as usize)
        .ok_or(Error::OutOfBounds)?;
    if end > data.len() {
        return Err(Error::OutOfBounds);
    }
    Ok(data[start..end]
        .chunks_exact(8)
        .map(|c| u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]))
        .collect())
}

fn rd_u64(d: &[u8], o: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&d[o..o + 8]);
    u64::from_le_bytes(b)
}

