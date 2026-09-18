#![allow(dead_code)]

use core::ptr;
use core::sync::atomic::Ordering;

use crate::cpu::scheduler::entities::task::{
    default_time_slice, weight_to_nice, SchedClass, SchedPolicy, TaskState, TaskStruct,
    DEFAULT_PRIO, MAX_PRIO, MAX_RT_PRIO,
};
use crate::cpu::scheduler::runqueue::{smp, DequeueFlags, EnqueueFlags, RunQueue};

pub struct SchedClassOps {
    pub name: &'static str,
    pub class: SchedClass,
    pub next: Option<&'static SchedClassOps>,

    pub enqueue_task: unsafe fn(&mut RunQueue, *mut TaskStruct, EnqueueFlags),
    pub dequeue_task: unsafe fn(&mut RunQueue, *mut TaskStruct, DequeueFlags),
    pub yield_task: unsafe fn(&mut RunQueue, *mut TaskStruct),
    pub pick_next_task: unsafe fn(&mut RunQueue) -> *mut TaskStruct,
    pub check_preempt_curr: unsafe fn(&RunQueue, *mut TaskStruct) -> bool,
    pub task_tick: unsafe fn(&mut RunQueue, *mut TaskStruct),
    pub charge_exec: unsafe fn(&mut RunQueue, *mut TaskStruct, u64, u64),
    pub get_rr_interval: unsafe fn(*const TaskStruct) -> u32,
    pub select_task_rq: unsafe fn(*const TaskStruct, &[*mut RunQueue]) -> u32,
}

// ------------------------------------------------------------------
// Idle
// ------------------------------------------------------------------

unsafe fn idle_enqueue(_rq: &mut RunQueue, task: *mut TaskStruct, _flags: EnqueueFlags) {
    (*task).se.on_rq = true;
}

unsafe fn idle_dequeue(_rq: &mut RunQueue, task: *mut TaskStruct, _flags: DequeueFlags) {
    (*task).se.on_rq = false;
}

unsafe fn idle_yield(_rq: &mut RunQueue, _task: *mut TaskStruct) {}

unsafe fn idle_pick_next(rq: &mut RunQueue) -> *mut TaskStruct {
    rq.idle
}

unsafe fn idle_check_preempt(_rq: &RunQueue, _candidate: *mut TaskStruct) -> bool {
    false
}

unsafe fn idle_task_tick(_rq: &mut RunQueue, _curr: *mut TaskStruct) {
}

unsafe fn idle_charge_exec(_rq: &mut RunQueue, _task: *mut TaskStruct, _delta_exec: u64, _now: u64) {}

unsafe fn idle_get_rr_interval(_task: *const TaskStruct) -> u32 {
    0
}

unsafe fn idle_select_task_rq(_task: *const TaskStruct, _registry: &[*mut RunQueue]) -> u32 {
    0
}

pub static IDLE_SCHED_CLASS: SchedClassOps = SchedClassOps {
    name: "idle",
    class: SchedClass::Idle,
    next: None,
    enqueue_task: idle_enqueue,
    dequeue_task: idle_dequeue,
    yield_task: idle_yield,
    pick_next_task: idle_pick_next,
    check_preempt_curr: idle_check_preempt,
    task_tick: idle_task_tick,
    charge_exec: idle_charge_exec,
    get_rr_interval: idle_get_rr_interval,
    select_task_rq: idle_select_task_rq,
};

// ------------------------------------------------------------------
// Fair (CFS/EEVDF-lite)
// ------------------------------------------------------------------

unsafe fn fair_enqueue(rq: &mut RunQueue, task: *mut TaskStruct, flags: EnqueueFlags) {
    rq.fair.enqueue(task, flags);
}

unsafe fn fair_dequeue(rq: &mut RunQueue, task: *mut TaskStruct, _flags: DequeueFlags) {
    rq.fair.dequeue(task);
}

unsafe fn fair_yield(rq: &mut RunQueue, task: *mut TaskStruct) {
    rq.yield_task(task);
}

unsafe fn fair_pick_next(rq: &mut RunQueue) -> *mut TaskStruct {
    rq.fair.pick_first()
}

unsafe fn fair_check_preempt(rq: &RunQueue, candidate: *mut TaskStruct) -> bool {
    let curr = rq.current();
    if curr.is_null() {
        return false;
    }
    rq.fair.should_preempt(curr, candidate)
}

unsafe fn fair_task_tick(_rq: &mut RunQueue, curr: *mut TaskStruct) {
    (*curr).set_need_resched();
}

unsafe fn fair_charge_exec(rq: &mut RunQueue, task: *mut TaskStruct, delta_exec: u64, now: u64) {
    rq.fair.charge_exec(task, delta_exec, now);
}

unsafe fn fair_get_rr_interval(_task: *const TaskStruct) -> u32 {
    0
}

unsafe fn fair_select_task_rq(task: *const TaskStruct, registry: &[*mut RunQueue]) -> u32 {
    smp::select_task_rq(task, registry)
}

pub static FAIR_SCHED_CLASS: SchedClassOps = SchedClassOps {
    name: "fair",
    class: SchedClass::Fair,
    next: Some(&IDLE_SCHED_CLASS),
    enqueue_task: fair_enqueue,
    dequeue_task: fair_dequeue,
    yield_task: fair_yield,
    pick_next_task: fair_pick_next,
    check_preempt_curr: fair_check_preempt,
    task_tick: fair_task_tick,
    charge_exec: fair_charge_exec,
    get_rr_interval: fair_get_rr_interval,
    select_task_rq: fair_select_task_rq,
};

// ------------------------------------------------------------------
// RealTime (SCHED_FIFO / SCHED_RR)
// ------------------------------------------------------------------

unsafe fn rt_enqueue(rq: &mut RunQueue, task: *mut TaskStruct, flags: EnqueueFlags) {
    rq.rt.enqueue(task, flags);
}

unsafe fn rt_dequeue(rq: &mut RunQueue, task: *mut TaskStruct, _flags: DequeueFlags) {
    rq.rt.dequeue(task);
}

unsafe fn rt_yield(rq: &mut RunQueue, task: *mut TaskStruct) {
    if (*task).se.on_rq {
        rq.rt.requeue(task);
    }
    (*task).set_need_resched();
}

unsafe fn rt_pick_next(rq: &mut RunQueue) -> *mut TaskStruct {
    rq.rt.pick_first()
}

unsafe fn rt_check_preempt(rq: &RunQueue, candidate: *mut TaskStruct) -> bool {
    let curr = rq.current();
    if curr.is_null() {
        return false;
    }
    (*candidate).prio < (*curr).prio
}

unsafe fn rt_task_tick(rq: &mut RunQueue, curr: *mut TaskStruct) {
    if (*curr).policy != SchedPolicy::RoundRobin {
        return;
    }
    if (*curr).rt.time_slice > 0 {
        return;
    }
    (*curr).rt.time_slice = default_time_slice(SchedPolicy::RoundRobin);
    if (*curr).se.on_rq {
        rq.rt.requeue(curr);
    }
    (*curr).set_need_resched();
}

unsafe fn rt_charge_exec(_rq: &mut RunQueue, task: *mut TaskStruct, delta_exec: u64, _now: u64) {
    if (*task).rt.time_slice == 0 {
        return;
    }
    let ticks = (delta_exec / 1_000_000) as u32;
    (*task).rt.time_slice = (*task).rt.time_slice.saturating_sub(ticks);
}

unsafe fn rt_get_rr_interval(task: *const TaskStruct) -> u32 {
    if (*task).policy == SchedPolicy::RoundRobin {
        default_time_slice(SchedPolicy::RoundRobin)
    } else {
        0
    }
}

unsafe fn rt_select_task_rq(task: *const TaskStruct, registry: &[*mut RunQueue]) -> u32 {
    smp::select_task_rq(task, registry)
}

pub static RT_SCHED_CLASS: SchedClassOps = SchedClassOps {
    name: "rt",
    class: SchedClass::RealTime,
    next: Some(&FAIR_SCHED_CLASS),
    enqueue_task: rt_enqueue,
    dequeue_task: rt_dequeue,
    yield_task: rt_yield,
    pick_next_task: rt_pick_next,
    check_preempt_curr: rt_check_preempt,
    task_tick: rt_task_tick,
    charge_exec: rt_charge_exec,
    get_rr_interval: rt_get_rr_interval,
    select_task_rq: rt_select_task_rq,
};

// ------------------------------------------------------------------
// Deadline (EDF + CBS)
// ------------------------------------------------------------------

unsafe fn dl_enqueue(rq: &mut RunQueue, task: *mut TaskStruct, flags: EnqueueFlags) {
    let now = rq.clock_task.load(Ordering::Relaxed);
    rq.dl.enqueue(task, now, flags);
}

unsafe fn dl_dequeue(rq: &mut RunQueue, task: *mut TaskStruct, flags: DequeueFlags) {
    let permanent = !flags.contains(DequeueFlags::DEQUEUE_SAVE);
    rq.dl.dequeue(task, permanent);
}

unsafe fn dl_yield(rq: &mut RunQueue, task: *mut TaskStruct) {
    (*task).dl.yielded = true;
    (*task).set_need_resched();
    let _ = rq;
}

unsafe fn dl_pick_next(rq: &mut RunQueue) -> *mut TaskStruct {
    rq.dl.pick_first()
}

unsafe fn dl_check_preempt(rq: &RunQueue, candidate: *mut TaskStruct) -> bool {
    let curr = rq.current();
    if curr.is_null() {
        return false;
    }
    rq.dl.should_preempt(curr, candidate)
}

unsafe fn dl_task_tick(rq: &mut RunQueue, curr: *mut TaskStruct) {
    if (*curr).dl.throttled {
        rq.dl.dequeue(curr, false);
        (*curr).set_need_resched();
    }
}

unsafe fn dl_charge_exec(rq: &mut RunQueue, task: *mut TaskStruct, delta_exec: u64, now: u64) {
    rq.dl.update_curr(task, delta_exec, now);
}

unsafe fn dl_get_rr_interval(_task: *const TaskStruct) -> u32 {
    0
}

unsafe fn dl_select_task_rq(task: *const TaskStruct, registry: &[*mut RunQueue]) -> u32 {
    smp::select_task_rq(task, registry)
}

pub static DL_SCHED_CLASS: SchedClassOps = SchedClassOps {
    name: "deadline",
    class: SchedClass::Deadline,
    next: Some(&RT_SCHED_CLASS),
    enqueue_task: dl_enqueue,
    dequeue_task: dl_dequeue,
    yield_task: dl_yield,
    pick_next_task: dl_pick_next,
    check_preempt_curr: dl_check_preempt,
    task_tick: dl_task_tick,
    charge_exec: dl_charge_exec,
    get_rr_interval: dl_get_rr_interval,
    select_task_rq: dl_select_task_rq,
};

// ------------------------------------------------------------------
// Stop
// ------------------------------------------------------------------

unsafe fn stop_enqueue(rq: &mut RunQueue, task: *mut TaskStruct, _flags: EnqueueFlags) {
    rq.stop.enqueue(task);
}

unsafe fn stop_dequeue(rq: &mut RunQueue, task: *mut TaskStruct, _flags: DequeueFlags) {
    rq.stop.dequeue(task);
}

unsafe fn stop_yield(_rq: &mut RunQueue, _task: *mut TaskStruct) {}

unsafe fn stop_pick_next(rq: &mut RunQueue) -> *mut TaskStruct {
    rq.stop.pick_first()
}

unsafe fn stop_check_preempt(_rq: &RunQueue, _candidate: *mut TaskStruct) -> bool {
    false
}

unsafe fn stop_task_tick(_rq: &mut RunQueue, _curr: *mut TaskStruct) {}

unsafe fn stop_charge_exec(_rq: &mut RunQueue, _task: *mut TaskStruct, _delta_exec: u64, _now: u64) {}

unsafe fn stop_get_rr_interval(_task: *const TaskStruct) -> u32 {
    0
}

unsafe fn stop_select_task_rq(_task: *const TaskStruct, _registry: &[*mut RunQueue]) -> u32 {
    0
}

pub static STOP_SCHED_CLASS: SchedClassOps = SchedClassOps {
    name: "stop",
    class: SchedClass::Stop,
    next: Some(&DL_SCHED_CLASS),
    enqueue_task: stop_enqueue,
    dequeue_task: stop_dequeue,
    yield_task: stop_yield,
    pick_next_task: stop_pick_next,
    check_preempt_curr: stop_check_preempt,
    task_tick: stop_task_tick,
    charge_exec: stop_charge_exec,
    get_rr_interval: stop_get_rr_interval,
    select_task_rq: stop_select_task_rq,
};

// ------------------------------------------------------------------
// Dispatch
// ------------------------------------------------------------------

pub fn class_of(class: SchedClass) -> &'static SchedClassOps {
    match class {
        SchedClass::Stop => &STOP_SCHED_CLASS,
        SchedClass::Deadline => &DL_SCHED_CLASS,
        SchedClass::RealTime => &RT_SCHED_CLASS,
        SchedClass::Fair => &FAIR_SCHED_CLASS,
        SchedClass::Idle => &IDLE_SCHED_CLASS,
    }
}

pub unsafe fn pick_next(rq: &mut RunQueue) -> *mut TaskStruct {
    let mut cursor: &'static SchedClassOps = &STOP_SCHED_CLASS;
    loop {
        let candidate = (cursor.pick_next_task)(rq);
        if !candidate.is_null() {
            return candidate;
        }
        match cursor.next {
            Some(next) => cursor = next,
            None => return rq.idle,
        }
    }
}

pub unsafe fn enqueue(rq: &mut RunQueue, task: *mut TaskStruct, flags: EnqueueFlags) {
    debug_assert!(!task.is_null());
    debug_assert!(rq.lock.is_locked());
    debug_assert!(!(*task).se.on_rq);

    let ops = class_of((*task).sched_class);
    (ops.enqueue_task)(rq, task, flags);

    (*task).set_rq_ptr(rq as *mut RunQueue as *mut core::ffi::c_void);
    rq.nr_running.fetch_add(1, Ordering::Relaxed);

    if flags.contains(EnqueueFlags::ENQUEUE_WAKEUP) {
        let now = rq.clock_task.load(Ordering::Relaxed);
        (*task).stats.last_enqueue_time = now;
        if check_preempt(rq, task) {
            rq.resched_curr();
        }
    }
}

pub unsafe fn dequeue(rq: &mut RunQueue, task: *mut TaskStruct, flags: DequeueFlags) {
    debug_assert!(!task.is_null());
    debug_assert!(rq.lock.is_locked());
    debug_assert!((*task).se.on_rq);

    let ops = class_of((*task).sched_class);
    (ops.dequeue_task)(rq, task, flags);

    if !flags.contains(DequeueFlags::DEQUEUE_MIGRATING) {
        (*task).set_rq_ptr(ptr::null_mut());
    }
    rq.nr_running.fetch_sub(1, Ordering::Relaxed);

    if flags.contains(DequeueFlags::DEQUEUE_SLEEP) && (*task).state() == TaskState::Uninterruptible {
        rq.nr_uninterruptible.fetch_add(1, Ordering::Relaxed);
    }
}

pub unsafe fn yield_task(rq: &mut RunQueue, task: *mut TaskStruct) {
    let ops = class_of((*task).sched_class);
    (ops.yield_task)(rq, task);
}

pub unsafe fn tick(rq: &mut RunQueue, curr: *mut TaskStruct) {
    let ops = class_of((*curr).sched_class);
    (ops.task_tick)(rq, curr);
}

pub unsafe fn charge(rq: &mut RunQueue, task: *mut TaskStruct, delta_exec: u64, now: u64) {
    let ops = class_of((*task).sched_class);
    (ops.charge_exec)(rq, task, delta_exec, now);
}

pub unsafe fn get_rr_interval(task: *const TaskStruct) -> u32 {
    let ops = class_of((*task).sched_class);
    (ops.get_rr_interval)(task)
}

pub unsafe fn select_task_rq(task: *const TaskStruct, registry: &[*mut RunQueue]) -> u32 {
    let ops = class_of((*task).sched_class);
    (ops.select_task_rq)(task, registry)
}

pub unsafe fn check_preempt(rq: &RunQueue, candidate: *mut TaskStruct) -> bool {
    let curr = rq.current();
    if curr.is_null() || core::ptr::eq(curr, candidate) {
        return false;
    }
    let curr_ops = class_of((*curr).sched_class);
    let cand_ops = class_of((*candidate).sched_class);
    if cand_ops.class != curr_ops.class {
        cand_ops.class > curr_ops.class
    } else {
        (curr_ops.check_preempt_curr)(rq, candidate)
    }
}

pub unsafe fn prio_changed(rq: &mut RunQueue, task: *mut TaskStruct, old_prio: i32) {
    if core::ptr::eq(task, rq.current()) {
        if (*task).prio > old_prio {
            rq.resched_curr();
        }
        return;
    }
    if (*task).se.on_rq && check_preempt(rq, task) {
        rq.resched_curr();
    }
}

pub unsafe fn switched_to(rq: &mut RunQueue, task: *mut TaskStruct) {
    if (*task).se.on_rq && !core::ptr::eq(task, rq.current()) && check_preempt(rq, task) {
        rq.resched_curr();
    }
}

unsafe fn recompute_prio(task: *mut TaskStruct, policy: SchedPolicy) {
    match policy {
        SchedPolicy::Fifo | SchedPolicy::RoundRobin => {
            let rt_prio = (*task).rt.rt_priority.min((MAX_RT_PRIO - 1) as u8);
            (*task).rt.rt_priority = rt_prio;
            (*task).static_prio = rt_prio as i32;
            (*task).normal_prio = rt_prio as i32;
            (*task).prio = rt_prio as i32;
            (*task).rt.time_slice = default_time_slice(policy);
        }
        SchedPolicy::Deadline => {
            (*task).static_prio = 0;
            (*task).normal_prio = 0;
            (*task).prio = 0;
        }
        SchedPolicy::Stop => {
            (*task).static_prio = -1;
            (*task).normal_prio = -1;
            (*task).prio = -1;
        }
        SchedPolicy::Idle => {
            (*task).static_prio = MAX_PRIO;
            (*task).normal_prio = MAX_PRIO;
            (*task).prio = MAX_PRIO;
        }
        SchedPolicy::Normal | SchedPolicy::Batch => {
            let nice = weight_to_nice((*task).se.weight);
            (*task).static_prio = DEFAULT_PRIO + nice as i32;
            (*task).normal_prio = (*task).static_prio;
            (*task).prio = (*task).normal_prio;
        }
    }
}

pub unsafe fn change_task_class(rq: &mut RunQueue, task: *mut TaskStruct, new_policy: SchedPolicy) {
    let was_queued = (*task).se.on_rq;
    let old_prio = (*task).prio;

    if was_queued {
        dequeue(rq, task, DequeueFlags::DEQUEUE_SAVE);
    }

    (*task).policy = new_policy;
    (*task).sched_class = SchedClass::from(new_policy);
    recompute_prio(task, new_policy);

    if was_queued {
        enqueue(rq, task, EnqueueFlags::ENQUEUE_RESTORE);
        switched_to(rq, task);
    } else {
        prio_changed(rq, task, old_prio);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::scheduler::entities::task::TaskId;

    fn make_idle(pid: TaskId) -> TaskStruct {
        let mut t = TaskStruct::blank();
        t.init_test_stub(pid, SchedPolicy::Idle, 0);
        t
    }

    fn make_task(pid: TaskId, policy: SchedPolicy, nice: i8) -> TaskStruct {
        let mut t = TaskStruct::blank();
        t.init_test_stub(pid, policy, nice);
        t
    }

    fn ptr_of(t: &mut TaskStruct) -> *mut TaskStruct {
        t as *mut TaskStruct
    }

    #[test]
    fn chain_is_ordered_stop_dl_rt_fair_idle() {
        assert_eq!(STOP_SCHED_CLASS.class, SchedClass::Stop);
        assert!(core::ptr::eq(STOP_SCHED_CLASS.next.unwrap(), &DL_SCHED_CLASS));
        assert_eq!(DL_SCHED_CLASS.class, SchedClass::Deadline);
        assert!(core::ptr::eq(DL_SCHED_CLASS.next.unwrap(), &RT_SCHED_CLASS));
        assert_eq!(RT_SCHED_CLASS.class, SchedClass::RealTime);
        assert!(core::ptr::eq(RT_SCHED_CLASS.next.unwrap(), &FAIR_SCHED_CLASS));
        assert_eq!(FAIR_SCHED_CLASS.class, SchedClass::Fair);
        assert!(core::ptr::eq(FAIR_SCHED_CLASS.next.unwrap(), &IDLE_SCHED_CLASS));
        assert_eq!(IDLE_SCHED_CLASS.class, SchedClass::Idle);
        assert!(IDLE_SCHED_CLASS.next.is_none());
    }

    #[test]
    fn class_of_maps_every_variant_correctly() {
        assert!(core::ptr::eq(class_of(SchedClass::Stop), &STOP_SCHED_CLASS));
        assert!(core::ptr::eq(class_of(SchedClass::Deadline), &DL_SCHED_CLASS));
        assert!(core::ptr::eq(class_of(SchedClass::RealTime), &RT_SCHED_CLASS));
        assert!(core::ptr::eq(class_of(SchedClass::Fair), &FAIR_SCHED_CLASS));
        assert!(core::ptr::eq(class_of(SchedClass::Idle), &IDLE_SCHED_CLASS));
    }

    #[test]
    fn pick_next_walks_the_chain_in_priority_order() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut fair_task = make_task(1, SchedPolicy::Normal, 0);
        let mut rt_task = make_task(2, SchedPolicy::Fifo, 0);
        rt_task.prio = 10;
        let mut dl_task = make_task(3, SchedPolicy::Deadline, 0);
        dl_task.dl.dl_runtime = 1000;
        dl_task.dl.dl_deadline = 100_000;
        dl_task.dl.dl_period = 100_000;
        let mut stop_task = make_task(4, SchedPolicy::Stop, 0);

        unsafe {
            let flags = rq.lock.lock_irqsave();

            assert!(core::ptr::eq(pick_next(&mut rq), ptr_of(&mut idle)));

            enqueue(&mut rq, ptr_of(&mut fair_task), EnqueueFlags::ENQUEUE_NEW);
            assert_eq!((*pick_next(&mut rq)).pid, 1);

            enqueue(&mut rq, ptr_of(&mut rt_task), EnqueueFlags::ENQUEUE_NEW);
            assert_eq!((*pick_next(&mut rq)).pid, 2);

            enqueue(&mut rq, ptr_of(&mut dl_task), EnqueueFlags::ENQUEUE_NEW);
            assert_eq!((*pick_next(&mut rq)).pid, 3);

            enqueue(&mut rq, ptr_of(&mut stop_task), EnqueueFlags::ENQUEUE_NEW);
            assert_eq!((*pick_next(&mut rq)).pid, 4);

            dequeue(&mut rq, ptr_of(&mut stop_task), DequeueFlags::empty());
            dequeue(&mut rq, ptr_of(&mut dl_task), DequeueFlags::empty());
            dequeue(&mut rq, ptr_of(&mut rt_task), DequeueFlags::empty());
            dequeue(&mut rq, ptr_of(&mut fair_task), DequeueFlags::empty());
            assert!(core::ptr::eq(pick_next(&mut rq), ptr_of(&mut idle)));

            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn rt_get_rr_interval_is_nonzero_only_for_round_robin() {
        let fifo = make_task(1, SchedPolicy::Fifo, 0);
        let rr = make_task(2, SchedPolicy::RoundRobin, 0);
        unsafe {
            assert_eq!(get_rr_interval(&fifo as *const _), 0);
            assert!(get_rr_interval(&rr as *const _) > 0);
        }
    }

    #[test]
    fn fair_and_deadline_and_idle_have_zero_rr_interval() {
        let fair = make_task(1, SchedPolicy::Normal, 0);
        let dl = make_task(2, SchedPolicy::Deadline, 0);
        let idle = make_idle(3);
        unsafe {
            assert_eq!(get_rr_interval(&fair as *const _), 0);
            assert_eq!(get_rr_interval(&dl as *const _), 0);
            assert_eq!(get_rr_interval(&idle as *const _), 0);
        }
    }

    #[test]
    fn check_preempt_cross_class_ignores_intra_class_ops() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut fair_task = make_task(1, SchedPolicy::Normal, 0);
        let mut rt_task = make_task(2, SchedPolicy::Fifo, 0);
        rt_task.prio = 5;

        unsafe {
            let flags = rq.lock.lock_irqsave();
            enqueue(&mut rq, ptr_of(&mut fair_task), EnqueueFlags::ENQUEUE_NEW);
            rq.set_curr_task(ptr_of(&mut fair_task));

            assert!(check_preempt(&rq, ptr_of(&mut rt_task)));

            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn switched_to_reschedules_when_new_class_outranks_current() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut fair_task = make_task(1, SchedPolicy::Normal, 0);
        let mut promoted = make_task(2, SchedPolicy::Normal, 0);

        unsafe {
            let flags = rq.lock.lock_irqsave();
            enqueue(&mut rq, ptr_of(&mut fair_task), EnqueueFlags::ENQUEUE_NEW);
            rq.set_curr_task(ptr_of(&mut fair_task));
            enqueue(&mut rq, ptr_of(&mut promoted), EnqueueFlags::ENQUEUE_NEW);
            assert!(!(*ptr_of(&mut fair_task)).needs_resched());

            change_task_class(&mut rq, ptr_of(&mut promoted), SchedPolicy::Fifo);

            assert_eq!((*ptr_of(&mut promoted)).sched_class, SchedClass::RealTime);
            assert!((*ptr_of(&mut fair_task)).needs_resched());

            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn change_task_class_moves_task_between_underlying_queues() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut t = make_task(1, SchedPolicy::Normal, 0);

        unsafe {
            let flags = rq.lock.lock_irqsave();
            enqueue(&mut rq, ptr_of(&mut t), EnqueueFlags::ENQUEUE_NEW);
            assert_eq!(rq.fair.nr_running, 1);
            assert_eq!(rq.rt.nr_running, 0);

            change_task_class(&mut rq, ptr_of(&mut t), SchedPolicy::Fifo);

            assert_eq!(rq.fair.nr_running, 0);
            assert_eq!(rq.rt.nr_running, 1);
            assert!((*ptr_of(&mut t)).se.on_rq);

            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn change_task_class_on_non_queued_task_does_not_touch_queues() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();
        let mut t = make_task(1, SchedPolicy::Normal, 0);

        unsafe {
            let flags = rq.lock.lock_irqsave();
            change_task_class(&mut rq, ptr_of(&mut t), SchedPolicy::Fifo);
            assert_eq!(rq.fair.nr_running, 0);
            assert_eq!(rq.rt.nr_running, 0);
            assert!(!(*ptr_of(&mut t)).se.on_rq);
            assert_eq!((*ptr_of(&mut t)).sched_class, SchedClass::RealTime);
            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn rt_task_tick_only_requeues_round_robin_on_slice_expiry() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut fifo_task = make_task(1, SchedPolicy::Fifo, 0);
        fifo_task.prio = 10;

        unsafe {
            let flags = rq.lock.lock_irqsave();
            enqueue(&mut rq, ptr_of(&mut fifo_task), EnqueueFlags::ENQUEUE_NEW);
            rq.set_curr_task(ptr_of(&mut fifo_task));

            tick(&mut rq, ptr_of(&mut fifo_task));
            assert!(!(*ptr_of(&mut fifo_task)).needs_resched(), "FIFO nie powinno dostać need_resched z samego ticka");

            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn rt_task_tick_marks_resched_when_round_robin_slice_expires() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut rr_task = make_task(1, SchedPolicy::RoundRobin, 0);
        rr_task.prio = 10;
        rr_task.rt.time_slice = 0;

        unsafe {
            let flags = rq.lock.lock_irqsave();
            enqueue(&mut rq, ptr_of(&mut rr_task), EnqueueFlags::ENQUEUE_NEW);
            rq.set_curr_task(ptr_of(&mut rr_task));

            tick(&mut rq, ptr_of(&mut rr_task));
            assert!((*ptr_of(&mut rr_task)).needs_resched());
            assert!((*ptr_of(&mut rr_task)).rt.time_slice > 0);

            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn dl_task_tick_dequeues_when_throttled() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut t = make_task(1, SchedPolicy::Deadline, 0);
        t.dl.dl_runtime = 1000;
        t.dl.dl_deadline = 100_000;
        t.dl.dl_period = 100_000;

        unsafe {
            let flags = rq.lock.lock_irqsave();
            enqueue(&mut rq, ptr_of(&mut t), EnqueueFlags::ENQUEUE_NEW);
            rq.set_curr_task(ptr_of(&mut t));
            (*ptr_of(&mut t)).dl.throttled = true;

            assert_eq!(rq.dl.dl_nr_running, 1);
            tick(&mut rq, ptr_of(&mut t));
            assert_eq!(rq.dl.dl_nr_running, 0);
            assert!((*ptr_of(&mut t)).needs_resched());

            rq.lock.unlock_irqrestore(flags);
        }
    }

    #[test]
    fn fair_charge_dispatches_to_underlying_rqfair() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();
        let mut t = make_task(1, SchedPolicy::Normal, 0);

        unsafe {
            let flags = rq.lock.lock_irqsave();
            enqueue(&mut rq, ptr_of(&mut t), EnqueueFlags::ENQUEUE_NEW);
            let before = (*ptr_of(&mut t)).se.vruntime;
            charge(&mut rq, ptr_of(&mut t), 1_000_000, 1_000_000);
            assert!((*ptr_of(&mut t)).se.vruntime > before);
            rq.lock.unlock_irqrestore(flags);
        }
    }
}