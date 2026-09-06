#![allow(dead_code)]

use core::sync::atomic::Ordering;

use crate::cpu::scheduler::arch_hooks;
use crate::cpu::scheduler::bitmap::{words_for_bits, AtomicBitmap};
use crate::cpu::scheduler::class;
use crate::cpu::scheduler::entities::task::{
    weight_to_nice, CpuMask, SchedPolicy, TaskError, TaskFlags, TaskId, TaskState, TaskStruct,
    MAX_RT_PRIO, NICE_MAX, NICE_MIN,
};
use crate::cpu::scheduler::runqueue::{DequeueFlags, EnqueueFlags, RunQueue};

// ------------------------------------------------------------------
// Alokator PID
// ------------------------------------------------------------------

pub const PID_MAX: usize = 32_768;
const PID_BITMAP_WORDS: usize = words_for_bits(PID_MAX);
static PID_BITMAP: AtomicBitmap<PID_BITMAP_WORDS> = AtomicBitmap::new();

pub fn alloc_pid() -> Option<TaskId> {
    PID_BITMAP.find_first_zero_and_set().map(|b| b as TaskId)
}

pub fn free_pid(pid: TaskId) {
    if (pid as usize) < PID_MAX {
        PID_BITMAP.test_and_clear(pid as usize);
    }
}

/// Rezerwuje konkretny PID (np. 0 dla `idle`/`swapper`, albo odtworzenie
/// procesu z checkpointa). Zwraca `false`, jeśli PID był już zajęty.
pub fn reserve_pid(pid: TaskId) -> bool {
    if (pid as usize) >= PID_MAX {
        return false;
    }
    !PID_BITMAP.test_and_set(pid as usize)
}

pub fn pid_in_use(pid: TaskId) -> bool {
    (pid as usize) < PID_MAX && PID_BITMAP.test(pid as usize)
}

// ------------------------------------------------------------------
// Rozruch CPU
// ------------------------------------------------------------------

pub unsafe fn boot_bsp(cpu: u32, apic_id: u32, idle: *mut TaskStruct, rq: &mut RunQueue) {
    reserve_pid((*idle).pid);
    rq.bind_idle_task();
    arch_hooks::register_apic_id(cpu, apic_id);
    arch_hooks::register_runqueue(cpu, rq as *mut RunQueue);
    arch_hooks::init_local_apic();
    arch_hooks::init_local_timer();
}

pub unsafe fn boot_ap(cpu: u32, apic_id: u32, idle: *mut TaskStruct, rq: &mut RunQueue) -> ! {
    reserve_pid((*idle).pid);
    rq.bind_idle_task();
    arch_hooks::cpu_bringup(cpu, apic_id, rq as *mut RunQueue)
}

// ------------------------------------------------------------------
// Cykl życia zadania: fork -> wake_up_new_task -> ... -> exit -> reap
// ------------------------------------------------------------------

pub unsafe fn sched_fork(
    parent: *mut TaskStruct,
    child: *mut TaskStruct,
    entry_point: usize,
    arg: usize,
) -> Result<TaskId, TaskError> {
    let pid = alloc_pid().ok_or(TaskError::ResourceExhausted)?;
    match (*parent).fork(&mut *child, pid, entry_point, arg) {
        Ok(()) => {
            let _ = (*child).set_state(TaskState::Interruptible);
            Ok(pid)
        }
        Err(e) => {
            free_pid(pid);
            Err(e)
        }
    }
}

pub unsafe fn spawn_kernel_thread(
    child: &mut TaskStruct,
    entry_point: usize,
    arg: usize,
    name: &str,
) -> Result<TaskId, TaskError> {
    let pid = alloc_pid().ok_or(TaskError::ResourceExhausted)?;
    match child.kthread_create(pid, entry_point, arg, name) {
        Ok(()) => Ok(pid),
        Err(e) => {
            free_pid(pid);
            Err(e)
        }
    }
}

pub unsafe fn wake_up_new_task(child: *mut TaskStruct, registry: &[*mut RunQueue]) -> Result<(), TaskError> {
    (*child).set_state(TaskState::Runnable)?;

    let target_cpu = class::select_task_rq(child, registry);
    let rq_ptr = match registry.get(target_cpu as usize) {
        Some(&p) if !p.is_null() => p,
        _ => return Err(TaskError::ResourceExhausted),
    };
    let rq = &mut *rq_ptr;

    let flags = rq.lock.lock_irqsave();
    (*child).se.last_cpu = target_cpu;
    (*child).cpu.store(target_cpu, Ordering::Release);
    class::enqueue(rq, child, EnqueueFlags::ENQUEUE_NEW);
    let need_kick = class::check_preempt(rq, child);
    rq.lock.unlock_irqrestore(flags);

    if need_kick {
        rq.resched_curr();
        if target_cpu != arch_hooks::current_cpu_id() {
            arch_hooks::send_resched_ipi(target_cpu);
        }
    }
    Ok(())
}

pub unsafe fn wake_up_process(task: *mut TaskStruct) {
    arch_hooks::wake_up(task);
}

unsafe fn exit_bookkeeping(task: *mut TaskStruct) -> *mut RunQueue {
    let rq_ptr = (*task).rq_ptr() as *mut RunQueue;
    if !rq_ptr.is_null() {
        let rq = &mut *rq_ptr;
        let flags = rq.lock.lock_irqsave();
        if (*task).se.on_rq {
            class::dequeue(rq, task, DequeueFlags::empty());
        }
        rq.lock.unlock_irqrestore(flags);
    }
    (*task).flags.fetch_insert(TaskFlags::PF_EXITING);
    let _ = (*task).set_state(TaskState::Zombie);
    rq_ptr
}

pub unsafe fn do_exit(task: *mut TaskStruct) -> ! {
    let rq_ptr = exit_bookkeeping(task);
    if !rq_ptr.is_null() {
        arch_hooks::context_switch(&mut *rq_ptr);
    }
    loop {
        core::hint::spin_loop();
    }
}

pub unsafe fn reap_zombie(child: *mut TaskStruct) {
    let pid = (*child).pid;
    (*child).destroy();
    free_pid(pid);
}

pub unsafe fn maybe_reschedule(cpu: u32, registry: &[*mut RunQueue]) {
    if let Some(&rq_ptr) = registry.get(cpu as usize) {
        if !rq_ptr.is_null() {
            let rq = &mut *rq_ptr;
            if (*rq.current()).needs_resched() {
                arch_hooks::context_switch(rq);
            }
        }
    }
}

// ------------------------------------------------------------------
// Wywołania systemowe planisty
// ------------------------------------------------------------------

pub unsafe fn sys_sched_yield(rq: &mut RunQueue) {
    let flags = rq.lock.lock_irqsave();
    let curr = rq.current();
    if !curr.is_null() {
        class::yield_task(rq, curr);
    }
    rq.lock.unlock_irqrestore(flags);
}

pub unsafe fn sys_nice(task: *mut TaskStruct, increment: i8) -> Result<(), TaskError> {
    if !(*task).policy.is_fair() {
        return Err(TaskError::InvalidNice);
    }
    let current_nice = weight_to_nice((*task).se.weight);
    let new_nice = (current_nice as i32 + increment as i32).clamp(NICE_MIN as i32, NICE_MAX as i32) as i8;
    (*task).set_nice(new_nice)
}

pub unsafe fn sys_sched_setscheduler(rq: &mut RunQueue, task: *mut TaskStruct, policy: SchedPolicy) {
    let flags = rq.lock.lock_irqsave();
    class::change_task_class(rq, task, policy);
    rq.lock.unlock_irqrestore(flags);
}

pub unsafe fn sys_sched_getscheduler(task: *const TaskStruct) -> SchedPolicy {
    (*task).policy
}

pub unsafe fn sys_sched_rr_get_interval(task: *const TaskStruct) -> u32 {
    class::get_rr_interval(task)
}

pub unsafe fn sys_sched_setaffinity(task: *mut TaskStruct, mask: CpuMask) -> Result<(), TaskError> {
    (*task).set_affinity(mask)
}

pub unsafe fn sys_sched_getaffinity(task: *const TaskStruct) -> CpuMask {
    (*task).se.cpus_allowed
}

pub fn sys_sched_get_priority_max(policy: SchedPolicy) -> i32 {
    if policy.is_realtime() {
        MAX_RT_PRIO - 1
    } else {
        0
    }
}

pub fn sys_sched_get_priority_min(policy: SchedPolicy) -> i32 {
    if policy.is_realtime() {
        0
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::scheduler::entities::task::TaskStruct as TaskStructT;

    fn make_idle(pid: TaskId) -> TaskStructT {
        let mut t = TaskStructT::blank();
        t.init_test_stub(pid, SchedPolicy::Idle, 0);
        t
    }

    fn make_task(pid: TaskId, policy: SchedPolicy, nice: i8) -> TaskStructT {
        let mut t = TaskStructT::blank();
        t.init_test_stub(pid, policy, nice);
        t
    }

    fn ptr_of(t: &mut TaskStructT) -> *mut TaskStructT {
        t as *mut TaskStructT
    }

    #[test]
    fn two_consecutive_allocations_are_distinct() {
        let a = alloc_pid().unwrap();
        let b = alloc_pid().unwrap();
        assert_ne!(a, b);
        free_pid(a);
        free_pid(b);
    }

    #[test]
    fn reserve_and_free_pid_roundtrip_on_a_dedicated_high_pid() {
        let pid = 25_000;
        assert!(reserve_pid(pid));
        assert!(!reserve_pid(pid));
        assert!(pid_in_use(pid));
        free_pid(pid);
        assert!(!pid_in_use(pid));
        assert!(reserve_pid(pid));
        free_pid(pid);
    }

    #[test]
    fn reserve_pid_rejects_out_of_range() {
        assert!(!reserve_pid(PID_MAX as TaskId + 1));
    }

    #[test]
    fn boot_bsp_registers_apic_and_runqueue() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(200, ptr_of(&mut idle));
        unsafe {
            boot_bsp(200, 42, ptr_of(&mut idle), &mut rq);
        }
        assert!(core::ptr::eq(arch_hooks::snapshot_registry()[200], &rq as *const RunQueue as *mut RunQueue));
        arch_hooks::register_runqueue(200, core::ptr::null_mut());
    }

    #[test]
    fn wake_up_new_task_enqueues_and_marks_runnable() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();

        let mut child = make_task(90_001, SchedPolicy::Normal, 0);
        child.set_state(TaskState::Interruptible).unwrap();

        let registry: [*mut RunQueue; 1] = [&mut rq as *mut RunQueue];
        unsafe {
            wake_up_new_task(ptr_of(&mut child), &registry).unwrap();
            assert_eq!((*ptr_of(&mut child)).state(), TaskState::Runnable);
            assert!((*ptr_of(&mut child)).se.on_rq);
        }
        assert_eq!(rq.fair.nr_running, 1);
    }

    #[test]
    fn exit_bookkeeping_dequeues_and_marks_zombie() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();
        let mut t = make_task(1, SchedPolicy::Normal, 0);

        unsafe {
            let flags = rq.lock.lock_irqsave();
            class::enqueue(&mut rq, ptr_of(&mut t), EnqueueFlags::ENQUEUE_NEW);
            rq.lock.unlock_irqrestore(flags);

            let returned_rq = exit_bookkeeping(ptr_of(&mut t));
            assert!(core::ptr::eq(returned_rq, &mut rq as *mut RunQueue));
            assert_eq!((*ptr_of(&mut t)).state(), TaskState::Zombie);
            assert!(!(*ptr_of(&mut t)).se.on_rq);
            assert!((*ptr_of(&mut t)).flags.contains(TaskFlags::PF_EXITING));
        }
    }

    #[test]
    fn exit_bookkeeping_on_unqueued_task_returns_null_rq() {
        let mut t = make_task(1, SchedPolicy::Normal, 0);
        unsafe {
            let rq_ptr = exit_bookkeeping(ptr_of(&mut t));
            assert!(rq_ptr.is_null());
            assert_eq!((*ptr_of(&mut t)).state(), TaskState::Zombie);
        }
    }

    #[test]
    fn reap_zombie_destroys_and_frees_pid() {
        let mut t = make_task(1, SchedPolicy::Normal, 0);
        let pid = 26_000;
        t.pid = pid;
        assert!(reserve_pid(pid));

        unsafe {
            let _ = t.set_state(TaskState::Zombie);
            reap_zombie(ptr_of(&mut t));
        }

        assert_eq!(t.state(), TaskState::Dead);
        assert!(!pid_in_use(pid));
    }

    #[test]
    fn maybe_reschedule_is_noop_when_not_needed() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();
        let registry: [*mut RunQueue; 1] = [&mut rq as *mut RunQueue];
        unsafe {
            maybe_reschedule(0, &registry);
        }
    }

    #[test]
    fn sys_nice_clamps_to_valid_range() {
        let mut t = make_task(1, SchedPolicy::Normal, 0);
        unsafe {
            sys_nice(ptr_of(&mut t), 100).unwrap();
            assert_eq!(weight_to_nice((*ptr_of(&mut t)).se.weight), NICE_MAX);
            sys_nice(ptr_of(&mut t), -100).unwrap();
            assert_eq!(weight_to_nice((*ptr_of(&mut t)).se.weight), NICE_MIN);
        }
    }

    #[test]
    fn sys_nice_rejects_non_fair_policy() {
        let mut t = make_task(1, SchedPolicy::Fifo, 0);
        unsafe {
            assert!(sys_nice(ptr_of(&mut t), 1).is_err());
        }
    }

    #[test]
    fn sys_sched_setscheduler_moves_task_to_new_class() {
        let mut idle = make_idle(0);
        let mut rq = RunQueue::new(0, ptr_of(&mut idle));
        rq.bind_idle_task();
        let mut t = make_task(1, SchedPolicy::Normal, 0);

        unsafe {
            let flags = rq.lock.lock_irqsave();
            class::enqueue(&mut rq, ptr_of(&mut t), EnqueueFlags::ENQUEUE_NEW);
            rq.lock.unlock_irqrestore(flags);

            sys_sched_setscheduler(&mut rq, ptr_of(&mut t), SchedPolicy::Fifo);
            assert_eq!(sys_sched_getscheduler(ptr_of(&mut t)), SchedPolicy::Fifo);
        }
        assert_eq!(rq.rt.nr_running, 1);
        assert_eq!(rq.fair.nr_running, 0);
    }

    #[test]
    fn sys_sched_rr_get_interval_matches_class_dispatch() {
        let fifo = make_task(1, SchedPolicy::Fifo, 0);
        let rr = make_task(2, SchedPolicy::RoundRobin, 0);
        unsafe {
            assert_eq!(sys_sched_rr_get_interval(&fifo as *const _), 0);
            assert!(sys_sched_rr_get_interval(&rr as *const _) > 0);
        }
    }

    #[test]
    fn sys_sched_affinity_roundtrips() {
        let mut t = make_task(1, SchedPolicy::Normal, 0);
        unsafe {
            let mask = CpuMask::single(3);
            sys_sched_setaffinity(ptr_of(&mut t), mask).unwrap();
            assert_eq!(sys_sched_getaffinity(ptr_of(&mut t)), mask);
        }
    }

    #[test]
    fn priority_range_helpers_match_posix_expectations() {
        assert_eq!(sys_sched_get_priority_min(SchedPolicy::Fifo), 0);
        assert_eq!(sys_sched_get_priority_max(SchedPolicy::Fifo), MAX_RT_PRIO - 1);
        assert_eq!(sys_sched_get_priority_max(SchedPolicy::Normal), 0);
        assert_eq!(sys_sched_get_priority_min(SchedPolicy::Normal), 0);
    }
}