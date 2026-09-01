use crate::cpu::shelduler::entities::runqueue::RunQueue;
use crate::cpu::shelduler::class::{SchedClass, DEFAULT_SCHED_CLASS};
use crate::cpu::shelduler::entities::task::TaskStruct;
use core::ptr;

// Zewntrzna funkcja w ASM. Sygnatura musi by idealnie dopasowana.
extern "C" {
    fn switch_to(prev: *mut TaskStruct, next: *mut TaskStruct);
}

// TODO: Zastp to swoim mechanizmem per-CPU lub globalnym z blokad.
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
        // Brak zada: wywoaj idle / halt (np. asm!("hlt"))
        return; 
    }

    let next = DEFAULT_SCHED_CLASS.pick_next_task(rq);

    if prev != next && !next.is_null() {
        // Opcjonalnie: tutaj zrzu stan FPU/SSE jeli task tego wymaga
        
        set_current_task(next);
        switch_to(prev, next);
    }
}

// Funkcja wywoywana z handlera przerwania czasowego (timer tick)
pub unsafe fn scheduler_tick() {
    let curr = get_current_task();
    if !curr.is_null() {
        DEFAULT_SCHED_CLASS.task_tick(get_rq(), curr);
        // Tutaj ustaw flag need_resched, a nie wymuszaj schedule() bezporednio w przerwaniu,
        // aby unikn zagniedania przecze kontekstu.
    }
}

// przepisze sie to za chwile
