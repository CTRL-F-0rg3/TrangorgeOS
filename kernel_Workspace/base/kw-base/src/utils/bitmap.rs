use crate::core::KResult;

pub struct Bitmap {
    data: *mut u64,
    bits: usize,
    owned: bool,
}

impl Bitmap {
    pub fn new_external(ptr: *mut u64, bits: usize) -> Self {
        Self { data: ptr, bits, owned: false }
    }

    pub fn words_needed(bits: usize) -> usize {
        (bits + 63) / 64
    }

    #[inline]
    fn word_idx(&self, bit: usize) -> usize {
        bit / 64
    }

    #[inline]
    fn bit_mask(&self, bit: usize) -> u64 {
        1u64 << (bit % 64)
    }

    pub fn set(&mut self, bit: usize) {
        if bit < self.bits {
            unsafe { *self.data.add(self.word_idx(bit)) |= self.bit_mask(bit); }
        }
    }

    pub fn clear(&mut self, bit: usize) {
        if bit < self.bits {
            unsafe { *self.data.add(self.word_idx(bit)) &= !self.bit_mask(bit); }
        }
    }

    pub fn test(&self, bit: usize) -> bool {
        if bit >= self.bits { return false; }
        unsafe { (*self.data.add(self.word_idx(bit)) & self.bit_mask(bit)) != 0 }
    }

    pub fn find_first_zero(&self) -> Option<usize> {
        for i in 0..Self::words_needed(self.bits) {
            let word = unsafe { *self.data.add(i) };
            if word != u64::MAX {
                let bit = (!word).trailing_zeros() as usize;
                let idx = i * 64 + bit;
                if idx < self.bits { return Some(idx); }
            }
        }
        None
    }

    pub fn find_first_set(&self) -> Option<usize> {
        for i in 0..Self::words_needed(self.bits) {
            let word = unsafe { *self.data.add(i) };
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                let idx = i * 64 + bit;
                if idx < self.bits { return Some(idx); }
            }
        }
        None
    }

    pub fn alloc_first_zero(&mut self) -> Option<usize> {
        let idx = self.find_first_zero()?;
        self.set(idx);
        Some(idx)
    }

    pub fn count_set(&self) -> usize {
        let mut count = 0;
        for i in 0..Self::words_needed(self.bits) {
            count += unsafe { (*self.data.add(i)).count_ones() as usize };
        }
        count
    }

    pub fn count_free(&self) -> usize {
        self.bits - self.count_set()
    }
}