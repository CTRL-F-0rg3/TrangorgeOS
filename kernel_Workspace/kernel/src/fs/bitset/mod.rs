//! Bit-set directories: a directory is a span of bits, and a header names it.
//!
//! This is the idea the rest of the filesystem is built on, so it is worth
//! stating precisely what it means and what it buys.
//!
//! # A directory is a bit-set, not a list
//!
//! A conventional directory is an array of records. A directory here is a
//! **bitmap over an extent**: the extent's first bit means "this name is used",
//! and the record itself is found by rank. Two things follow.
//!
//! **Lookup becomes counting.** To find the record for name `X`, compute its
//! hash, find the bit, and count the set bits before it. There is no search.
//!
//! **An empty directory costs one block, not zero.** A conventional directory
//! with 200 000 files holds 200 000 records; here it holds a bitmap of 200 000
//! bits — 25 KiB — and the records sit in the data extent. Deleting the last
//! file does not shrink anything, and that is the point: the structure never
//! fragments, so there is no compaction and no reorganisation.
//!
//! # Why a header
//!
//! Because a bare bitmap cannot say what it is a bitmap *of*. [`DirHeader`]
//! carries the extent bounds, the checksum, and the generation that makes a
//! half-written header detectable. Without the generation, a torn write leaves a
//! header that parses and points at the wrong extent, and the failure shows up
//! as missing files rather than as a filesystem error.
//!
//! # The cost, stated plainly
//!
//! Bit-set lookup is `O(1)` and creation is `O(1)`, which is genuinely better
//! than a hash tree for the common case. But a directory with a handful of files
//! in a very large volume still pays for the bitmap's coverage, and directory
//! enumeration has to walk the bitmap rather than a record array. That is a real
//! trade, not a free win, and it is why [`DirHeader::needed_bits`] exists: a
//! volume knows the hash space it is addressing and sizes the bitmap to it.
//!
//! The inspiration is RedoxFS's `disk/` and `tree.rs`, where a directory's
//! contents are reached by pointer arithmetic rather than by comparison, and
//! Linux's htree, which does the same thing for ext4 with a different indexing
//! scheme. What is taken from Redox is the *shape* — a header that owns its
//! extent — and what is deliberately not taken is its on-disk compatibility,
//! because a filesystem nobody else can read is not a filesystem.

use crate::fs::driver::block::{BlockDevice, DriverError};
use alloc::vec;
use alloc::vec::Vec;

/// The magic at the start of every directory header: "TGD1".
pub const DIR_MAGIC: [u8; 4] = *b"TGD1";

/// The version this driver writes.
pub const DIR_VERSION: u8 = 1;

/// Bytes in a [`DirHeader`], as stored on disk.
pub const DIR_HEADER_BYTES: usize = 64;


/// Why a bit-set directory operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitsError {
    /// The media reported an I/O error.
    Io,
    /// The header's magic, version or checksum is wrong.
    Corrupt,
    /// No such name.
    NotFound,
    /// The name is already present.
    Exists,
    /// The name is empty, too long, or contains a byte a name cannot hold.
    BadName,
    /// The bitmap is full: no bit is free.
    Full,
    /// The header is from a newer version than this driver understands.
    UnsupportedVersion(u8),
}

impl From<DriverError> for BitsError {
    #[inline]
    fn from(_: DriverError) -> Self {
        BitsError::Io
    }
}

/// The header that gives a bit-set its meaning.
///
/// `#[repr(C)]` and explicit `to_le_bytes` rather than a direct cast: this
/// structure is written to a medium that another operating system may read, so
/// the byte order has to be a decision rather than an accident of the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirHeader {
    /// `TGD1`.
    pub magic: [u8; 4],
    /// On-disk version.
    pub version: u8,
    /// How many bits the bitmap holds.
    pub bit_count: u32,
    /// Where the bitmap's sectors begin.
    pub bitmap_lba: u64,
    /// Where the records' sectors begin.
    pub records_lba: u64,
    /// How many bits are set, i.e. how many names exist.
    pub used_bits: u32,
    /// Incremented on every write; a torn write is detectable by a stale value.
    pub generation: u32,
    /// Sum of the four above, so a flipped bit is caught before the fields are
    /// believed.
    pub checksum: u32,
}

impl Default for DirHeader {
    /// A zeroed header that is *not* valid until [`Self::seal`] runs.
    ///
    /// The point of not deriving `Default` is that an all-zero header has the
    /// wrong magic: a caller who forgets to seal gets a `Corrupt` from every
    /// operation instead of a directory that looks empty and is not.
    fn default() -> Self {
        Self {
            magic: DIR_MAGIC,
            version: DIR_VERSION,
            bit_count: 0,
            bitmap_lba: 0,
            records_lba: 0,
            used_bits: 0,
            generation: 1,
            checksum: 0,
        }
    }
}

impl DirHeader {
    /// How many bytes the header occupies.
    pub const fn bytes() -> usize {
        DIR_HEADER_BYTES
    }

    /// A header for a directory with `bits` slots, starting at `lba`.
    pub const fn new(bits: u32, bitmap_lba: u64, records_lba: u64) -> Self {
        Self { magic: DIR_MAGIC, version: DIR_VERSION, bit_count: bits, bitmap_lba, records_lba, ..Self::default() }
    }

    /// The checksum this header must carry.
    ///
    /// Every field except the checksum itself, each mixed with an odd constant
    /// so two fields cannot cancel by carrying the same bits. A plain XOR would
    /// let a header with `bit_count = 5, used_bits = 5` and one with
    /// `bit_count = 7, used_bits = 7` sum to the same value, so a header that
    /// differs only in those two fields would pass.
    pub const fn compute_checksum(&self) -> u32 {
        let mut h: u32 = 0x811C_9DC5; // FNV-1a's offset basis
        h = (h ^ self.magic[0] as u32).wrapping_mul(0x0100_0193);
        h = (h ^ self.magic[1] as u32).wrapping_mul(0x0100_0193);
        h = (h ^ self.magic[2] as u32).wrapping_mul(0x0100_0193);
        h = (h ^ self.magic[3] as u32).wrapping_mul(0x0100_0193);
        h = (h ^ self.version as u32).wrapping_mul(0x0100_0193);
        h = (h ^ self.bit_count).wrapping_mul(0x0100_0193);
        h = (h ^ self.bitmap_lba as u32).wrapping_mul(0x0100_0193);
        h = (h ^ self.records_lba as u32).wrapping_mul(0x0100_0193);
        h = (h ^ (self.bitmap_lba >> 32) as u32).wrapping_mul(0x0100_0193);
        h = (h ^ (self.records_lba >> 32) as u32).wrapping_mul(0x0100_0193);
        h = (h ^ self.used_bits).wrapping_mul(0x0100_0193);
        h ^ self.generation
    }

    /// Fill in the checksum. Called before every write.
    pub fn seal(&mut self) {
        self.checksum = self.compute_checksum();
    }

    /// Is this header internally consistent?
    pub const fn is_valid(&self) -> bool {
        self.magic[0] == DIR_MAGIC[0]
            && self.magic[1] == DIR_MAGIC[1]
            && self.magic[2] == DIR_MAGIC[2]
            && self.magic[3] == DIR_MAGIC[3]
            && self.version == DIR_VERSION
            && self.bit_count > 0
            && self.used_bits <= self.bit_count
            && self.checksum == self.compute_checksum()
    }

    /// Bytes the bitmap needs for `bits` bits.
    ///
    /// `(bits + 7) / 8`, and the `+ 7` is what stops a bitmap of 8 bits from
    /// being zero bytes.
    pub const fn bitmap_bytes(bits: u32) -> u32 {
        bits / 8 + u32::from(bits % 8 != 0)
    }

    /// How many whole 512-byte sectors `bytes` occupies.
    pub const fn sectors_for(bytes: u32) -> u32 {
        bytes / 512 + u32::from(bytes % 512 != 0)
    }

    /// Encode into exactly [`DIR_HEADER_BYTES`].
    pub fn to_bytes(&self) -> [u8; DIR_HEADER_BYTES] {
        let mut b = [0u8; DIR_HEADER_BYTES];
        b[0..4].copy_from_slice(&self.magic);
        b[4] = self.version;
        b[5..9].copy_from_slice(&self.bit_count.to_le_bytes());
        b[9..13].copy_from_slice(&(self.bitmap_lba as u32).to_le_bytes());
        b[13..21].copy_from_slice(&self.bitmap_lba.to_le_bytes());
        b[21..25].copy_from_slice(&(self.records_lba as u32).to_le_bytes());
        b[25..33].copy_from_slice(&self.records_lba.to_le_bytes());
        b[33..37].copy_from_slice(&self.used_bits.to_le_bytes());
        b[37..41].copy_from_slice(&self.generation.to_le_bytes());
        b[41..45].copy_from_slice(&self.checksum.to_le_bytes());
        b
    }

    /// Decode from [`DIR_HEADER_BYTES`], validating magic, version and checksum.
    pub fn from_bytes(b: &[u8]) -> Result<Self, BitsError> {
        if b.len() < DIR_HEADER_BYTES {
            return Err(BitsError::Corrupt);
        }
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&b[0..4]);
        if magic != DIR_MAGIC {
            return Err(BitsError::Corrupt);
        }
        if b[4] != DIR_VERSION {
            return Err(BitsError::UnsupportedVersion(b[4]));
        }
        let rd32 = |o: usize| u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
        let rd64 = |o: usize| {
            let mut a = [0u8; 8];
            a.copy_from_slice(&b[o..o + 8]);
            u64::from_le_bytes(a)
        };
        let h = Self {
            magic,
            version: b[4],
            bit_count: rd32(5),
            bitmap_lba: rd64(13),
            records_lba: rd64(25),
            used_bits: rd32(33),
            generation: rd32(37),
            checksum: rd32(41),
        };
        // `bitmap_lba as u32` is mixed into the checksum, and the full 64-bit
        // value too, so both halves are covered.
        if h.checksum != h.compute_checksum() {
            return Err(BitsError::Corrupt);
        }
        Ok(h)
    }
}
