pub const PAGE_SIZE: usize = 4096;

pub struct BitmapFrameAllocator<'a> {
    bitmap: &'a mut [u64],
    total_frames: usize,
    used_frames: usize,
    last_word_idx: usize,
}

impl<'a> BitmapFrameAllocator<'a> {
    pub fn new(bitmap_slice: &'a mut [u64], total_frames: usize) -> Self {
        let required_words = (total_frames + 63) / 64;
        assert!(
            bitmap_slice.len() >= required_words,
            "Przekazana bitmapa jest za mała dla podanej liczby ramek!"
        );

        Self {
            bitmap: bitmap_slice,
            total_frames,
            used_frames: 0,
            last_word_idx: 0,
        }
    }

    pub fn mark_all_used(&mut self) {
        for word in self.bitmap.iter_mut() {
            *word = !0u64; // Same jedynki
        }
        self.used_frames = self.total_frames;
        self.last_word_idx = 0;
    }

    pub fn allocate_frame(&mut self) -> Option<usize> {
        let num_words = (self.total_frames + 63) / 64;

        for i in 0..num_words {
            let word_idx = (self.last_word_idx + i) % num_words;
            let word = self.bitmap[word_idx];

            if word != !0u64 {
                let bit_idx = (!word).trailing_zeros() as usize;
                let frame_idx = word_idx * 64 + bit_idx;

                if frame_idx >= self.total_frames {
                    return None; 
                }

                self.bitmap[word_idx] |= 1u64 << bit_idx;
                self.used_frames += 1;
                self.last_word_idx = word_idx;

                return Some(frame_idx);
            }
        }

        None 
    }

    pub fn deallocate_frame(&mut self, frame_idx: usize) {
        assert!(frame_idx < self.total_frames, "Indeks ramki poza zakresem!");

        let word_idx = frame_idx / 64;
        let bit_idx = frame_idx % 64;

        if (self.bitmap[word_idx] & (1u64 << bit_idx)) != 0 {
            self.bitmap[word_idx] &= !(1u64 << bit_idx);
            self.used_frames -= 1;

            if word_idx < self.last_word_idx {
                self.last_word_idx = word_idx;
            }
        }
    }

    pub fn mark_range_free(&mut self, start_frame: usize, count: usize) {
        for frame_idx in start_frame..start_frame + count {
            if frame_idx >= self.total_frames { break; }

            let word_idx = frame_idx / 64;
            let bit_idx = frame_idx % 64;

            if (self.bitmap[word_idx] & (1u64 << bit_idx)) != 0 {
                self.bitmap[word_idx] &= !(1u64 << bit_idx);
                self.used_frames -= 1;
            }
        }
    }

    pub fn mark_range_used(&mut self, start_frame: usize, count: usize) {
        for frame_idx in start_frame..start_frame + count {
            if frame_idx >= self.total_frames { break; }

            let word_idx = frame_idx / 64;
            let bit_idx = frame_idx % 64;

            if (self.bitmap[word_idx] & (1u64 << bit_idx)) == 0 {
                self.bitmap[word_idx] |= 1u64 << bit_idx;
                self.used_frames += 1;
            }
        }
    }

    


    pub fn frame_to_addr(frame_idx: usize) -> u64 {
        (frame_idx * PAGE_SIZE) as u64
    }


    pub fn addr_to_frame(addr: u64) -> usize {
        (addr as usize) / PAGE_SIZE
    }

    pub fn total_frames(&self) -> usize { self.total_frames }
    pub fn used_frames(&self) -> usize { self.used_frames }
    pub fn free_frames(&self) -> usize { self.total_frames - self.used_frames }
}