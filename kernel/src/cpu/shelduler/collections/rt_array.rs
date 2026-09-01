use crate::cpu::scheduler::entities::task::{TaskStruct, ListHead, MAX_RT_PRIO};
use core::ptr;

pub struct RtArray {
    pub queue: [ListHead; MAX_RT_PRIO as usize],
    pub bitmap: [u64; 2], 
    pub nr_running: usize,
}

impl RtArray {
    pub const fn new() -> Self {
        Self {
            queue: [ListHead::new(); MAX_RT_PRIO as usize],
            bitmap: [0, 0],
            nr_running: 0,
        }
    }

    #[inline(always)]
    pub fn set_bit(&mut self, prio: usize) {
        let word = prio / 64;
        let bit = prio % 64;
        self.bitmap[word] |= 1u64 << bit;
    }

    #[inline(always)]
    pub fn clear_bit(&mut self, prio: usize) {
        let word = prio / 64;
        let bit = prio % 64;
        self.bitmap[word] &= !(1u64 << bit);
    }

    #[inline(always)]
    pub fn highest_prio(&self) -> Option<usize> {
        if self.bitmap[0] != 0 {
            Some(self.bitmap[0].trailing_zeros() as usize)
        } else if self.bitmap[1] != 0 {
            Some(64 + self.bitmap[1].trailing_zeros() as usize)
        } else {
            None
        }
    }

    pub unsafe fn enqueue(&mut self, task: *mut TaskStruct) {
        let prio = (*task).rt.rt_priority as usize;
        let list = &mut self.queue[prio];
        
        if list.is_empty() {
            self.set_bit(prio);
        }
        
        list.insert_before(task as *mut ListHead); 
        (*task).rt.run_list = ptr::null_mut(); 
    }

    pub unsafe fn dequeue(&mut self, task: *mut TaskStruct) {
        let prio = (*task).rt.rt_priority as usize;
        let list = &mut self.queue[prio];
        
        (task as *mut ListHead).remove();
        
        if list.is_empty() {
            self.clear_bit(prio);
        }
        self.nr_running -= 1;
    }

    pub unsafe fn pick_next(&mut self) -> *mut TaskStruct {
        if let Some(prio) = self.highest_prio() {
            let list = &mut self.queue[prio];
            if !list.is_empty() {
                let next = list.next;
                (next).remove();
                list.insert_before(next);
                return next as *mut TaskStruct;
            }
        }
        ptr::null_mut()
    }
}