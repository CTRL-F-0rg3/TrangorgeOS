pub const PAGE_SIZE: usize = 4096;

crate::test_module!({
    let mut storage = [0u64; 4];
    let mut allocator = BitmapFrameAllocator::new(&mut storage, 200);

    let first = match allocator.allocate_frame() {
        Some(idx) => idx,
        None => return Err("allocate_frame returned None on a fresh allocator"),
    };
    let second = match allocator.allocate_frame() {
        Some(idx) => idx,
        None => return Err("allocate_frame returned None for the second frame"),
    };
    if first == second {
        return Err("allocate_frame returned the same frame index twice");
    }
    if allocator.used_frames() != 2 {
        return Err("used_frames did not track two allocations correctly");
    }

    allocator.deallocate_frame(first);
    if allocator.used_frames() != 1 {
        return Err("used_frames did not decrease after deallocate_frame");
    }

    let reused = match allocator.allocate_frame() {
        Some(idx) => idx,
        None => return Err("allocate_frame returned None after freeing a frame"),
    };
    if reused != first {
        return Err("freed frame was not reused by the next allocation");
    }

    Ok("bitmap allocator alloc/dealloc/reuse verified")
});

pub struct BitmapFrameAllocator<'a> {
    bitmap: &'a mut [u64],
    total_frames: usize,
    used_frames: usize,
    last_word_idx: usize,
}

impl<'a> BitmapFrameAllocator<'a> {
    pub fn new(bitmap_slice: &'a mut [u64], total_frames: usize) -> Self {
        let required_words = total_frames.div_ceil(64);
        assert!(
            bitmap_slice.len() >= required_words,
            "bitmap slice too small for the requested frame count"
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
            *word = !0u64;
        }
        self.used_frames = self.total_frames;
        self.last_word_idx = 0;
    }

    pub fn allocate_frame(&mut self) -> Option<usize> {
        let num_words = self.total_frames.div_ceil(64);

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
        assert!(frame_idx < self.total_frames, "frame index out of range");

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
            if frame_idx >= self.total_frames {
                break;
            }

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
            if frame_idx >= self.total_frames {
                break;
            }

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

    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    pub fn used_frames(&self) -> usize {
        self.used_frames
    }

    pub fn free_frames(&self) -> usize {
        self.total_frames - self.used_frames
    }
}
