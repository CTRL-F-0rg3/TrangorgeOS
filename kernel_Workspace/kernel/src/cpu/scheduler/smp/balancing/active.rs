#![allow(dead_code)]

use core::ptr;

use crate::cpu::scheduler::entities::task::CPU_NONE;
use crate::cpu::scheduler::runqueue::{DequeueFlags, EnqueueFlags};
use crate::cpu::scheduler::smp::migration::stopper;
use crate::cpu::scheduler::smp::topology::{SchedDomain, SchedGroup};

use super::calculate::LoadCalculation;

#[repr(C)]
struct ActiveBalanceArg {
    src_cpu: u32,
    dst_cpu: u32,
    sd_flags: u32,
}

/// Przenosi (przez `stop_one_cpu`, czyli bezpiecznie względem współbieżnego
/// `schedule()` na CPU źródłowym) bieżące zadanie najbardziej obciążonego
/// CPU na `cpu`, jeśli `calc` wskazuje na realną nierównowagę. Wcześniej ta
/// funkcja żyła (razem z `ActiveBalanceArg`/`active_balance_fn`) w
/// `smp/mod.rs`, mimo że logicznie należy do `balancing` — przeniesione tu,
/// żeby `mod active;` w `balancing/mod.rs` w ogóle coś zawierał.
pub unsafe fn active_balance(
    cpu: u32,
    sd: *mut SchedDomain,
    busiest: *mut SchedGroup,
    _local: *mut SchedGroup,
    calc: &LoadCalculation,
) {
    if calc.imbalance == 0 {
        return;
    }

    let busiest_cpu = (*busiest).group_mask.first().unwrap_or(CPU_NONE);
    if busiest_cpu == CPU_NONE || busiest_cpu == cpu {
        return;
    }

    let arg = ActiveBalanceArg {
        src_cpu: busiest_cpu,
        dst_cpu: cpu,
        sd_flags: (*sd).flags,
    };

    stopper::stop_one_cpu(busiest_cpu, active_balance_fn, &arg as *const _ as *mut core::ffi::c_void);
}

unsafe extern "C" fn active_balance_fn(arg: *mut core::ffi::c_void) -> u32 {
    let arg = &*(arg as *const ActiveBalanceArg);

    let src_rq_ptr = crate::cpu::scheduler::runqueue::get_rq(arg.src_cpu);
    let dst_rq_ptr = crate::cpu::scheduler::runqueue::get_rq(arg.dst_cpu);

    if src_rq_ptr.is_null() || dst_rq_ptr.is_null() {
        return 1;
    }

    let src_rq = &mut *src_rq_ptr;
    let dst_rq = &mut *dst_rq_ptr;

    let task = src_rq.current();
    if task.is_null() {
        return 1;
    }
    if !(*task).se.cpus_allowed.is_set(arg.dst_cpu) {
        return 1;
    }

    src_rq.dequeue_task(task, DequeueFlags::DEQUEUE_MIGRATING);
    (*task).record_migration(arg.dst_cpu);
    dst_rq.enqueue_task(task, EnqueueFlags::ENQUEUE_MIGRATED);

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::scheduler::smp::topology::SchedGroup;

    #[test]
    fn active_balance_is_noop_with_zero_imbalance() {
        let mut sg = SchedGroup::empty();
        sg.group_mask.set(3);
        let mut sd = crate::cpu::scheduler::smp::topology::SchedDomain::empty();
        let calc = LoadCalculation::empty();
        unsafe {
            active_balance(0, &mut sd, &mut sg, ptr::null_mut(), &calc);
        }
    }

    #[test]
    fn active_balance_is_noop_when_target_equals_busiest() {
        let mut sg = SchedGroup::empty();
        sg.group_mask.set(2);
        let mut sd = crate::cpu::scheduler::smp::topology::SchedDomain::empty();
        let mut calc = LoadCalculation::empty();
        calc.imbalance = 10;
        unsafe {
            // busiest == cpu -> nie ma sensu "aktywnie" ściągać zadania z
            // samego siebie na siebie.
            active_balance(2, &mut sd, &mut sg, ptr::null_mut(), &calc);
        }
    }
}