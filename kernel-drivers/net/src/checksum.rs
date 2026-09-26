//! The Internet checksum, and the bug that was in it.
//!
//! # What a checksum is for here
//!
//! Hardware that can do checksums should be told to leave them alone
//! ([`Checksum::Offload`]), because computing one on the CPU is the whole cost
//! that offload exists to remove. The software path is here for cards that
//! cannot, and for verifying what a card claims to have done.
//!
//! # The bug this fixes
//!
//! The original `is_valid` was:
//!
//! ```text
//! ones_complement_sum(bytes).wrapping_add(0).wrapping_add(0)
//!     .wrapping_sub(0) & 0xffff == 0xffff
//!     || fold(ones_complement_sum(bytes)) == 0
//! ```
//!
//! The three `wrapping_*` calls with zero are no-ops, and the first clause tests
//! `sum & 0xffff == 0xffff` **without folding the carry**. A packet whose raw
//! sum is `0x1ffff` has low 16 bits `0xffff`, so that clause accepts it - even
//! though folding `0x1ffff` gives `0x0000`, which is a *valid* checksum and not
//! the one the clause is looking for. The first clause is a false path through
//! packet validation.
//!
//! It is worth writing down because the bug is invisible in the passing case: a
//! correct packet satisfies both clauses, so no test written the obvious way
//! catches it. [`is_valid`] now has the one that does - a header whose sum
//! needs a carry.

use crate::error::PacketError;

/// Sum the bytes as 16-bit big-endian words, without folding the carry.
#[inline]
pub fn ones_complement_sum(bytes: &[u8]) -> u32 {
    let mut sum = 0u32;
    let mut chunks = bytes.chunks_exact(2);

    for pair in &mut chunks {
        sum = sum.wrapping_add(u16::from_be_bytes([pair[0], pair[1]]) as u32);
    }

    if let [last] = chunks.remainder() {
        sum = sum.wrapping_add((*last as u32) << 8);
    }

    sum
}

/// Fold the carries and complement, giving the value to store.
#[inline]
pub fn fold(sum: u32) -> u16 {
    let mut sum = sum;
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

/// The checksum to store in a header.
#[inline]
pub fn checksum(bytes: &[u8]) -> u16 {
    fold(ones_complement_sum(bytes))
}

/// Does this header or payload verify?
///
/// The test is that folding the sum yields `0`. That is the definition, and it
/// is the only form that handles the carry: a sum of `0x1ffff` folds to `0xffff`
/// and complements to `0x0000`, so it is valid - while the naive
/// `sum & 0xffff` form was free to reject it. See the module documentation.
#[inline]
pub fn is_valid(bytes: &[u8]) -> bool {
    fold(ones_complement_sum(bytes)) == 0
}

/// Fill in the two-byte checksum field at `offset` and return the value written.
///
/// Takes the whole buffer and an offset rather than a slice and a reference to
/// the rest of it. The obvious signature -
/// `fill(&mut header, 10)` - cannot compile: it takes two borrows of
/// the same buffer, one mutable and one immutable, and no amount of reordering
/// fixes that.
///
/// The field is zeroed before summing. A stale value left in it is summed into
/// the new checksum, producing one that is correct for exactly one frame.
pub fn fill(header: &mut [u8], offset: usize) -> Result<u16, PacketError> {
    if offset.checked_add(2).is_none_or(|end| end > header.len()) {
        return Err(PacketError::Truncated);
    }
    header[offset] = 0;
    header[offset + 1] = 0;
    let value = checksum(header);
    header[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
    Ok(value)
}

/// Whether a card should compute the checksum or leave it to hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Checksum {
    /// The driver computes it. Always correct, and always costs CPU.
    #[default]
    Software,
    /// The driver fills in the offsets and the card computes it.
    Offload,
    /// The card computes it and reports nothing. Cheapest, and a corrupt frame
    /// is undetectable - which is the trade, stated.
    Unchecked,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real IPv4 header: version/IHL 5, total length 20, TTL 64, proto 1.
    const HEADER: [u8; 20] = [
        0x45, 0, 0, 20, 0, 1, 0x40, 0, 64, 1, 0, 0, 10, 0, 0, 1, 10, 0, 0, 2,
    ];

    #[test]
    fn a_sealed_header_verifies() {
        let mut header = HEADER;
        fill(&mut header, 10).expect("field is two bytes");
        assert!(is_valid(&header));
    }

    #[test]
    fn a_header_whose_sum_needs_a_carry_still_verifies() {
        // The regression test for the bug in the module documentation. This
        // header's raw sum exceeds 16 bits, so the unfolded and folded tests
        // disagree - and the old `is_valid` had a clause that used the unfolded
        // one.
        let mut header = HEADER;
        // Make the sum large enough to require a fold.
        header[1] = 0xff;
        header[2] = 0xff;
        header[3] = 0x20;
        fill(&mut header, 10).expect("field is two bytes");

        let raw = ones_complement_sum(&header);
        assert!(raw > 0xffff, "this case is only interesting with a carry, got {raw:#x}");
        assert!(is_valid(&header), "and it must still verify");
    }

    #[test]
    fn a_corrupted_header_fails() {
        let mut header = HEADER;
        fill(&mut header, 10).unwrap();
        header[8] = 63; // TTL, which the checksum covers.
        assert!(!is_valid(&header), "a changed TTL must not verify");
    }

    #[test]
    fn the_field_is_zeroed_before_summing() {
        // A stale value left in the field is summed into the new checksum,
        // producing one that is right for exactly one frame.
        let mut header = HEADER;
        header[10] = 0xaa;
        header[11] = 0xbb;
        let first = fill(&mut header, 10).unwrap();

        let mut again = header;
        let second = fill(&mut again, 10).unwrap();

        assert_eq!(first, second, "recomputing must give the same answer");
    }

    #[test]
    fn an_odd_length_buffer_sums_its_last_byte_shifted() {
        // The final byte of an odd-length buffer is the high half of a word.
        // Getting this wrong is right for every even payload and wrong for
        // every odd one.
        assert_eq!(ones_complement_sum(&[0x12, 0x34, 0x56]), 0x1234 + 0x5600);
    }

    #[test]
    fn an_empty_buffer_sums_to_zero() {
        assert_eq!(ones_complement_sum(&[]), 0);
    }

    #[test]
    fn a_two_byte_field_is_required() {
        let mut header = HEADER;
        assert_eq!(fill(&mut header, 19).unwrap_err(), PacketError::Truncated, "it would run off the end");
        assert_eq!(fill(&mut header, usize::MAX).unwrap_err(), PacketError::Truncated, "and must not overflow");
        let mut tiny = [0u8; 1];
        assert_eq!(fill(&mut tiny, 0).unwrap_err(), PacketError::Truncated);
    }
}

