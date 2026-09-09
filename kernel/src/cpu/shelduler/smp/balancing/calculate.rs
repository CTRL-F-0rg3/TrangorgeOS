#![allow(dead_code)]
use core::sync::atomic::{Ordering};
use core::ptr;
use crate::cpu::scheduler::entities::task::{TaskStruct, CpuMask, MAX_CPUS, CPU_NONE, SchedClass};
use crate::cpu::scheduler::runqueue::RunQueue;
use crate::cpu::scheduler::smp::topology::{SchedDomain, SchedGroup, SD_SHARE_CPUCAPACITY, SD_SHARE_PKG_RESOURCES};
use crate::cpu::scheduler::power::em;

pub const MIGRATION_COST_NS: u64 = 500_000; 
pub const MIN_BALANCE_INTERVAL_NS: u64 = 1_000_000;
pub const IMBALANCE_PCT: u32 = 125; 
pub const GROUP_OVERLOAD_THRESHOLD: u32 = 90; 
pub const MISFIT_TASK_THRESHOLD: u32 = 80; 

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupType {
    FullyIdle,
    HasIdle,
    GroupAsym,
    GroupMisfit,
    GroupOverloaded,
    GroupSmt,
    GroupImbalanced,
}

#[repr(C)]
pub struct LoadCalculation {
    pub src_load: u64,
    pub dst_load: u64,
    pub src_util: u64,
    pub dst_util: u64,
    pub src_capacity: u64,
    pub dst_capacity: u64,
    pub imbalance: u64,
    pub nr_moving: u32,
    pub env_task_util: u32,
}

impl LoadCalculation {
    pub const fn empty() -> Self {
        Self {
            src_load: 0, dst_load: 0,
            src_util: 0, dst_util: 0,
            src_capacity: 0, dst_capacity: 0,
            imbalance: 0, nr_moving: 0, env_task_util: 0,
        }
    }
}

pub unsafe fn update_group_capacity(sd: *mut SchedDomain, sg: *mut SchedGroup, cpu: u32) {
    if sg.is_null() || sd.is_null() { return; }
    
    let mut capacity = 0u32;
    let mut util = 0u32;
    let mut nr_running = 0u32;
    let mut idle_cpus = CpuMask::empty();

    let mask = &(*sg).group_mask;
    for c in mask.iter() {
        let rq_ptr = crate::cpu::scheduler::runqueue::get_rq(c);
        if rq_ptr.is_null() { continue; }
        let rq = &*rq_ptr;
        
        let raw_cap = em::em_get_pd_for_cpu(c).map(|pd| pd.max_capacity).unwrap_or(1024);
        let thermal_pressure = em::em_get_pd_for_cpu(c).map(|pd| pd.thermal_pressure.load(Ordering::Relaxed)).unwrap_or(0);
        let eff_cap = raw_cap.saturating_sub(thermal_pressure);
        
        capacity = capacity.saturating_add(eff_cap);
        util = util.saturating_add(rq.fair.load_weight as u32); 
        nr_running = nr_running.saturating_add(rq.nr_running());
        
        if rq.is_idle() {
            idle_cpus.set(c);
        }
    }

    (*sg).group_capacity = capacity;
    (*sg).group_util = util;
    (*sg).idle_cpus = idle_cpus;
    (*sg).group_weight = mask.count();
    
    if nr_running > 0 && util > (capacity * GROUP_OVERLOAD_THRESHOLD / 100) {
        (*sg).group_overloaded = true;
    } else {
        (*sg).group_overloaded = false;
    }
}

pub unsafe fn check_misfit_task(sg: *mut SchedGroup, rq: &RunQueue) -> bool {
    let curr = rq.current();
    if curr.is_null() { return false; }
    
    let task_util = (*curr).se.util_avg as u32;
    let cpu_cap = (*sg).group_capacity / (*sg).group_weight.max(1);
    
    if task_util > (cpu_cap * MISFIT_TASK_THRESHOLD / 100) {
        let em_ref = &em::GLOBAL_ENERGY_MODEL;
        let is_big = em_ref.is_cpu_big(rq.cpu);
        
        if !is_big && task_util > cpu_cap {
            return true;
        }
    }
    false
}

pub unsafe fn calculate_imbalance(env: &mut LoadCalculation, busiest: *mut SchedGroup, local: *mut SchedGroup) {
    let busiest_cap = (*busiest).group_capacity;
    let local_cap = (*local).group_capacity;
    
    let busiest_util = (*busiest).group_util;
    let local_util = (*local).group_util;
    
    let total_util = busiest_util.saturating_add(local_util);
    let total_cap = busiest_cap.saturating_add(local_cap);
    
    let target_util_per_cpu = if total_cap > 0 { (total_util * 1024) / total_cap } else { 0 };
    
    let busiest_target = (busiest_cap * target_util_per_cpu) / 1024;
    let local_target = (local_cap * target_util_per_cpu) / 1024;
    
    let busiest_excess = busiest_util.saturating_sub(busiest_target);
    let local_deficit = local_target.saturating_sub(local_util);
    
    let imbalance = busiest_excess.min(local_deficit);
    
    env.imbalance = imbalance as u64;
    env.src_util = busiest_util as u64;
    env.dst_util = local_util as u64;
    env.src_capacity = busiest_cap as u64;
    env.dst_capacity = local_cap as u64;
}

pub unsafe fn classify_group(sg: *mut SchedGroup) -> GroupType {
    if sg.is_null() { return GroupType::FullyIdle; }
    
    if (*sg).group_util == 0 {
        return GroupType::FullyIdle;
    }
    
    if !(*sg).idle_cpus.is_empty() {
        return GroupType::HasIdle;
    }
    
    if (*sg).group_misfit_task {
        return GroupType::GroupMisfit;
    }
    
    if (*sg).group_overloaded {
        return GroupType::GroupOverloaded;
    }
    
    GroupType::GroupAsym
}

/// Zwraca (najbardziej obciążoną, lokalną) grupę w domenie `sd`, widzianą
/// z `local_cpu`. Wcześniej ta funkcja odpytywała `get_rq(local_cpu)` w
/// KAŻDEJ iteracji pętli tylko po to, żeby ewentualnie przerwać całą
/// funkcję (`local_rq` i tak nigdy nie było dalej używane) — sprawdzamy
/// to raz, przed pętlą, i faktycznie z tego korzystamy.
pub unsafe fn find_busiest_group(sd: *mut SchedDomain, local_cpu: u32, calc: &mut LoadCalculation) -> (*mut SchedGroup, *mut SchedGroup) {
    let rq_ptr = crate::cpu::scheduler::runqueue::get_rq(local_cpu);
    if rq_ptr.is_null() {
        return (ptr::null_mut(), ptr::null_mut());
    }

    let mut busiest: *mut SchedGroup = ptr::null_mut();
    let mut local: *mut SchedGroup = ptr::null_mut();
    let mut max_load = 0u64;

    let mut sg = (*sd).groups;
    while !sg.is_null() {
        update_group_capacity(sd, sg, local_cpu);

        if (*sg).group_mask.is_set(local_cpu) {
            local = sg;
        } else {
            let load = (*sg).group_util as u64;
            if load > max_load {
                max_load = load;
                busiest = sg;
            }
        }

        sg = (*sg).next;
    }

    if !busiest.is_null() && !local.is_null() {
        calculate_imbalance(calc, busiest, local);
    }

    (busiest, local)
}

/// Czy `local_cpu` jest tym, który powinien PRZEPROWADZIĆ balansowanie dla
/// domeny `sd` w tej chwili. Bezczynny CPU zawsze balansuje (to jest
/// `idle_balance`, tanie i pożądane). Zajęty CPU balansuje tylko jeśli:
///   (a) minął minimalny odstęp od ostatniego balansowania tej domeny, i
///   (b) jest NAJNIŻSZYM numerem CPU spośród online w zasięgu domeny —
///       dokładnie jeden CPU na domenę powinien ponosić ten koszt, żeby
///       wiele CPU jednocześnie nie próbowało balansować tej samej domeny.
/// Wcześniej warunek (b) był zastąpiony arbitralnym `local_cpu % 4 == 0`,
/// co nie miało związku z faktycznym zasięgiem domeny (np. domena SMT
/// obejmująca CPU 5 i 6 nigdy nie zbalansowałaby się, bo żaden z nich nie
/// dzieli się przez 4).
pub unsafe fn should_we_balance(sd: *mut SchedDomain, local_cpu: u32) -> bool {
    let idle = crate::cpu::scheduler::runqueue::get_rq(local_cpu).map(|rq| unsafe { (*rq).is_idle() }).unwrap_or(true);
    
    if idle {
        return true;
    }
    
    let last_balance = (*sd).last_balance.load(Ordering::Acquire);
    let now = crate::cpu::scheduler::runqueue::get_rq(local_cpu).map(|rq| unsafe { (*rq).clock.load(Ordering::Relaxed) }).unwrap_or(0);
    
    let interval = if (*sd).has_flag(SD_SHARE_CPUCAPACITY) {
        MIN_BALANCE_INTERVAL_NS / 2
    } else {
        MIN_BALANCE_INTERVAL_NS
    };
    
    if now.saturating_sub(last_balance) < interval {
        return false;
    }

    match (*sd).span.iter().find(|&c| crate::cpu::scheduler::cpumask::is_online(c)) {
        Some(designated) => designated == local_cpu,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_calculation_defaults() {
        let calc = LoadCalculation::empty();
        assert_eq!(calc.imbalance, 0);
        assert_eq!(calc.src_util, 0);
    }

    #[test]
    fn group_classification_idle() {
        let mut sg = SchedGroup::empty();
        sg.group_util = 0;
        unsafe {
            assert_eq!(classify_group(&mut sg), GroupType::FullyIdle);
        }
    }

    #[test]
    fn group_classification_has_idle_beats_overloaded() {
        let mut sg = SchedGroup::empty();
        sg.group_util = 500;
        sg.idle_cpus.set(3);
        sg.group_overloaded = true;
        unsafe {
            assert_eq!(classify_group(&mut sg), GroupType::HasIdle);
        }
    }

    #[test]
    fn group_classification_overloaded_when_busy_and_full() {
        let mut sg = SchedGroup::empty();
        sg.group_util = 900;
        sg.group_overloaded = true;
        unsafe {
            assert_eq!(classify_group(&mut sg), GroupType::GroupOverloaded);
        }
    }

    #[test]
    fn calculate_imbalance_moves_load_toward_equilibrium() {
        let mut busiest = SchedGroup::empty();
        busiest.group_capacity = 1024;
        busiest.group_util = 900;

        let mut local = SchedGroup::empty();
        local.group_capacity = 1024;
        local.group_util = 100;

        let mut calc = LoadCalculation::empty();
        unsafe {
            calculate_imbalance(&mut calc, &mut busiest, &mut local);
        }

        // Równe pojemności, więc docelowo obie strony powinny dążyć do
        // ~500 każda; nierównowaga to mniejsza z (nadwyżka busiest,
        // deficyt local) = 400.
        assert_eq!(calc.imbalance, 400);
        assert_eq!(calc.src_util, 900);
        assert_eq!(calc.dst_util, 100);
    }

    #[test]
    fn calculate_imbalance_is_zero_for_balanced_groups() {
        let mut busiest = SchedGroup::empty();
        busiest.group_capacity = 1024;
        busiest.group_util = 500;

        let mut local = SchedGroup::empty();
        local.group_capacity = 1024;
        local.group_util = 500;

        let mut calc = LoadCalculation::empty();
        unsafe {
            calculate_imbalance(&mut calc, &mut busiest, &mut local);
        }
        assert_eq!(calc.imbalance, 0);
    }
}