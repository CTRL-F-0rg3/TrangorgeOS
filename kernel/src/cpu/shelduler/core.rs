use crate::cpu::shelduler::entities::runqueue::RunQueue;
use crate::cpu::shelduler::class::{SchedClass, DEFAULT_SCHED_CLASS};
use crate::cpu::shelduler::entities::task::TaskStruct;
use core::ptr;

extern "C" {
    fn switch_to(prev: *mut TaskStruct, next: *mut TaskStruct);
}

static mut GLOBAL_RQ: RunQueue = RunQueue::new();
static mut CURRENT_TASK: *mut TaskStruct = ptr::null_mut();

pub unsafe fn set_current_task(task: *mut TaskStruct) {
    CURRENT_TASK = task;
}

pub unsafe fn get_current_task() -> *mut TaskStruct {
    CURRENT_TASK
}

pub unsafe fn get_rq() -> &'static mut RunQueue {
    &mut GLOBAL_RQ
}

pub unsafe fn schedule() {
    let prev = get_current_task();
    let rq = get_rq();
    
    if rq.count == 0 {
        return; 
    }

    let next = DEFAULT_SCHED_CLASS.pick_next_task(rq);

    if prev != next && !next.is_null() {
        
        set_current_task(next);
        switch_to(prev, next);
    }
}

pub unsafe fn scheduler_tick() {
    let curr = get_current_task();
    if !curr.is_null() {
        DEFAULT_SCHED_CLASS.task_tick(get_rq(), curr);

    }
}

