pub mod topology;
pub mod ipi;
pub mod migration;
pub mod balancing;

use core::sync::atomic::Ordering;

use crate::cpu::scheduler::runqueue::RunQueue;
use balancing::calculate::{LoadCalculation, find_busiest_group, should_we_balance};

pub unsafe fn trigger_load_balance(cpu: u32, rq: &mut RunQueue) {
    let sd_ptr = topology::GLOBAL_TOPOLOGY.per_cpu_domains[cpu as usize];
    if sd_ptr.is_null() {
        return;
    }

    let mut sd = sd_ptr;
    while !sd.is_null() {
        if !should_we_balance(sd, cpu) {
            sd = (*sd).parent;
            continue;
        }

        let mut calc = LoadCalculation::empty();
        let (busiest, local) = find_busiest_group(sd, cpu, &mut calc);

        if !busiest.is_null() && !local.is_null() && calc.imbalance > 0 {
            balancing::active::active_balance(cpu, sd, busiest, local, &calc);
        }

        (*sd).last_balance.store(rq.clock.load(Ordering::Relaxed), Ordering::Release);
        sd = (*sd).parent;
    }
}