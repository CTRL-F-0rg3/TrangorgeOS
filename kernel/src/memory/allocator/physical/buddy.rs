
use crate::allocator::traits::{Frame, FrameAllocator};

const NIL: u32 = u32::MAX;

pub struct BuddyFrameAllocator<'a> {
    next: &'a mut [u32],
    order_of: &'a mut [u8],
    free_heads: [u32; Self::MAX_ORDER + 1],
    total_frames: usize,
    free_frames: usize,
}

impl<'a> BuddyFrameAllocator<'a> {
    pub const MAX_ORDER: usize = 18; // 2^18 ramek = 1 GiB przy stronach 4 KiB
    const NIL_ORDER: u8 = u8::MAX;

    pub fn new(next: &'a mut [u32], order_of: &'a mut [u8], total_frames: usize) -> Self {
        assert!(next.len() >= total_frames && order_of.len() >= total_frames);
        for slot in next.iter_mut() { *slot = NIL; }
        for slot in order_of.iter_mut() { *slot = Self::NIL_ORDER; }
        Self {
            next,
            order_of,
            free_heads: [NIL; Self::MAX_ORDER + 1],
            total_frames,
            free_frames: 0,
        }
    }

    fn push_free(&mut self, order: usize, frame: usize) {
        self.next[frame] = self.free_heads[order];
        self.order_of[frame] = order as u8;
        self.free_heads[order] = frame as u32;
    }

    fn pop_free(&mut self, order: usize) -> Option<usize> {
        let head = self.free_heads[order];
        if head == NIL { return None; }
        self.free_heads[order] = self.next[head as usize];
        self.order_of[head as usize] = Self::NIL_ORDER;
        Some(head as usize)
    }

    fn remove_free(&mut self, order: usize, frame: usize) -> bool {
        let mut cur = self.free_heads[order];
        let mut prev: Option<usize> = None;
        while cur != NIL {
            if cur as usize == frame {
                let nxt = self.next[cur as usize];
                match prev {
                    Some(p) => self.next[p] = nxt,
                    None => self.free_heads[order] = nxt,
                }
                self.order_of[cur as usize] = Self::NIL_ORDER;
                return true;
            }
            prev = Some(cur as usize);
            cur = self.next[cur as usize];
        }
        false
    }

    pub fn add_free_range(&mut self, start_frame: usize, count: usize) {
        let mut frame = start_frame;
        let mut remaining = count;
        while remaining > 0 {
            let align_order = if frame == 0 {
                Self::MAX_ORDER
            } else {
                (frame.trailing_zeros() as usize).min(Self::MAX_ORDER)
            };
            let mut order = align_order;
            while (1usize << order) > remaining {
                order -= 1;
            }
            self.push_free(order, frame);
            self.free_frames += 1 << order;
            frame += 1 << order;
            remaining -= 1 << order;
        }
    }

    pub fn allocate_order(&mut self, order: usize) -> Option<usize> {
        if order > Self::MAX_ORDER { return None; }
        let mut current_order = order;
        while current_order <= Self::MAX_ORDER {
            if let Some(frame) = self.pop_free(current_order) {
                let mut split_order = current_order;
                while split_order > order {
                    split_order -= 1;
                    let buddy = frame + (1 << split_order);
                    self.push_free(split_order, buddy);
                }
                self.free_frames -= 1 << order;
                return Some(frame);
            }
            current_order += 1;
        }
        None
    }

    pub fn deallocate_order(&mut self, mut frame: usize, mut order: usize) {
        self.free_frames += 1 << order;
        while order < Self::MAX_ORDER {
            let buddy = frame ^ (1 << order);
            if buddy >= self.total_frames { break; }
            if self.order_of[buddy] as usize == order && self.remove_free(order, buddy) {
                frame = frame.min(buddy);
                order += 1;
            } else {
                break;
            }
        }
        self.push_free(order, frame);
    }
}

impl<'a> FrameAllocator for BuddyFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<Frame> {
        self.allocate_order(0).map(Frame)
    }
    fn deallocate_frame(&mut self, frame: Frame) {
        self.deallocate_order(frame.0, 0);
    }
    fn total_frames(&self) -> usize { self.total_frames }
    fn free_frames(&self) -> usize { self.free_frames }
}