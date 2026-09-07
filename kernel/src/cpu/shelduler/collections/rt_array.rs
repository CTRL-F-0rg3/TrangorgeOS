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

        list.insert_before((*task).rt.run_list);
        self.nr_running += 1;
    }

    pub unsafe fn dequeue(&mut self, task: *mut TaskStruct) {
        let prio = (*task).rt.rt_priority as usize;

        ListHead::remove((*task).rt.run_list);

        if self.queue[prio].is_empty() {
            self.clear_bit(prio);
        }
        self.nr_running -= 1;
    }

    pub unsafe fn pick_next(&mut self) -> *mut TaskStruct {
        let prio = match self.highest_prio() {
            Some(p) => p,
            None => return ptr::null_mut(),
        };
        let list = &mut self.queue[prio];
        if list.is_empty() {
            return ptr::null_mut();
        }

        let next = list.next;
        list.next = (*next).next;
        if !list.next.is_null() {
            (*list.next).prev = list as *mut ListHead;
        }
        list.insert_before(next);

        Self::task_from_run_list(next)
    }

    #[inline(always)]
    unsafe fn task_from_run_list(node: *mut ListHead) -> *mut TaskStruct {
        (node as *mut u8).sub(core::mem::offset_of!(
            crate::cpu::scheduler::entities::task::RtFields,
            run_list
        ) + core::mem::offset_of!(TaskStruct, rt)) as *mut TaskStruct
    }
}
