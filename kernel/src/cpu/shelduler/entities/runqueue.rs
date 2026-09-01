use crate::cpu::shelduler::entities::task::{TaskState, TaskStruct};\
use core::ptr;

#[repr(C)]
pub struct RunQueue {
    pub head: *mut TaskStruct,
    pub tail: *mut TaskStruct,
    pub count: usize,
}

impl RunQueue {
    pub const fn new() -> Self {
        Self {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Peek at the head of the queue without removing it.
    pub fn peek(&self) -> *mut TaskStruct {
        self.head
    }

    /// Remove the first occurrence of `task` (anywhere in the queue).
    /// Returns `true` if a node was removed
    pub unsafe fn remove(&mut self, task: *mut TaskStruct) -> bool {
        if self.is_empty() {
            return false;
        }
        let mut cur = self.head;
        while !cur.is_null() {
            if cur == task {
                let prev = (*cur).sched.rq_prev;
                let next = (*cur).sched.rq_next;
                if prev.is_null() {
                    self.head = next;
                } else {
                    (*prev).sched.rq_next = next;
                }
                if next.is_null() {
                    self.tail = prev;
                } else {
                    (*next).sched.rq_prev = prev;
                }
                (*cur).sched.rq_next = ptr::null_mut();
                (*cur).sched.rq_prev = ptr::null_mut();
                self.count -= 1;
                return true;
            }
            cur = (*cur).sched.rq_next;
        }
        false
    }

    pub fn enqueue(&mut self, task: *mut TaskStruct) {
        unsafe {
            (*task).sched.rq_next = ptr::null_mut();
            (*task).sched.rq_prev = self.tail;

            if !self.tail.is_null() {
                (*self.tail).sched.rq_next = task;
            } else {
                self.head = task;
            }

            self.tail = task;
            self.count += 1;
        }
    }

    pub fn dequeue(&mut self) -> Option<*mut TaskStruct> {
        if self.is_empty() {
            return None;
        }

        unsafe {
            let task = self.head;

            self.head = (*task).sched.rq_next;

            if !self.head.is_null() {
                (*self.head).sched.rq_prev = ptr::null_mut();
            } else {
                self.tail = ptr::null_mut();
            }

            (*task).sched.rq_next = ptr::null_mut();
            (*task).sched.rq_prev = ptr::null_mut();

            self.count -= 1;

            Some(task)
        }
    }
}