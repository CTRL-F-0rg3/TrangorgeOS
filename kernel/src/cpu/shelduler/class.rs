//! Simplest round-robin scheduling class.
//!
//! Uses the intrusive `RunQueue` from `entities::runqueue`.

use crate::cpu::shelduler::entities::runqueue::RunQueue;
use crate::cpu::shelduler::entities::task::TaskStruct;

pub trait SchedClass {
    fn enqueue(&self, rq: &mut RunQueue, task: *mut TaskStruct);
    fn dequeue(&self, rq: &mut RunQueue, task: *mut TaskStruct);
    fn pick_next_task(&self, rq: &mut RunQueue) -> *mut TaskStruct;
    fn task_tick(&self, rq: &mut RunQueue, task: *mut TaskStruct);
}

pub struct BasicRrClass;

impl SchedClass for BasicRrClass {
    fn enqueue(&self, rq: &mut RunQueue, task: *mut TaskStruct) {
        unsafe { rq.enqueue(task); }
    }

    fn dequeue(&self, rq: &mut RunQueue, task: *mut TaskStruct) {
        unsafe { let _ = rq.remove(task); }
    }

    fn pick_next_task(&self, rq: &mut RunQueue) -> *mut TaskStruct {
        rq.peek()
    }

    fn task_tick(&self, rq: &mut RunQueue, task: *mut TaskStruct) {
        // Round-robin: move the current task to the tail when its quantum ends.
        unsafe {
            if !task.is_null() && rq.count > 1 {
                let _ = rq.remove(task);
                rq.enqueue(task);
            }
        }
    }
}

pub static DEFAULT_SCHED_CLASS: BasicRrClass = BasicRrClass;
