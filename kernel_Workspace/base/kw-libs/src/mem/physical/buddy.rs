// kw-libs/src/mem/physical/buddy.rs

use kstd_base::{PhysAddr, Size};
use kw_base::utils::Bitmap;
use kw_base::core::{KResult, KernelError};

const MAX_ORDER: usize = 11; // Up to 2^10 frames (4MB)

struct FreeList {
    head: u64, // Physical address of the first free block, 0 if empty
}

impl FreeList {
    const fn new() -> Self { Self { head: 0 } }
}

pub struct BuddyAllocator {
    base_phys: PhysAddr,
    total_frames: usize,
    free_lists: [FreeList; MAX_ORDER],
    // Bitmap to track if a block is split or free
    // For simplicity, we use a basic tracking mechanism here.
    // In a full implementation, this would be a per-order bitmap.
    split_bitmap: *mut u64, 
}

unsafe impl Send for BuddyAllocator {}
unsafe impl Sync for BuddyAllocator {}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            base_phys: PhysAddr(0),
            total_frames: 0,
            free_lists: [FreeList::new(); MAX_ORDER],
            split_bitmap: core::ptr::null_mut(),
        }
    }

    pub fn init(&mut self, base: PhysAddr, frames: usize, bitmap_ptr: *mut u64) {
        self.base_phys = base;
        self.total_frames = frames;
        self.split_bitmap = bitmap_ptr;
        
        // Clear bitmap
        let words = kw_base::utils::Bitmap::words_needed(frames);
        unsafe {
            core::ptr::write_bytes(bitmap_ptr, 0, words * 8);
        }

        // Add all frames to the highest possible order blocks
        let mut current_frame = 0;
        while current_frame < frames {
            let remaining = frames - current_frame;
            let mut order = MAX_ORDER - 1;
            
            // Find the largest order that fits
            while order > 0 && (1 << order) > remaining {
                order -= 1;
            }
            
            // Align to order boundary
            let align_mask = (1 << order) - 1;
            if (current_frame & align_mask) != 0 {
                current_frame = (current_frame + align_mask) & !align_mask;
                continue;
            }

            if (1 << order) <= remaining {
                self.free_block(current_frame, order);
                current_frame += 1 << order;
            } else {
                current_frame += 1;
            }
        }
    }

    fn frame_to_phys(&self, frame: usize) -> PhysAddr {
        PhysAddr(self.base_phys.0 + (frame as u64 * 4096))
    }

    fn phys_to_frame(&self, phys: PhysAddr) -> usize {
        ((phys.0 - self.base_phys.0) / 4096) as usize
    }

    fn free_block(&mut self, frame: usize, order: usize) {
        // Mark as not split in bitmap (simplified logic)
        let mut bitmap = Bitmap::new_external(self.split_bitmap, self.total_frames);
        bitmap.clear(frame);

        // Push to free list
        let phys = self.frame_to_phys(frame);
        unsafe {
            let ptr = phys.0 as *mut u64;
            *ptr = self.free_lists[order].head;
        }
        self.free_lists[order].head = phys.0;
    }

    pub fn alloc(&mut self, order: usize) -> KResult<PhysAddr> {
        if order >= MAX_ORDER { return Err(KernelError::InvalidArg); }

        // Find the smallest available order >= requested
        let mut current_order = order;
        while current_order < MAX_ORDER && self.free_lists[current_order].head == 0 {
            current_order += 1;
        }

        if current_order == MAX_ORDER {
            return Err(KernelError::NoMemory);
        }

        // Pop from free list
        let phys = PhysAddr(self.free_lists[current_order].head);
        unsafe {
            self.free_lists[current_order].head = *(phys.0 as *mut u64);
        }

        // Split blocks down to requested order
        while current_order > order {
            current_order -= 1;
            let buddy_frame = self.phys_to_frame(phys) + (1 << current_order);
            self.free_block(buddy_frame, current_order);
        }

        // Mark as allocated/split
        let mut bitmap = Bitmap::new_external(self.split_bitmap, self.total_frames);
        bitmap.set(self.phys_to_frame(phys));

        Ok(phys)
    }

    pub fn free(&mut self, phys: PhysAddr, order: usize) {
        let mut frame = self.phys_to_frame(phys);
        let mut current_order = order;

        let mut bitmap = Bitmap::new_external(self.split_bitmap, self.total_frames);

        // Merge with buddy if possible
        while current_order < MAX_ORDER - 1 {
            let buddy_frame = frame ^ (1 << current_order);
            if buddy_frame >= self.total_frames { break; }
            
            // Check if buddy is free and of the same order
            if bitmap.test(buddy_frame) { break; } // Buddy is allocated
            
            // TODO: In a full implementation, we would verify the buddy is in the free list
            // For now, we assume if it's not marked split, it's free.
            // This is a simplification.
            break; 
        }

        self.free_block(frame, current_order);
    }
}