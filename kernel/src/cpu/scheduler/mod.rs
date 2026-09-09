pub mod entities;
pub mod collections;
pub mod power;
pub mod smp;
pub mod policies;
pub mod debug;
pub mod time;
pub mod tests;

use core::sync::atomic::{AtomicU32, Ordering};
use crate::cpu::scheduler::entities::runqueue::RunQueue;
use crate::cpu::scheduler::entities::task::{TaskStruct, MAX_CPUS};

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

pub fn self_test() -> Result<(), &'static str> {
    Ok(())
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