  pub mod em;
  pub mod placement;

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};
use crate::cpu::scheduler::entities::task::{CpuMask, MAX_CPUS, SpinLock};
use crate::cpu::scheduler::power::em::{GLOBAL_ENERGY_MODEL, PerformanceDomain};

pub const THERMAL_NORMAL: u32 = 0;
pub const THERMAL_LIGHT: u32 = 1;
pub const THERMAL_MEDIUM: u32 = 2;
pub const THERMAL_SEVERE: u32 = 3;
pub const THERMAL_CRITICAL: u32 = 4;

pub const TEMP_NORMAL_MAX: u32 = 60000;
pub const TEMP_LIGHT_MAX: u32 = 75000;
pub const TEMP_MEDIUM_MAX: u32 = 85000;
pub const TEMP_SEVERE_MAX: u32 = 95000;
pub const TEMP_CRITICAL_MAX: u32 = 105000;

pub const PID_KP: i64 = 50;
pub const PID_KI: i64 = 10;
pub const PID_KD: i64 = 20;
pub const PID_INTEGRAL_MAX: i64 = 10000;
pub const PID_INTEGRAL_MIN: i64 = -10000;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PidState {
    pub setpoint: i64,
    pub integral: i64,
    pub prev_error: i64,
    pub output: i64,
}

impl PidState {
    pub const fn new(setpoint: i64) -> Self {
        Self {
            setpoint,
            integral: 0,
            prev_error: 0,
            output: 0,
        }
    }

    pub fn update(&mut self, measurement: i64) -> i64 {
        let error = self.setpoint - measurement;
        self.integral = (self.integral + error).clamp(PID_INTEGRAL_MIN, PID_INTEGRAL_MAX);
        let derivative = error - self.prev_error;
        self.prev_error = error;
        self.output = (PID_KP * error + PID_KI * self.integral + PID_KD * derivative) / 100;
        self.output
    }

    pub fn reset(&mut self) {
        self.integral = 0;
        self.prev_error = 0;
        self.output = 0;
    }
}

#[repr(C)]
pub struct ThermalZone {
    pub pd_idx: u32,
    pub pid: PidState,
    pub current_temp: AtomicU32,
    pub current_level: AtomicU32,
    pub trip_points: [u32; 5],
    pub hysteresis: u32,
    pub mitigation_active: AtomicBool,
    pub cooldown_time_ms: AtomicU32,
    pub last_mitigation_time_ms: AtomicU64,
}

impl ThermalZone {
    pub const fn empty() -> Self {
        Self {
            pd_idx: u32::MAX,
            pid: PidState::new(TEMP_LIGHT_MAX as i64),
            current_temp: AtomicU32::new(0),
            current_level: AtomicU32::new(THERMAL_NORMAL),
            trip_points: [TEMP_NORMAL_MAX, TEMP_LIGHT_MAX, TEMP_MEDIUM_MAX, TEMP_SEVERE_MAX, TEMP_CRITICAL_MAX],
            hysteresis: 2000,
            mitigation_active: AtomicBool::new(false),
            cooldown_time_ms: AtomicU32::new(100),
            last_mitigation_time_ms: AtomicU64::new(0),
        }
    }

    pub fn init(&mut self, pd_idx: u32) {
        self.pd_idx = pd_idx;
        self.pid = PidState::new(TEMP_LIGHT_MAX as i64);
        self.current_temp.store(25000, Ordering::Relaxed);
        self.current_level.store(THERMAL_NORMAL, Ordering::Relaxed);
        self.mitigation_active.store(false, Ordering::Relaxed);
        self.cooldown_time_ms.store(100, Ordering::Relaxed);
        self.last_mitigation_time_ms.store(0, Ordering::Relaxed);
    }

    pub fn update_temp(&self, temp_millideg: u32) {
        self.current_temp.store(temp_millideg, Ordering::Relaxed);
    }

    pub fn get_temp(&self) -> u32 {
        self.current_temp.load(Ordering::Acquire)
    }

    pub fn get_level(&self) -> u32 {
        self.current_level.load(Ordering::Acquire)
    }

    pub fn evaluate_level(&self) -> u32 {
        let temp = self.get_temp();
        let current_level = self.get_level();
        let hyst = self.hysteresis;
        
        let new_level = if temp >= self.trip_points[4] {
            THERMAL_CRITICAL
        } else if temp >= self.trip_points[3] {
            THERMAL_SEVERE
        } else if temp >= self.trip_points[2] {
            THERMAL_MEDIUM
        } else if temp >= self.trip_points[1] {
            THERMAL_LIGHT
        } else {
            THERMAL_NORMAL
        };

        if new_level > current_level {
            self.current_level.store(new_level, Ordering::Release);
            return new_level;
        }

        let drop_threshold = if current_level > 0 {
            self.trip_points[current_level as usize - 1] - hyst
        } else {
            0
        };

        if temp < drop_threshold {
            let dropped_level = if current_level > 0 { current_level - 1 } else { 0 };
            self.current_level.store(dropped_level, Ordering::Release);
            return dropped_level;
        }

        current_level
    }

    pub fn compute_pid_output(&mut self) -> i64 {
        let temp = self.get_temp() as i64;
        self.pid.update(temp)
    }
}

#[repr(C)]
pub struct PowerScheduler {
    pub zones: [ThermalZone; 8],
    pub nr_zones: u32,
    pub lock: SpinLock,
    pub eas_enabled: AtomicBool,
    pub thermal_mitigation_enabled: AtomicBool,
    pub global_thermal_level: AtomicU32,
    pub tick_count: AtomicU64,
    pub last_tick_time_ns: AtomicU64,
    pub emergency_shutdown_triggered: AtomicBool,
    pub max_allowed_capacity: [AtomicU32; MAX_CPUS],
}

impl PowerScheduler {
    pub const fn empty() -> Self {
        const EMPTY_ZONE: ThermalZone = ThermalZone::empty();
        const ZERO_CAP: AtomicU32 = AtomicU32::new(0);
        Self {
            zones: [EMPTY_ZONE; 8],
            nr_zones: 0,
            lock: SpinLock::new(),
            eas_enabled: AtomicBool::new(true),
            thermal_mitigation_enabled: AtomicBool::new(true),
            global_thermal_level: AtomicU32::new(THERMAL_NORMAL),
            tick_count: AtomicU64::new(0),
            last_tick_time_ns: AtomicU64::new(0),
            emergency_shutdown_triggered: AtomicBool::new(false),
            max_allowed_capacity: [ZERO_CAP; MAX_CPUS],
        }
    }

    pub fn init(&mut self) {
        self.nr_zones = 0;
        self.eas_enabled.store(true, Ordering::Relaxed);
        self.thermal_mitigation_enabled.store(true, Ordering::Relaxed);
        self.global_thermal_level.store(THERMAL_NORMAL, Ordering::Relaxed);
        self.tick_count.store(0, Ordering::Relaxed);
        self.last_tick_time_ns.store(0, Ordering::Relaxed);
        self.emergency_shutdown_triggered.store(false, Ordering::Relaxed);
        for i in 0..MAX_CPUS {
            self.max_allowed_capacity[i].store(u32::MAX, Ordering::Relaxed);
        }
    }

    pub fn register_zone(&mut self, pd_idx: u32) -> bool {
        if self.nr_zones as usize >= 8 {
            return false;
        }
        let idx = self.nr_zones as usize;
        self.zones[idx].init(pd_idx);
        self.nr_zones += 1;
        true
    }

    pub fn get_zone_for_pd(&self, pd_idx: u32) -> Option<&ThermalZone> {
        for i in 0..self.nr_zones as usize {
            if self.zones[i].pd_idx == pd_idx {
                return Some(&self.zones[i]);
            }
        }
        None
    }

    pub fn get_zone_for_pd_mut(&mut self, pd_idx: u32) -> Option<&mut ThermalZone> {
        for i in 0..self.nr_zones as usize {
            if self.zones[i].pd_idx == pd_idx {
                return Some(&mut self.zones[i]);
            }
        }
        None
    }

    pub fn update_tick(&mut self, current_time_ns: u64) {
        let last_time = self.last_tick_time_ns.load(Ordering::Relaxed);
        let delta_ns = current_time_ns.saturating_sub(last_time);
        self.last_tick_time_ns.store(current_time_ns, Ordering::Relaxed);
        self.tick_count.fetch_add(1, Ordering::Relaxed);

        if delta_ns < 10_000_000 {
            return;
        }

        let delta_ms = (delta_ns / 1_000_000) as u32;
        let _total_power = unsafe { GLOBAL_ENERGY_MODEL.compute_system_energy(delta_ms) };

        let mut max_level = THERMAL_NORMAL;
        for i in 0..self.nr_zones as usize {
            let zone = &mut self.zones[i];
            let pd = unsafe { &GLOBAL_ENERGY_MODEL.domains[zone.pd_idx as usize] };
            let temp = pd.current_temp_millideg.load(Ordering::Relaxed);
            zone.update_temp(temp);
            let level = zone.evaluate_level();
            if level > max_level {
                max_level = level;
            }
        }
        self.global_thermal_level.store(max_level, Ordering::Release);

        if max_level == THERMAL_CRITICAL {
            self.emergency_shutdown_triggered.store(true, Ordering::Relaxed);
        }
    }

    pub fn apply_mitigation(&mut self) {
        if !self.thermal_mitigation_enabled.load(Ordering::Relaxed) {
            return;
        }

        for i in 0..self.nr_zones as usize {
            let zone = &mut self.zones[i];
            let level = zone.get_level();
            let pd_idx = zone.pd_idx;
            
            let pd = unsafe { &GLOBAL_ENERGY_MODEL.domains[pd_idx as usize] };
            let max_cap = pd.max_capacity;
            let mut pressure = 0u32;

            match level {
                THERMAL_NORMAL => {
                    pressure = 0;
                    zone.mitigation_active.store(false, Ordering::Relaxed);
                }
                THERMAL_LIGHT => {
                    let pid_out = zone.compute_pid_output();
                    pressure = (max_cap as i64 * pid_out.abs() / 1000).min(max_cap as i64) as u32;
                    zone.mitigation_active.store(true, Ordering::Relaxed);
                }
                THERMAL_MEDIUM => {
                    pressure = max_cap / 4;
                    zone.mitigation_active.store(true, Ordering::Relaxed);
                }
                THERMAL_SEVERE => {
                    pressure = max_cap / 2;
                    zone.mitigation_active.store(true, Ordering::Relaxed);
                }
                THERMAL_CRITICAL => {
                    pressure = max_cap - pd.min_capacity;
                    zone.mitigation_active.store(true, Ordering::Relaxed);
                }
                _ => {}
            }

            unsafe {
                GLOBAL_ENERGY_MODEL.invert_capacity_for_thermal(pd_idx, pressure);
            }

            for cpu in pd.cpus.iter() {
                let eff_cap = pd.get_effective_capacity();
                self.max_allowed_capacity[cpu as usize].store(eff_cap, Ordering::Release);
            }
        }
    }

    pub fn get_max_allowed_capacity(&self, cpu: u32) -> u32 {
        if cpu as usize >= MAX_CPUS {
            return 0;
        }
        self.max_allowed_capacity[cpu as usize].load(Ordering::Acquire)
    }

    pub fn is_eas_enabled(&self) -> bool {
        self.eas_enabled.load(Ordering::Acquire)
    }

    pub fn enable_eas(&self) {
        self.eas_enabled.store(true, Ordering::Release);
    }

    pub fn disable_eas(&self) {
        self.eas_enabled.store(false, Ordering::Release);
    }

    pub fn is_emergency_shutdown(&self) -> bool {
        self.emergency_shutdown_triggered.load(Ordering::Acquire)
    }

    pub fn dump_thermal_state(&self) {
        let _ = self.global_thermal_level.load(Ordering::Relaxed);
        let _ = self.nr_zones;
    }
}

pub static mut GLOBAL_POWER_SCHEDULER: PowerScheduler = PowerScheduler::empty();

pub unsafe fn power_init() {
    GLOBAL_POWER_SCHEDULER.init();
    crate::cpu::scheduler::power::em::em_init();
}

pub unsafe fn power_register_zone(pd_idx: u32) -> bool {
    let _guard = GLOBAL_POWER_SCHEDULER.lock.lock_irqsave_guard();
    GLOBAL_POWER_SCHEDULER.register_zone(pd_idx)
}

pub unsafe fn power_tick(current_time_ns: u64) {
    let _guard = GLOBAL_POWER_SCHEDULER.lock.lock_irqsave_guard();
    GLOBAL_POWER_SCHEDULER.update_tick(current_time_ns);
    GLOBAL_POWER_SCHEDULER.apply_mitigation();
}

pub unsafe fn power_get_max_capacity(cpu: u32) -> u32 {
    GLOBAL_POWER_SCHEDULER.get_max_allowed_capacity(cpu)
}

pub unsafe fn power_is_eas_enabled() -> bool {
    GLOBAL_POWER_SCHEDULER.is_eas_enabled()
}

pub unsafe fn power_emergency_check() -> bool {
    GLOBAL_POWER_SCHEDULER.is_emergency_shutdown()
}

pub unsafe fn power_dump_state() {
    GLOBAL_POWER_SCHEDULER.dump_thermal_state();
    crate::cpu::scheduler::power::em::GLOBAL_ENERGY_MODEL.dump_state();
}