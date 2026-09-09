use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use core::ptr;
use crate::cpu::scheduler::entities::task::{CpuMask, TaskStruct, MAX_CPUS};
use crate::cpu::scheduler::runqueue::RunQueue;

pub const MAX_OPP_COUNT: usize = 16;
pub const MAX_PD_COUNT: usize = 8;
pub const EM_MAX_POWER: u32 = 100_000_000;
pub const EM_MIN_POWER: u32 = 100;
pub const EM_FREQ_SCALE: u64 = 1_000_000;
pub const EM_POWER_SCALE: u64 = 1_000;
pub const EM_CAPACITY_SCALE: u64 = 1024;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapacityState {
    pub frequency: u32,
    pub power: u32,
    pub cost: u32,
    pub performance: u32,
    pub capacity: u32,
    pub voltage: u32,
    pub transition_latency_ns: u32,
    pub flags: u32,
}

impl CapacityState {
    pub const fn empty() -> Self {
        Self {
            frequency: 0,
            power: 0,
            cost: 0,
            performance: 0,
            capacity: 0,
            voltage: 0,
            transition_latency_ns: 0,
            flags: 0,
        }
    }

    pub const fn new(freq: u32, power: u32, cap: u32, volt: u32, lat: u32) -> Self {
        Self {
            frequency: freq,
            power: power,
            cost: 0,
            performance: cap,
            capacity: cap,
            voltage: volt,
            transition_latency_ns: lat,
            flags: 0,
        }
    }

    pub fn compute_cost(&mut self, max_cap: u32) {
        if self.capacity == 0 {
            self.cost = u32::MAX;
            return;
        }
        let inverted_cap = max_cap - self.capacity;
        let power_cost = (self.power as u64 * (inverted_cap as u64 + 1)) / EM_CAPACITY_SCALE;
        self.cost = power_cost.min(u32::MAX as u64) as u32;
    }

    pub fn is_valid(&self) -> bool {
        self.frequency > 0 && self.capacity > 0 && self.power > 0
    }
}

#[repr(C)]
pub struct PerformanceDomain {
    pub states: [CapacityState; MAX_OPP_COUNT],
    pub nr_states: u32,
    pub cpus: CpuMask,
    pub td_lock: crate::cpu::scheduler::entities::task::SpinLock,
    pub max_capacity: u32,
    pub min_capacity: u32,
    pub flags: u32,
    pub id: u32,
    pub thermal_pressure: AtomicU32,
    pub effective_capacity: AtomicU32,
    pub active_state_idx: AtomicU32,
    pub power_multiplier: u32,
    pub leakage_coefficient: u32,
    pub dynamic_coefficient: u32,
    pub thermal_resistance: u32,
    pub thermal_capacitance: u32,
    pub ambient_temp_celsius: u32,
    pub current_temp_millideg: AtomicU32,
    pub max_temp_millideg: u32,
    pub throttle_temp_millideg: u32,
}

impl PerformanceDomain {
    pub const fn empty() -> Self {
        const EMPTY_STATE: CapacityState = CapacityState::empty();
        Self {
            states: [EMPTY_STATE; MAX_OPP_COUNT],
            nr_states: 0,
            cpus: CpuMask::empty(),
            td_lock: crate::cpu::scheduler::entities::task::SpinLock::new(),
            max_capacity: 0,
            min_capacity: 0,
            flags: 0,
            id: 0,
            thermal_pressure: AtomicU32::new(0),
            effective_capacity: AtomicU32::new(0),
            active_state_idx: AtomicU32::new(0),
            power_multiplier: 1024,
            leakage_coefficient: 100,
            dynamic_coefficient: 500,
            thermal_resistance: 10,
            thermal_capacitance: 100,
            ambient_temp_celsius: 25,
            current_temp_millideg: AtomicU32::new(25000),
            max_temp_millideg: 105000,
            throttle_temp_millideg: 85000,
        }
    }

    pub fn init(&mut self, id: u32, cpus: CpuMask) {
        self.id = id;
        self.cpus = cpus;
        self.nr_states = 0;
        self.max_capacity = 0;
        self.min_capacity = u32::MAX;
        self.thermal_pressure.store(0, Ordering::Relaxed);
        self.effective_capacity.store(0, Ordering::Relaxed);
        self.active_state_idx.store(0, Ordering::Relaxed);
        self.current_temp_millideg.store(self.ambient_temp_celsius * 1000, Ordering::Relaxed);
    }

    pub fn add_state(&mut self, state: CapacityState) -> bool {
        if self.nr_states as usize >= MAX_OPP_COUNT {
            return false;
        }
        if !state.is_valid() {
            return false;
        }
        let idx = self.nr_states as usize;
        self.states[idx] = state;
        self.nr_states += 1;
        if state.capacity > self.max_capacity {
            self.max_capacity = state.capacity;
        }
        if state.capacity < self.min_capacity {
            self.min_capacity = state.capacity;
        }
        true
    }

    pub fn finalize(&mut self) {
        let max_cap = self.max_capacity;
        for i in 0..self.nr_states as usize {
            self.states[i].compute_cost(max_cap);
        }
        self.sort_states();
        self.effective_capacity.store(self.max_capacity, Ordering::Release);
    }

    fn sort_states(&mut self) {
        let n = self.nr_states as usize;
        for i in 0..n {
            for j in (i + 1)..n {
                if self.states[j].capacity < self.states[i].capacity {
                    let tmp = self.states[i];
                    self.states[i] = self.states[j];
                    self.states[j] = tmp;
                }
            }
        }
    }

    pub fn find_state_for_capacity(&self, target_cap: u32) -> Option<&CapacityState> {
        for i in 0..self.nr_states as usize {
            if self.states[i].capacity >= target_cap {
                return Some(&self.states[i]);
            }
        }
        if self.nr_states > 0 {
            Some(&self.states[self.nr_states as usize - 1])
        } else {
            None
        }
    }

    pub fn find_state_for_frequency(&self, target_freq: u32) -> Option<&CapacityState> {
        let mut best: Option<&CapacityState> = None;
        let mut best_diff = u32::MAX;
        for i in 0..self.nr_states as usize {
            let diff = if self.states[i].frequency > target_freq {
                self.states[i].frequency - target_freq
            } else {
                target_freq - self.states[i].frequency
            };
            if diff < best_diff {
                best_diff = diff;
                best = Some(&self.states[i]);
            }
        }
        best
    }

    pub fn get_effective_capacity(&self) -> u32 {
        self.effective_capacity.load(Ordering::Acquire)
    }

    pub fn update_thermal_pressure(&self, pressure: u32) {
        self.thermal_pressure.store(pressure, Ordering::Relaxed);
        let max_cap = self.max_capacity;
        let effective = if pressure >= max_cap {
            self.min_capacity
        } else {
            max_cap - pressure
        };
        self.effective_capacity.store(effective, Ordering::Release);
    }

    pub fn compute_power_at_state(&self, state_idx: usize, temp_millideg: u32) -> u32 {
        if state_idx >= self.nr_states as usize {
            return 0;
        }
        let state = &self.states[state_idx];
        let freq_mhz = state.frequency as u64;
        let volt_mv = state.voltage as u64;
        let dyn_power = (self.dynamic_coefficient as u64 * freq_mhz * volt_mv * volt_mv) / (EM_POWER_SCALE * EM_POWER_SCALE);
        let temp_celsius = temp_millideg / 1000;
        let temp_factor = if temp_celsius > 25 {
            1024 + ((temp_celsius - 25) * 10)
        } else {
            1024
        };
        let static_power = (self.leakage_coefficient as u64 * temp_factor as u64) / 1024;
        let total = (dyn_power + static_power) * self.power_multiplier as u64 / 1024;
        total.min(EM_MAX_POWER as u64) as u32
    }

    pub fn update_temperature(&self, power_mw: u32, delta_time_ms: u32) {
        let current_temp = self.current_temp_millideg.load(Ordering::Relaxed);
        let ambient = self.ambient_temp_celsius * 1000;
        let thermal_r = self.thermal_resistance as u64;
        let thermal_c = self.thermal_capacitance as u64;
        let dt = delta_time_ms as u64;
        let power = power_mw as u64;
        let temp_diff = current_temp as i64 - ambient as i64;
        let heat_flow = (temp_diff * 1000) / (thermal_r.max(1) as i64);
        let net_heat = ((power as i64) * (dt as i64)) - heat_flow;
        let temp_delta = (net_heat * 1000) / (thermal_c.max(1) as i64);
        let new_temp = (current_temp as i64 + temp_delta).max(ambient as i64).min(self.max_temp_millideg as i64);
        self.current_temp_millideg.store(new_temp as u32, Ordering::Relaxed);
    }

    pub fn contains_cpu(&self, cpu: u32) -> bool {
        self.cpus.is_set(cpu)
    }

    pub fn get_active_state_idx(&self) -> u32 {
        self.active_state_idx.load(Ordering::Acquire)
    }

    pub fn set_active_state_idx(&self, idx: u32) {
        if idx < self.nr_states {
            self.active_state_idx.store(idx, Ordering::Release);
        }
    }
}

#[repr(C)]
pub struct EnergyModel {
    pub domains: [PerformanceDomain; MAX_PD_COUNT],
    pub nr_domains: u32,
    pub cpu_to_pd: [u32; MAX_CPUS],
    pub lock: crate::cpu::scheduler::entities::task::SpinLock,
    pub flags: u32,
    pub origin: u32,
    pub global_max_capacity: u32,
    pub global_min_capacity: u32,
    pub total_system_power_mw: AtomicU32,
    pub last_update_time_ns: AtomicU64,
}

impl EnergyModel {
    pub const fn empty() -> Self {
        const EMPTY_PD: PerformanceDomain = PerformanceDomain::empty();
        Self {
            domains: [EMPTY_PD; MAX_PD_COUNT],
            nr_domains: 0,
            cpu_to_pd: [u32::MAX; MAX_CPUS],
            lock: crate::cpu::scheduler::entities::task::SpinLock::new(),
            flags: 0,
            origin: 0,
            global_max_capacity: 0,
            global_min_capacity: u32::MAX,
            total_system_power_mw: AtomicU32::new(0),
            last_update_time_ns: AtomicU64::new(0),
        }
    }

    pub fn init(&mut self) {
        self.nr_domains = 0;
        self.global_max_capacity = 0;
        self.global_min_capacity = u32::MAX;
        for i in 0..MAX_CPUS {
            self.cpu_to_pd[i] = u32::MAX;
        }
    }

    pub fn register_domain(&mut self, pd: PerformanceDomain) -> bool {
        if self.nr_domains as usize >= MAX_PD_COUNT {
            return false;
        }
        let idx = self.nr_domains as usize;
        self.domains[idx] = pd;
        self.domains[idx].id = idx as u32;
        for cpu in pd.cpus.iter() {
            self.cpu_to_pd[cpu as usize] = idx as u32;
        }
        if pd.max_capacity > self.global_max_capacity {
            self.global_max_capacity = pd.max_capacity;
        }
        if pd.min_capacity < self.global_min_capacity {
            self.global_min_capacity = pd.min_capacity;
        }
        self.nr_domains += 1;
        true
    }

    pub fn get_pd_for_cpu(&self, cpu: u32) -> Option<&PerformanceDomain> {
        if cpu as usize >= MAX_CPUS {
            return None;
        }
        let pd_idx = self.cpu_to_pd[cpu as usize];
        if pd_idx == u32::MAX || pd_idx >= self.nr_domains {
            return None;
        }
        Some(&self.domains[pd_idx as usize])
    }

    pub fn get_pd_for_cpu_mut(&mut self, cpu: u32) -> Option<&mut PerformanceDomain> {
        if cpu as usize >= MAX_CPUS {
            return None;
        }
        let pd_idx = self.cpu_to_pd[cpu as usize];
        if pd_idx == u32::MAX || pd_idx >= self.nr_domains {
            return None;
        }
        Some(&mut self.domains[pd_idx as usize])
    }

    pub fn compute_system_energy(&self, delta_time_ms: u32) -> u32 {
        let mut total_power = 0u32;
        for i in 0..self.nr_domains as usize {
            let pd = &self.domains[i];
            let active_idx = pd.get_active_state_idx() as usize;
            let temp = pd.current_temp_millideg.load(Ordering::Relaxed);
            let power = pd.compute_power_at_state(active_idx, temp);
            total_power = total_power.saturating_add(power);
            pd.update_temperature(power, delta_time_ms);
        }
        self.total_system_power_mw.store(total_power, Ordering::Relaxed);
        total_power
    }

    pub fn compute_task_energy_on_cpu(&self, task_util: u32, cpu: u32) -> u32 {
        let pd = match self.get_pd_for_cpu(cpu) {
            Some(p) => p,
            None => return u32::MAX,
        };
        let target_cap = (task_util as u64 * pd.max_capacity as u64 / 1024) as u32;
        let state = match pd.find_state_for_capacity(target_cap) {
            Some(s) => s,
            None => return u32::MAX,
        };
        let temp = pd.current_temp_millideg.load(Ordering::Relaxed);
        let power = pd.compute_power_at_state(pd.states.iter().position(|s| s.capacity == state.capacity).unwrap_or(0), temp);
        let energy = (power as u64 * task_util as u64) / 1024;
        energy.min(u32::MAX as u64) as u32
    }

    pub fn find_best_state_for_domain(&self, pd_idx: u32, target_util: u32) -> Option<usize> {
        if pd_idx >= self.nr_domains {
            return None;
        }
        let pd = &self.domains[pd_idx as usize];
        let eff_cap = pd.get_effective_capacity();
        let target_cap = (target_util as u64 * eff_cap as u64 / 1024) as u32;
        let mut best_idx = None;
        let mut best_cost = u32::MAX;
        for i in 0..pd.nr_states as usize {
            if pd.states[i].capacity >= target_cap {
                if pd.states[i].cost < best_cost {
                    best_cost = pd.states[i].cost;
                    best_idx = Some(i);
                }
            }
        }
        if best_idx.is_none() && pd.nr_states > 0 {
            best_idx = Some(pd.nr_states as usize - 1);
        }
        best_idx
    }

    pub fn invert_capacity_for_thermal(&mut self, cpu: u32, thermal_pressure: u32) {
        if let Some(pd) = self.get_pd_for_cpu_mut(cpu) {
            pd.update_thermal_pressure(thermal_pressure);
        }
    }

    pub fn clear_thermal_pressure(&mut self) {
        for i in 0..self.nr_domains as usize {
            self.domains[i].update_thermal_pressure(0);
        }
    }

    pub fn get_max_capacity(&self) -> u32 {
        self.global_max_capacity
    }

    pub fn get_min_capacity(&self) -> u32 {
        self.global_min_capacity
    }

    pub fn is_cpu_big(&self, cpu: u32) -> bool {
        if let Some(pd) = self.get_pd_for_cpu(cpu) {
            pd.max_capacity > (self.global_max_capacity / 2)
        } else {
            false
        }
    }

    pub fn is_cpu_little(&self, cpu: u32) -> bool {
        if let Some(pd) = self.get_pd_for_cpu(cpu) {
            pd.max_capacity <= (self.global_max_capacity / 2)
        } else {
            false
        }
    }

    pub fn dump_state(&self) {
        let _ = self.nr_domains;
        let _ = self.global_max_capacity;
        let _ = self.total_system_power_mw.load(Ordering::Relaxed);
    }
}

pub static mut GLOBAL_ENERGY_MODEL: EnergyModel = EnergyModel::empty();

pub unsafe fn em_init() {
    GLOBAL_ENERGY_MODEL.init();
}

pub unsafe fn em_register_pd(pd: PerformanceDomain) -> bool {
    let _guard = GLOBAL_ENERGY_MODEL.lock.lock_irqsave_guard();
    GLOBAL_ENERGY_MODEL.register_domain(pd)
}

pub unsafe fn em_get_pd_for_cpu(cpu: u32) -> Option<&'static PerformanceDomain> {
    GLOBAL_ENERGY_MODEL.get_pd_for_cpu(cpu)
}

pub unsafe fn em_compute_task_energy(task_util: u32, cpu: u32) -> u32 {
    GLOBAL_ENERGY_MODEL.compute_task_energy_on_cpu(task_util, cpu)
}

pub unsafe fn em_invert_capacity(cpu: u32, pressure: u32) {
    let _guard = GLOBAL_ENERGY_MODEL.lock.lock_irqsave_guard();
    GLOBAL_ENERGY_MODEL.invert_capacity_for_thermal(cpu, pressure);
}

pub unsafe fn em_clear_thermal_pressure() {
    let _guard = GLOBAL_ENERGY_MODEL.lock.lock_irqsave_guard();
    GLOBAL_ENERGY_MODEL.clear_thermal_pressure();
}

pub unsafe fn em_compute_system_power(delta_ms: u32) -> u32 {
    GLOBAL_ENERGY_MODEL.compute_system_energy(delta_ms)
}

pub unsafe fn em_find_best_state(pd_idx: u32, util: u32) -> Option<usize> {
    GLOBAL_ENERGY_MODEL.find_best_state_for_domain(pd_idx, util)
}

pub fn em_build_synthetic_little_pd(cpus: CpuMask) -> PerformanceDomain {
    let mut pd = PerformanceDomain::empty();
    pd.init(0, cpus);
    pd.dynamic_coefficient = 200;
    pd.leakage_coefficient = 50;
    pd.thermal_resistance = 15;
    pd.thermal_capacitance = 120;
    pd.add_state(CapacityState::new(500, 100, 150, 800, 10));
    pd.add_state(CapacityState::new(700, 150, 250, 850, 10));
    pd.add_state(CapacityState::new(1000, 250, 382, 900, 10));
    pd.add_state(CapacityState::new(1200, 350, 480, 950, 10));
    pd.add_state(CapacityState::new(1500, 500, 600, 1000, 10));
    pd.add_state(CapacityState::new(1800, 700, 750, 1050, 10));
    pd.add_state(CapacityState::new(2000, 900, 870, 1100, 10));
    pd.finalize();
    pd
}

pub fn em_build_synthetic_big_pd(cpus: CpuMask) -> PerformanceDomain {
    let mut pd = PerformanceDomain::empty();
    pd.init(1, cpus);
    pd.dynamic_coefficient = 800;
    pd.leakage_coefficient = 200;
    pd.thermal_resistance = 8;
    pd.thermal_capacitance = 80;
    pd.add_state(CapacityState::new(600, 300, 400, 850, 20));
    pd.add_state(CapacityState::new(900, 500, 650, 900, 20));
    pd.add_state(CapacityState::new(1200, 800, 900, 950, 20));
    pd.add_state(CapacityState::new(1500, 1200, 1100, 1000, 20));
    pd.add_state(CapacityState::new(1800, 1800, 1400, 1050, 20));
    pd.add_state(CapacityState::new(2100, 2500, 1700, 1100, 20));
    pd.add_state(CapacityState::new(2400, 3500, 2000, 1150, 20));
    pd.add_state(CapacityState::new(2800, 5000, 2400, 1200, 20));
    pd.finalize();
    pd
}

pub fn em_build_synthetic_huge_pd(cpus: CpuMask) -> PerformanceDomain {
    let mut pd = PerformanceDomain::empty();
    pd.init(2, cpus);
    pd.dynamic_coefficient = 1500;
    pd.leakage_coefficient = 400;
    pd.thermal_resistance = 5;
    pd.thermal_capacitance = 50;
    pd.add_state(CapacityState::new(800, 800, 800, 900, 30));
    pd.add_state(CapacityState::new(1200, 1500, 1300, 950, 30));
    pd.add_state(CapacityState::new(1600, 2500, 1800, 1000, 30));
    pd.add_state(CapacityState::new(2000, 4000, 2400, 1050, 30));
    pd.add_state(CapacityState::new(2400, 6000, 3000, 1100, 30));
    pd.add_state(CapacityState::new(2800, 9000, 3600, 1150, 30));
    pd.add_state(CapacityState::new(3200, 13000, 4200, 1200, 30));
    pd.add_state(CapacityState::new(3600, 18000, 4800, 1250, 30));
    pd.finalize();
    pd
}