#![allow(dead_code)]

use core::sync::atomic::{AtomicU64, Ordering};

pub const BITS_PER_WORD: usize = 64;

pub const fn words_for_bits(nbits: usize) -> usize {
    (nbits + BITS_PER_WORD - 1) / BITS_PER_WORD
}

#[inline]
const fn word_of(bit: usize) -> usize {
    bit / BITS_PER_WORD
}

#[inline]
const fn mask_of(bit: usize) -> u64 {
    1u64 << (bit % BITS_PER_WORD)
}

// ------------------------------------------------------------------
// Bitmap<WORDS> — mapa bitowa o stałym rozmiarze, bez atomiki
// ------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bitmap<const WORDS: usize> {
    words: [u64; WORDS],
}

impl<const WORDS: usize> Bitmap<WORDS> {
    pub const CAPACITY: usize = WORDS * BITS_PER_WORD;

    pub const fn new() -> Self {
        Self { words: [0; WORDS] }
    }

    pub const fn filled() -> Self {
        Self { words: [u64::MAX; WORDS] }
    }

    #[inline]
    pub fn set(&mut self, bit: usize) {
        if bit < Self::CAPACITY {
            self.words[word_of(bit)] |= mask_of(bit);
        }
    }

    #[inline]
    pub fn clear(&mut self, bit: usize) {
        if bit < Self::CAPACITY {
            self.words[word_of(bit)] &= !mask_of(bit);
        }
    }

    #[inline]
    pub fn toggle(&mut self, bit: usize) {
        if bit < Self::CAPACITY {
            self.words[word_of(bit)] ^= mask_of(bit);
        }
    }

    #[inline]
    pub fn test(&self, bit: usize) -> bool {
        bit < Self::CAPACITY && self.words[word_of(bit)] & mask_of(bit) != 0
    }

    pub fn set_range(&mut self, start: usize, end: usize) {
        let end = end.min(Self::CAPACITY);
        let mut bit = start;
        while bit < end {
            self.set(bit);
            bit += 1;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|w| *w == 0)
    }

    pub fn is_full(&self) -> bool {
        self.words.iter().all(|w| *w == u64::MAX)
    }

    pub fn weight(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }

    pub fn find_first_set(&self) -> Option<usize> {
        for (idx, word) in self.words.iter().enumerate() {
            if *word != 0 {
                return Some(idx * BITS_PER_WORD + word.trailing_zeros() as usize);
            }
        }
        None
    }

    pub fn find_first_zero(&self) -> Option<usize> {
        for (idx, word) in self.words.iter().enumerate() {
            if *word != u64::MAX {
                let bit = idx * BITS_PER_WORD + (!word).trailing_zeros() as usize;
                if bit < Self::CAPACITY {
                    return Some(bit);
                }
                return None;
            }
        }
        None
    }

    pub fn find_next_set(&self, after: usize) -> Option<usize> {
        let mut bit = after + 1;
        while bit < Self::CAPACITY {
            let idx = word_of(bit);
            let shift = bit % BITS_PER_WORD;
            let masked = self.words[idx] & (u64::MAX << shift);
            if masked != 0 {
                let found = idx * BITS_PER_WORD + masked.trailing_zeros() as usize;
                return if found < Self::CAPACITY { Some(found) } else { None };
            }
            bit = (idx + 1) * BITS_PER_WORD;
        }
        None
    }

    pub fn and(&self, other: &Self) -> Self {
        let mut out = Self::new();
        for i in 0..WORDS {
            out.words[i] = self.words[i] & other.words[i];
        }
        out
    }

    pub fn or(&self, other: &Self) -> Self {
        let mut out = Self::new();
        for i in 0..WORDS {
            out.words[i] = self.words[i] | other.words[i];
        }
        out
    }

    pub fn xor(&self, other: &Self) -> Self {
        let mut out = Self::new();
        for i in 0..WORDS {
            out.words[i] = self.words[i] ^ other.words[i];
        }
        out
    }

    pub fn andnot(&self, other: &Self) -> Self {
        let mut out = Self::new();
        for i in 0..WORDS {
            out.words[i] = self.words[i] & !other.words[i];
        }
        out
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.words.iter().zip(other.words.iter()).any(|(a, b)| a & b != 0)
    }

    pub fn iter(&self) -> BitmapIter<'_> {
        BitmapIter { words: &self.words, next_bit: 0, capacity: Self::CAPACITY }
    }
}

impl<const WORDS: usize> Default for Bitmap<WORDS> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BitmapIter<'a> {
    words: &'a [u64],
    next_bit: usize,
    capacity: usize,
}

impl<'a> Iterator for BitmapIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        while self.next_bit < self.capacity {
            let bit = self.next_bit;
            self.next_bit += 1;
            if self.words[word_of(bit)] & mask_of(bit) != 0 {
                return Some(bit);
            }
        }
        None
    }
}

// ------------------------------------------------------------------
// BitmapSlice — widok nad `&mut [u64]` o rozmiarze nieznanym w
// czasie kompilacji (np. maski o rozmiarze zależnym od liczby CPU
// wykrytej w ACPI/MADT).
// ------------------------------------------------------------------

pub struct BitmapSlice<'a> {
    words: &'a mut [u64],
    nbits: usize,
}

impl<'a> BitmapSlice<'a> {
    pub fn new(words: &'a mut [u64], nbits: usize) -> Self {
        debug_assert!(words.len() >= words_for_bits(nbits));
        Self { words, nbits }
    }

    pub fn capacity(&self) -> usize {
        self.nbits
    }

    pub fn set(&mut self, bit: usize) {
        if bit < self.nbits {
            self.words[word_of(bit)] |= mask_of(bit);
        }
    }

    pub fn clear(&mut self, bit: usize) {
        if bit < self.nbits {
            self.words[word_of(bit)] &= !mask_of(bit);
        }
    }

    pub fn test(&self, bit: usize) -> bool {
        bit < self.nbits && self.words[word_of(bit)] & mask_of(bit) != 0
    }

    pub fn weight(&self) -> u32 {
        self.words.iter().map(|w| w.count_ones()).sum()
    }

    pub fn find_first_zero(&self) -> Option<usize> {
        for (idx, word) in self.words.iter().enumerate() {
            if *word != u64::MAX {
                let bit = idx * BITS_PER_WORD + (!word).trailing_zeros() as usize;
                if bit < self.nbits {
                    return Some(bit);
                }
                return None;
            }
        }
        None
    }

    pub fn clear_all(&mut self) {
        for w in self.words.iter_mut() {
            *w = 0;
        }
    }
}

// ------------------------------------------------------------------
// AtomicBitmap<WORDS> — bezpieczna współbieżnie mapa bitowa, w tym
// bez-blokadowy przydział pierwszego wolnego bitu (test-and-set przez
// CAS) — używana m.in. przez alokator PID w `core.rs`.
// ------------------------------------------------------------------

const ZERO_WORD: AtomicU64 = AtomicU64::new(0);

pub struct AtomicBitmap<const WORDS: usize> {
    words: [AtomicU64; WORDS],
}

impl<const WORDS: usize> AtomicBitmap<WORDS> {
    pub const CAPACITY: usize = WORDS * BITS_PER_WORD;

    pub const fn new() -> Self {
        Self { words: [ZERO_WORD; WORDS] }
    }

    #[inline]
    pub fn test(&self, bit: usize) -> bool {
        bit < Self::CAPACITY && self.words[word_of(bit)].load(Ordering::Acquire) & mask_of(bit) != 0
    }

    /// Ustawia bit atomowo, zwraca POPRZEDNI stan (true = już był ustawiony).
    pub fn test_and_set(&self, bit: usize) -> bool {
        if bit >= Self::CAPACITY {
            return true;
        }
        let prev = self.words[word_of(bit)].fetch_or(mask_of(bit), Ordering::AcqRel);
        prev & mask_of(bit) != 0
    }

    /// Czyści bit atomowo, zwraca POPRZEDNI stan (true = był ustawiony).
    pub fn test_and_clear(&self, bit: usize) -> bool {
        if bit >= Self::CAPACITY {
            return false;
        }
        let prev = self.words[word_of(bit)].fetch_and(!mask_of(bit), Ordering::AcqRel);
        prev & mask_of(bit) != 0
    }

    pub fn weight(&self) -> u32 {
        self.words.iter().map(|w| w.load(Ordering::Relaxed).count_ones()).sum()
    }

    pub fn is_full(&self) -> bool {
        self.words.iter().all(|w| w.load(Ordering::Relaxed) == u64::MAX)
    }

    /// Bez blokad: skanuje słowa w poszukiwaniu pierwszego wyzerowanego
    /// bitu i próbuje go zająć przez `compare_exchange_weak`. Przegrany
    /// wyścig oznacza, że ktoś inny właśnie zajął ten sam bit (albo inny
    /// w tym samym słowie) — ponawiamy próbę na TYM SAMYM słowie zamiast
    /// przechodzić dalej, żeby nie pominąć bitów zwolnionych w
    /// międzyczasie.
    pub fn find_first_zero_and_set(&self) -> Option<usize> {
        for idx in 0..WORDS {
            loop {
                let cur = self.words[idx].load(Ordering::Acquire);
                if cur == u64::MAX {
                    break;
                }
                let bit_in_word = (!cur).trailing_zeros() as usize;
                let global_bit = idx * BITS_PER_WORD + bit_in_word;
                if global_bit >= Self::CAPACITY {
                    return None;
                }
                let attempt = cur | (1u64 << bit_in_word);
                match self.words[idx].compare_exchange_weak(
                    cur,
                    attempt,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return Some(global_bit),
                    Err(_) => continue,
                }
            }
        }
        None
    }
}

impl<const WORDS: usize> Default for AtomicBitmap<WORDS> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_for_bits_rounds_up() {
        assert_eq!(words_for_bits(1), 1);
        assert_eq!(words_for_bits(64), 1);
        assert_eq!(words_for_bits(65), 2);
        assert_eq!(words_for_bits(128), 2);
    }

    #[test]
    fn set_clear_test_roundtrip() {
        let mut b: Bitmap<2> = Bitmap::new();
        assert!(!b.test(70));
        b.set(70);
        assert!(b.test(70));
        assert!(!b.test(69));
        b.clear(70);
        assert!(!b.test(70));
    }

    #[test]
    fn out_of_range_access_is_a_safe_noop() {
        let mut b: Bitmap<1> = Bitmap::new();
        b.set(1000);
        assert!(!b.test(1000));
        assert!(b.is_empty());
    }

    #[test]
    fn weight_counts_set_bits_across_words() {
        let mut b: Bitmap<2> = Bitmap::new();
        b.set(0);
        b.set(63);
        b.set(64);
        b.set(127);
        assert_eq!(b.weight(), 4);
    }

    #[test]
    fn find_first_set_crosses_word_boundary() {
        let mut b: Bitmap<3> = Bitmap::new();
        b.set(130);
        assert_eq!(b.find_first_set(), Some(130));
    }

    #[test]
    fn find_first_zero_skips_full_words() {
        let mut b: Bitmap<2> = Bitmap::filled();
        b.clear(70);
        assert_eq!(b.find_first_zero(), Some(70));
    }

    #[test]
    fn find_first_zero_none_when_full() {
        let b: Bitmap<2> = Bitmap::filled();
        assert_eq!(b.find_first_zero(), None);
        assert!(b.is_full());
    }

    #[test]
    fn find_next_set_after_given_bit() {
        let mut b: Bitmap<2> = Bitmap::new();
        b.set(5);
        b.set(80);
        assert_eq!(b.find_next_set(5), Some(80));
        assert_eq!(b.find_next_set(80), None);
    }

    #[test]
    fn set_range_sets_contiguous_span() {
        let mut b: Bitmap<2> = Bitmap::new();
        b.set_range(60, 70);
        for i in 60..70 {
            assert!(b.test(i), "bit {i} powinien być ustawiony");
        }
        assert!(!b.test(59));
        assert!(!b.test(70));
        assert_eq!(b.weight(), 10);
    }

    #[test]
    fn boolean_ops_behave_as_expected() {
        let mut a: Bitmap<1> = Bitmap::new();
        let mut b: Bitmap<1> = Bitmap::new();
        a.set(1);
        a.set(2);
        b.set(2);
        b.set(3);
        assert_eq!(a.and(&b).weight(), 1);
        assert!(a.and(&b).test(2));
        assert_eq!(a.or(&b).weight(), 3);
        assert_eq!(a.xor(&b).weight(), 2);
        assert_eq!(a.andnot(&b).weight(), 1);
        assert!(a.andnot(&b).test(1));
        assert!(a.intersects(&b));
    }

    #[test]
    fn iter_yields_set_bits_in_order() {
        let mut b: Bitmap<2> = Bitmap::new();
        b.set(3);
        b.set(64);
        b.set(100);
        let collected: [usize; 3] = {
            let mut it = b.iter();
            [it.next().unwrap(), it.next().unwrap(), it.next().unwrap()]
        };
        assert_eq!(collected, [3, 64, 100]);
        assert!(b.iter().nth(3).is_none());
    }

    #[test]
    fn slice_bitmap_respects_logical_bit_count_not_word_count() {
        let mut storage = [0u64; 2];
        let mut view = BitmapSlice::new(&mut storage, 70);
        assert_eq!(view.capacity(), 70);
        view.set(69);
        assert!(view.test(69));
        view.set(100); // poza `nbits`, mimo że mieści się w drugim słowie
        assert!(!view.test(100));
        assert_eq!(view.find_first_zero(), Some(0));
    }

    #[test]
    fn slice_bitmap_clear_all_resets_backing_storage() {
        let mut storage = [u64::MAX; 1];
        let mut view = BitmapSlice::new(&mut storage, 64);
        assert_eq!(view.weight(), 64);
        view.clear_all();
        assert_eq!(view.weight(), 0);
    }

    #[test]
    fn atomic_bitmap_test_and_set_reports_previous_state() {
        let bm: AtomicBitmap<1> = AtomicBitmap::new();
        assert!(!bm.test_and_set(5));
        assert!(bm.test(5));
        assert!(bm.test_and_set(5));
    }

    #[test]
    fn atomic_bitmap_test_and_clear_reports_previous_state() {
        let bm: AtomicBitmap<1> = AtomicBitmap::new();
        bm.test_and_set(9);
        assert!(bm.test_and_clear(9));
        assert!(!bm.test(9));
        assert!(!bm.test_and_clear(9));
    }

    #[test]
    fn atomic_bitmap_find_first_zero_and_set_claims_sequentially() {
        let bm: AtomicBitmap<1> = AtomicBitmap::new();
        assert_eq!(bm.find_first_zero_and_set(), Some(0));
        assert_eq!(bm.find_first_zero_and_set(), Some(1));
        bm.test_and_clear(0);
        assert_eq!(bm.find_first_zero_and_set(), Some(0));
    }

    #[test]
    fn atomic_bitmap_find_first_zero_and_set_returns_none_when_full() {
        let bm: AtomicBitmap<1> = AtomicBitmap::new();
        for _ in 0..64 {
            assert!(bm.find_first_zero_and_set().is_some());
        }
        assert!(bm.is_full());
        assert_eq!(bm.find_first_zero_and_set(), None);
    }

    #[test]
    fn atomic_bitmap_weight_matches_manual_count() {
        let bm: AtomicBitmap<2> = AtomicBitmap::new();
        bm.test_and_set(0);
        bm.test_and_set(64);
        bm.test_and_set(127);
        assert_eq!(bm.weight(), 3);
    }

    #[test]
    fn atomic_bitmap_out_of_range_set_is_harmless() {
        let bm: AtomicBitmap<1> = AtomicBitmap::new();
        assert!(bm.test_and_set(1000));
        assert!(!bm.test_and_clear(1000));
    }
}