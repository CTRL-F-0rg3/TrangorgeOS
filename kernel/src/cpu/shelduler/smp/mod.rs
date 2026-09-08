pub mod topology;
pub mod ipi;
pub mod migration;
pub mod balancing;

use crate::cpu::scheduler::runqueue::RunQueue;
use balancing::calculate::{LoadCalculation, find_busiest_group, should_we_balance};
use migration::stopper;

pub unsafe fn trigger_load_balance(cpu: u32, rq: &mut RunQueue) {
    let sd_ptr = topology::GLOBAL_TOPOLOGY.per_cpu_domains[cpu as usize];
    if sd_ptr.is_null() { return; }

    let mut sd = sd_ptr;
    while !sd.is_null() {
        if !should_we_balance(sd, cpu) {
            sd = (*sd).parent;
            continue;
        }

        let mut calc = LoadCalculation::empty();
        let (busiest, local) = find_busiest_group(sd, cpu, &mut calc);

        if !busiest.is_null() && !local.is_null() && calc.imbalance > 0 {
            active_balance(cpu, sd, busiest, local, &calc);
        }

        (*sd).last_balance.store(rq.clock.load(Ordering::Relaxed), Ordering::Release);
        sd = (*sd).parent;
    }
}

unsafe fn active_balance(cpu: u32, sd: *mut topology::SchedDomain, busiest: *mut topology::SchedGroup, local: *mut topology::SchedGroup, calc: &LoadCalculation) {
    let busiest_cpu = (*busiest).group_mask.first().unwrap_or(CPU_NONE);
    if busiest_cpu == CPU_NONE || busiest_cpu == cpu { return; }

    let arg = ActiveBalanceArg {
        src_cpu: busiest_cpu,
        dst_cpu: cpu,
        sd_flags: (*sd).flags,
    };

    stopper::stop_one_cpu(busiest_cpu, active_balance_fn, &arg as *const _ as *mut core::ffi::c_void);
}

#[repr(C)]
struct ActiveBalanceArg {
    src_cpu: u32,
    dst_cpu: u32,
    sd_flags: u32,
}

unsafe extern "C" fn active_balance_fn(arg: *mut core::ffi::c_void) -> u32 {
    let arg = &*(arg as *const ActiveBalanceArg);
    
    let src_rq_ptr = crate::cpu::scheduler::runqueue::get_rq(arg.src_cpu);
    let dst_rq_ptr = crate::cpu::scheduler::runqueue::get_rq(arg.dst_cpu);
    
    if src_rq_ptr.is_null() || dst_rq_ptr.is_null() { return 1; }
    
    let src_rq = &mut *src_rq_ptr;
    let dst_rq = &mut *dst_rq_ptr;
    
    let task = src_rq.current();
    if task.is_null() { return 1; }
    
    if !(*task).se.cpus_allowed.is_set(arg.dst_cpu) { return 1; }
    
    src_rq.dequeue_task(task, crate::cpu::scheduler::runqueue::DequeueFlags::DEQUEUE_MIGRATING);
    (*task).record_migration(arg.dst_cpu);
    dst_rq.enqueue_task(task, crate::cpu::scheduler::runqueue::EnqueueFlags::ENQUEUE_MIGRATED);
    
    0
}