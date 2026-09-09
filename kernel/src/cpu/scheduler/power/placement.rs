use core::sync::atomic::{AtomicU32, Ordering};
use crate::cpu::scheduler::entities::task::{CpuMask, TaskStruct, TaskFlags, SchedClass, MAX_CPUS, CPU_NONE};
use crate::cpu::scheduler::runqueue::RunQueue;
use crate::cpu::scheduler::power::em::{GLOBAL_ENERGY_MODEL, PerformanceDomain};

pub const EAS_MARGIN_PERCENT: u32 = 20;
pub const EAS_MAX_IDLE_CPU_PERCENT: u32 = 15;
pub const EAS_PACKING_THRESHOLD: u32 = 80;
pub const EAS_MIGRATION_COST_NS: u64 = 500_000;
pub const EAS_CACHE_AFFINITY_BONUS: u32 = 50;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuSnapshot {
    pub cpu: u32,
    pub pd_idx: u32,
    pub eff_capacity: u32,
    pub max_capacity: u32,
    pub util: u32,
    pub nr_running: u32,
    pub is_big: bool,
    pub is_little: bool,
    pub shares_cache_with_prev: bool,
    pub thermal_pressure: u32,
}

impl CpuSnapshot {
    pub const fn empty() -> Self {
        Self {
            cpu: CPU_NONE,
            pd_idx: u32::MAX,
            eff_capacity: 0,
            max_capacity: 0,
            util: 0,
            nr_running: 0,
            is_big: false,
            is_little: false,
            shares_cache_with_prev: false,
            thermal_pressure: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DomainSnapshot {
    pub pd_idx: u32,
    pub cpus: [CpuSnapshot; MAX_CPUS],
    pub nr_cpus: u32,
    pub total_util: u32,
    pub max_eff_capacity: u32,
    pub min_energy: u32,
    pub best_cpu: u32,
    pub is_overutilized: bool,
}

impl DomainSnapshot {
    pub const fn empty() -> Self {
        const EMPTY_CPU: CpuSnapshot = CpuSnapshot::empty();
        Self {
            pd_idx: u32::MAX,
            cpus: [EMPTY_CPU; MAX_CPUS],
            nr_cpus: 0,
            total_util: 0,
            max_eff_capacity: 0,
            min_energy: u32::MAX,
            best_cpu: CPU_NONE,
            is_overutilized: false,
        }
    }
}

#[repr(C)]
pub struct EnergyCtx {
    pub task: *mut TaskStruct,
    pub task_util: u32,
    pub task_util_est: u32,
    pub target_cpu: u32,
    pub prev_cpu: u32,
    pub env: [DomainSnapshot; 8],
    pub nr_domains: u32,
    pub skip_greedy: bool,
    pub sync_wake: bool,
    pub uclamp_min: u32,
    pub uclamp_max: u32,
}

impl EnergyCtx {
    pub const fn empty() -> Self {
        const EMPTY_DOM: DomainSnapshot = DomainSnapshot::empty();
        Self {
            task: core::ptr::null_mut(),
            task_util: 0,
            task_util_est: 0,
            target_cpu: CPU_NONE,
            prev_cpu: CPU_NONE,
            env: [EMPTY_DOM; 8],
            nr_domains: 0,
            skip_greedy: false,
            sync_wake: false,
            uclamp_min: 0,
            uclamp_max: 1024,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PlacementResult {
    pub best_cpu: u32,
    pub best_energy: u32,
    pub best_pd_idx: u32,
    pub fallback_cpu: u32,
    pub reason: u32,
}

impl PlacementResult {
    pub const fn empty() -> Self {
        Self {
            best_cpu: CPU_NONE,
            best_energy: u32::MAX,
            best_pd_idx: u32::MAX,
            fallback_cpu: CPU_NONE,
            reason: 0,
        }
    }
}

pub const REASON_ENERGY: u32 = 1;
pub const REASON_CAPACITY: u32 = 2;
pub const REASON_AFFINITY: u32 = 3;
pub const REASON_THERMAL: u32 = 4;
pub const REASON_FALLBACK: u32 = 5;

pub unsafe fn init_energy_ctx(task: *mut TaskStruct, prev_cpu: u32) -> EnergyCtx {
    let mut ctx = EnergyCtx::empty();
    ctx.task = task;
    ctx.prev_cpu = prev_cpu;
    if !task.is_null() {
        ctx.task_util = (*task).se.util_avg as u32;
        ctx.task_util_est = core::cmp::max((*task).se.util_avg as u32, (*task).se.load_avg as u32);
        ctx.uclamp_min = 0;
        ctx.uclamp_max = 1024;
        ctx.target_cpu = prev_cpu;
    }
    ctx
}

pub unsafe fn build_domain_snapshots(ctx: &mut EnergyCtx, rq_registry: &[*mut RunQueue]) {
    let em = &GLOBAL_ENERGY_MODEL;
    ctx.nr_domains = em.nr_domains;
    for i in 0..em.nr_domains as usize {
        let pd = &em.domains[i];
        let dom_snap = &mut ctx.env[i];
        dom_snap.pd_idx = i as u32;
        dom_snap.nr_cpus = 0;
        dom_snap.total_util = 0;
        dom_snap.max_eff_capacity = pd.get_effective_capacity();
        dom_snap.is_overutilized = false;
        dom_snap.min_energy = u32::MAX;
        dom_snap.best_cpu = CPU_NONE;

        for cpu in pd.cpus.iter() {
            if cpu as usize >= MAX_CPUS {
                continue;
            }
            let rq_ptr = match rq_registry.get(cpu as usize) {
                Some(&p) if !p.is_null() => p,
                _ => continue,
            };
            let rq = &*rq_ptr;
            let cpu_snap = &mut dom_snap.cpus[dom_snap.nr_cpus as usize];
            cpu_snap.cpu = cpu;
            cpu_snap.pd_idx = i as u32;
            cpu_snap.eff_capacity = pd.get_effective_capacity();
            cpu_snap.max_capacity = pd.max_capacity;
            cpu_snap.util = rq.nr_running() * 100;
            cpu_snap.nr_running = rq.nr_running();
            cpu_snap.is_big = em.is_cpu_big(cpu);
            cpu_snap.is_little = em.is_cpu_little(cpu);
            cpu_snap.thermal_pressure = pd.thermal_pressure.load(Ordering::Relaxed);
            cpu_snap.shares_cache_with_prev = (cpu == ctx.prev_cpu) || (em.is_cpu_big(cpu) == em.is_cpu_big(ctx.prev_cpu));
            
            dom_snap.total_util = dom_snap.total_util.saturating_add(cpu_snap.util);
            if cpu_snap.util > cpu_snap.eff_capacity {
                dom_snap.is_overutilized = true;
            }
            dom_snap.nr_cpus += 1;
        }
    }
}

pub unsafe fn task_fits_capacity(task_util: u32, cpu_cap: u32) -> bool {
    let required_cap = task_util + (task_util * EAS_MARGIN_PERCENT / 100);
    required_cap <= cpu_cap
}

pub unsafe fn cpu_overutilized(cpu_util: u32, cpu_cap: u32) -> bool {
    cpu_util > (cpu_cap * EAS_PACKING_THRESHOLD / 100)
}

pub unsafe fn cpu_underutilized(cpu_util: u32, cpu_cap: u32) -> bool {
    cpu_util < (cpu_cap * EAS_MAX_IDLE_CPU_PERCENT / 100)
}

pub unsafe fn compute_energy_delta(ctx: &EnergyCtx, dom_snap: &DomainSnapshot, target_cpu: u32, task_util: u32) -> u32 {
    let em = &GLOBAL_ENERGY_MODEL;
    let pd = &em.domains[dom_snap.pd_idx as usize];
    
    let mut base_util = 0u32;
    let mut base_cap = pd.get_effective_capacity();
    let mut new_util = 0u32;
    
    for i in 0..dom_snap.nr_cpus as usize {
        let cpu_snap = &dom_snap.cpus[i];
        base_util = base_util.saturating_add(cpu_snap.util);
        if cpu_snap.cpu == target_cpu {
            new_util = cpu_snap.util.saturating_add(task_util);
        } else {
            new_util = cpu_snap.util;
        }
    }
    
    let base_state = match pd.find_state_for_capacity((base_util as u64 * base_cap as u64 / 1024) as u32) {
        Some(s) => s,
        None => return u32::MAX,
    };
    
    let total_util_with_task = base_util.saturating_add(task_util);
    let new_state = match pd.find_state_for_capacity((total_util_with_task as u64 * base_cap as u64 / 1024) as u32) {
        Some(s) => s,
        None => return u32::MAX,
    };
    
    let base_energy = (base_state.power as u64 * base_util as u64) / 1024;
    let new_energy = (new_state.power as u64 * total_util_with_task as u64) / 1024;
    
    if new_energy < base_energy {
        0
    } else {
        (new_energy - base_energy).min(u32::MAX as u64) as u32
    }
}

pub unsafe fn evaluate_cpu(ctx: &mut EnergyCtx, dom_snap: &mut DomainSnapshot, cpu_idx: usize) {
    let cpu_snap = &dom_snap.cpus[cpu_idx];
    let cpu = cpu_snap.cpu;
    let task_util = ctx.task_util_est;
    
    if !task_fits_capacity(task_util, cpu_snap.eff_capacity) {
        return;
    }
    
    if cpu_overutilized(cpu_snap.util, cpu_snap.eff_capacity) {
        return;
    }
    
    let thermal_penalty = if cpu_snap.thermal_pressure > 500 {
        cpu_snap.thermal_pressure / 10
    } else {
        0
    };
    
    let mut energy = compute_energy_delta(ctx, dom_snap, cpu, task_util);
    energy = energy.saturating_add(thermal_penalty);
    
    if cpu == ctx.prev_cpu {
        if energy > EAS_CACHE_AFFINITY_BONUS {
            energy -= EAS_CACHE_AFFINITY_BONUS;
        } else {
            energy = 0;
        }
    }
    
    if energy < dom_snap.min_energy {
        dom_snap.min_energy = energy;
        dom_snap.best_cpu = cpu;
    }
}

pub unsafe fn evaluate_domain(ctx: &mut EnergyCtx, dom_idx: usize) {
    let is_overutilized = ctx.env[dom_idx].is_overutilized;
    let nr_cpus = ctx.env[dom_idx].nr_cpus;
    
    if is_overutilized {
        return;
    }
    
    for i in 0..nr_cpus as usize {
        evaluate_cpu(ctx, dom_idx, i);
    }
}

pub unsafe fn evaluate_cpu(ctx: &mut EnergyCtx, dom_idx: usize, cpu_idx: usize) {
    let cpu_snap = &ctx.env[dom_idx].cpus[cpu_idx];
    let cpu = cpu_snap.cpu;
    let task_util = ctx.task_util_est;
    
    if !task_fits_capacity(task_util, cpu_snap.eff_capacity) {
        return;
    }
    
    if cpu_overutilized(cpu_snap.util, cpu_snap.eff_capacity) {
        return;
    }
    
    let thermal_penalty = if cpu_snap.thermal_pressure > 500 {
        cpu_snap.thermal_pressure / 10
    } else {
        0
    };
    
    let mut energy = compute_energy_delta(ctx, dom_idx, cpu, task_util);
    energy = energy.saturating_add(thermal_penalty);
    
    if cpu == ctx.prev_cpu {
        if energy > EAS_CACHE_AFFINITY_BONUS {
            energy -= EAS_CACHE_AFFINITY_BONUS;
        } else {
            energy = 0;
        }
    }
    
    if energy < ctx.env[dom_idx].min_energy {
        ctx.env[dom_idx].min_energy = energy;
        ctx.env[dom_idx].best_cpu = cpu;
    }
}

pub unsafe fn compute_energy_delta(ctx: &EnergyCtx, dom_idx: usize, target_cpu: u32, task_util: u32) -> u32 {
    let em = &GLOBAL_ENERGY_MODEL;
    let pd = &em.domains[dom_idx];
    let dom_snap = &ctx.env[dom_idx];
    
pub unsafe fn find_best_domain(ctx: &EnergyCtx) -> (u32, u32) {
    let mut best_pd = u32::MAX;
    let mut min_energy = u32::MAX;
    for i in 0..ctx.nr_domains as usize {
        let dom_snap = &ctx.env[i];
        if dom_snap.best_cpu != CPU_NONE && dom_snap.min_energy < min_energy {
            min_energy = dom_snap.min_energy;
            best_pd = i as u32;
        }
    }
    (best_pd, min_energy)
}

pub unsafe fn find_fallback_cpu(ctx: &EnergyCtx, rq_registry: &[*mut RunQueue]) -> u32 {
    let task_util = ctx.task_util_est;
    let mut best_cpu = CPU_NONE;
    let mut best_cap = u32::MAX;
    
    for i in 0..ctx.nr_domains as usize {
        let dom_snap = &ctx.env[i];
        for j in 0..dom_snap.nr_cpus as usize {
            let cpu_snap = &dom_snap.cpus[j];
            if task_fits_capacity(task_util, cpu_snap.eff_capacity) {
                if cpu_snap.eff_capacity < best_cap {
                    best_cap = cpu_snap.eff_capacity;
                    best_cpu = cpu_snap.cpu;
                }
            }
        }
    }
    
    if best_cpu == CPU_NONE {
        let em = &GLOBAL_ENERGY_MODEL;
        best_cpu = em.global_max_capacity;
    }
    
    best_cpu
}

pub unsafe fn find_energy_efficient_cpu(task: *mut TaskStruct, prev_cpu: u32, rq_registry: &[*mut RunQueue]) -> PlacementResult {
    let mut result = PlacementResult::empty();
    if task.is_null() {
        result.reason = REASON_FALLBACK;
        return result;
    }
    
    let mut ctx = init_energy_ctx(task, prev_cpu);
    build_domain_snapshots(&mut ctx, rq_registry);
    
    for i in 0..ctx.nr_domains as usize {
        evaluate_domain(&mut ctx, i);
    }
    
    let (best_pd, min_energy) = find_best_domain(&ctx);
    
    if best_pd != u32::MAX {
        result.best_pd_idx = best_pd;
        result.best_energy = min_energy;
        result.best_cpu = ctx.env[best_pd as usize].best_cpu;
        result.reason = REASON_ENERGY;
    } else {
        result.best_cpu = find_fallback_cpu(&ctx, rq_registry);
        result.reason = REASON_CAPACITY;
    }
    
    result.fallback_cpu = find_fallback_cpu(&ctx, rq_registry);
    result
}

pub unsafe fn select_target_cpu(task: *mut TaskStruct, prev_cpu: u32, rq_registry: &[*mut RunQueue]) -> u32 {
    let res = find_energy_efficient_cpu(task, prev_cpu, rq_registry);
    if res.best_cpu != CPU_NONE {
        res.best_cpu
    } else if res.fallback_cpu != CPU_NONE {
        res.fallback_cpu
    } else {
        prev_cpu
    }
}

pub unsafe fn check_preempt_eas(curr: *mut TaskStruct, cand: *mut TaskStruct, rq: &RunQueue) -> bool {
    if curr.is_null() || cand.is_null() {
        return false;
    }
    let curr_util = (*curr).se.util_avg as u32;
    let cand_util = (*cand).se.util_avg as u32;
    let cpu = rq.cpu;
    
    let em = &GLOBAL_ENERGY_MODEL;
    let pd = match em.get_pd_for_cpu(cpu) {
        Some(p) => p,
        None => return cand_util < curr_util,
    };
    
    let curr_energy = em.compute_task_energy_on_cpu(curr_util, cpu);
    let cand_energy = em.compute_task_energy_on_cpu(cand_util, cpu);
    
    if cand_energy < curr_energy {
        return true;
    }
    
    if (*cand).se.vruntime < (*curr).se.vruntime {
        return true;
    }
    
    false
}

pub unsafe fn update_cpu_util_for_eas(rq: &mut RunQueue, task: *mut TaskStruct, enqueue: bool) {
    if task.is_null() {
        return;
    }
    let util = (*task).se.util_avg as u32;
    if enqueue {
        rq.nr_running.fetch_add(util, Ordering::Relaxed);
    } else {
        rq.nr_running.fetch_sub(util, Ordering::Relaxed);
    }
}

pub unsafe fn dump_placement_result(res: &PlacementResult) {
    let _ = res.best_cpu;
    let _ = res.best_energy;
    let _ = res.reason;
}