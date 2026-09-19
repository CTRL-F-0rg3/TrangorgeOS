pub mod entities;
pub mod collections;
pub mod power;
pub mod smp;
pub mod policies;
pub mod debug;
pub mod time;
#[cfg(test)]
pub mod tests;

pub use collections::cpumask as cpumask;
pub use entities::runqueue as runqueue;

use core::sync::atomic::{AtomicU32, Ordering};
use crate::cpu::scheduler::entities::runqueue::{DequeueFlags, EnqueueFlags, RunQueue};
use crate::cpu::scheduler::entities::task::{SchedPolicy, TaskStruct, MAX_CPUS};

static TOTAL_CPUS: AtomicU32 = AtomicU32::new(0);
static mut RUN_QUEUES: [*mut RunQueue; MAX_CPUS] = [core::ptr::null_mut(); MAX_CPUS];

pub unsafe fn init(nr_cpus: usize) {
    TOTAL_CPUS.store(nr_cpus as u32, Ordering::Release);

    crate::cpu::scheduler::collections::cpumask::mark_possible(0);
    crate::cpu::scheduler::collections::cpumask::mark_online(0);

    crate::cpu::scheduler::smp::topology::topology_init();
    crate::cpu::scheduler::smp::migration::stopper::stopper_init();
    crate::cpu::scheduler::power::em::em_init();
    crate::cpu::scheduler::power::power_init();

    // Bring the BSP runqueue online with its idle task (pid 0).
    let idle = alloc::boxed::Box::leak(alloc::boxed::Box::new(TaskStruct::blank()));
    idle.init_test_stub(0, SchedPolicy::Idle, 0);

    let rq = alloc::boxed::Box::leak(alloc::boxed::Box::new(RunQueue::new(
        0,
        idle as *mut TaskStruct,
    )));
    rq.bind_idle_task();

    RUN_QUEUES[0] = rq as *mut RunQueue;
}

pub unsafe fn tick(cpu: u32, delta_ns: u64) -> Result<(), &'static str> {
    if cpu as usize >= MAX_CPUS {
        return Err("Invalid CPU");
    }

    let rq_ptr = RUN_QUEUES[cpu as usize];
    if rq_ptr.is_null() {
        return Err("RunQueue not initialized");
    }

    let rq = &mut *rq_ptr;
    let flags = rq.lock.lock_irqsave();

    rq.advance_clock(delta_ns);
    rq.task_tick();

    if rq.nr_switches.load(Ordering::Relaxed) % 100 == 0 {
        crate::cpu::scheduler::smp::trigger_load_balance(cpu, rq);
    }

    rq.lock.unlock_irqrestore(flags);
    Ok(())
}

pub fn self_test() -> Result<&'static str, &'static str> {
    unsafe {
        let rq_ptr = get_rq(0);
        if rq_ptr.is_null() {
            return Err("BSP runqueue is not online");
        }

        let rq = &mut *rq_ptr;

        // Roundtrip: enqueue a fair task, pick it, then remove it again.
        let task = alloc::boxed::Box::leak(alloc::boxed::Box::new(TaskStruct::blank()));
        task.init_test_stub(100, SchedPolicy::Normal, 0);

        let flags = rq.lock.lock_irqsave();

        let idle_before = rq.is_idle();
        rq.enqueue_task(task as *mut TaskStruct, EnqueueFlags::ENQUEUE_NEW);
        let nr_after_enqueue = rq.nr_running();
        let picked = rq.pick_next_task();

        let picked_ok = core::ptr::eq(picked, task as *mut TaskStruct);

        rq.dequeue_task(task as *mut TaskStruct, DequeueFlags::DEQUEUE_SLEEP);
        let nr_after_dequeue = rq.nr_running();

        rq.lock.unlock_irqrestore(flags);

        if !idle_before || nr_after_enqueue != 1 || !picked_ok || nr_after_dequeue != 0 {
            return Err("scheduler enqueue/pick/dequeue roundtrip failed");
        }

        Ok("scheduler: BSP runqueue online + enqueue/pick/dequeue roundtrip")
    }
}

pub unsafe fn get_rq(cpu: u32) -> *mut RunQueue {
    if cpu as usize >= MAX_CPUS {
        return core::ptr::null_mut();
    }
    RUN_QUEUES[cpu as usize]
}

pub fn current_cpu_id() -> u32 {
    0
}
