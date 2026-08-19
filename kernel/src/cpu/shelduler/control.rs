//! Public configuration, control-plane validation and observability helpers.
//! This module deliberately contains no hardware access and is safe to call
//! from diagnostics, boot configuration and future system-call adapters.

use super::policy::{Class, Policy, Priority};
use super::{CpuMask, Deadline, TaskBudget, TaskConfig, TaskId, TaskState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerMode {
    Disabled,
    Cooperative,
    Preemptive,
    Deterministic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MigrationPolicy {
    Disabled,
    Periodic,
    Pressure,
    Aggressive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakePolicy {
    PreserveCpu,
    Pack,
    Spread,
    PreferIdle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigError {
    ZeroQuantum,
    QuantumTooSmall,
    QuantumTooLarge,
    InvalidAgingPeriod,
    InvalidCpuMask,
    DeadlineWithoutRealtimeClass,
    RuntimeExceedsDeadline,
    BudgetWindowTooSmall,
    AffinityHasNoOnlineCpu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchedulerConfig {
    pub mode: SchedulerMode,
    pub quantum_ns: u64,
    pub min_quantum_ns: u64,
    pub max_quantum_ns: u64,
    pub aging_period_ns: u64,
    pub load_balance_period_ns: u64,
    pub migration: MigrationPolicy,
    pub wake_policy: WakePolicy,
    pub max_latency_ns: u64,
    pub watchdog_ns: u64,
    pub online_cpus: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            mode: SchedulerMode::Cooperative,
            quantum_ns: 4_000_000,
            min_quantum_ns: 500_000,
            max_quantum_ns: 50_000_000,
            aging_period_ns: 2_000_000,
            load_balance_period_ns: 10_000_000,
            migration: MigrationPolicy::Pressure,
            wake_policy: WakePolicy::PreferIdle,
            max_latency_ns: 20_000_000,
            watchdog_ns: 500_000_000,
            online_cpus: 1,
        }
    }
}

impl SchedulerConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.quantum_ns == 0 {
            return Err(ConfigError::ZeroQuantum);
        }
        if self.min_quantum_ns == 0 || self.quantum_ns < self.min_quantum_ns {
            return Err(ConfigError::QuantumTooSmall);
        }
        if self.max_quantum_ns < self.quantum_ns {
            return Err(ConfigError::QuantumTooLarge);
        }
        if self.aging_period_ns == 0 {
            return Err(ConfigError::InvalidAgingPeriod);
        }
        if self.online_cpus == 0 {
            return Err(ConfigError::InvalidCpuMask);
        }
        if self.watchdog_ns < self.max_latency_ns {
            return Err(ConfigError::InvalidAgingPeriod);
        }
        Ok(())
    }

    pub fn cpu_is_online(&self, cpu: usize) -> bool {
        cpu < 64 && (self.online_cpus & (1u64 << cpu)) != 0
    }

    pub fn with_cpu_count(mut self, count: usize) -> Self {
        let bounded = count.clamp(1, 64);
        self.online_cpus = if bounded == 64 {
            u64::MAX
        } else {
            (1u64 << bounded) - 1
        };
        self
    }

    pub fn set_mode(&mut self, mode: SchedulerMode) {
        self.mode = mode;
    }

    pub fn allows_preemption(&self) -> bool {
        matches!(
            self.mode,
            SchedulerMode::Preemptive | SchedulerMode::Deterministic
        )
    }

    pub fn should_balance(&self, elapsed_ns: u64) -> bool {
        elapsed_ns >= self.load_balance_period_ns
            && !matches!(self.migration, MigrationPolicy::Disabled)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskPolicyUpdate {
    pub priority: Option<Priority>,
    pub class: Option<Class>,
    pub policy: Option<Policy>,
    pub affinity: Option<CpuMask>,
    pub deadline: Option<Option<Deadline>>,
    pub budget: Option<TaskBudget>,
}

impl TaskPolicyUpdate {
    pub const fn empty() -> Self {
        Self {
            priority: None,
            class: None,
            policy: None,
            affinity: None,
            deadline: None,
            budget: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.priority.is_none()
            && self.class.is_none()
            && self.policy.is_none()
            && self.affinity.is_none()
            && self.deadline.is_none()
            && self.budget.is_none()
    }

    pub fn apply_to(&self, config: &mut TaskConfig) {
        if let Some(value) = self.priority {
            config.priority = value;
        }
        if let Some(value) = self.class {
            config.class = value;
        }
        if let Some(value) = self.policy {
            config.policy = value;
        }
        if let Some(value) = self.affinity {
            config.affinity = value;
        }
        if let Some(value) = self.deadline {
            config.deadline = value;
        }
        if let Some(value) = self.budget {
            config.budget = value;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskPolicySnapshot {
    pub id: TaskId,
    pub state: TaskState,
    pub class: Class,
    pub policy: Policy,
    pub priority: Priority,
    pub affinity: CpuMask,
    pub cpu: usize,
    pub runtime_ns: u64,
    pub wait_ns: u64,
    pub switches: u64,
    pub migrations: u64,
    pub deadline_ns: Option<u64>,
    pub budget_remaining_ns: u64,
}

impl TaskPolicySnapshot {
    pub fn latency_score(&self, now_ns: u64, last_wake_ns: u64) -> u64 {
        now_ns.saturating_sub(last_wake_ns)
    }

    pub fn is_deadline_task(&self) -> bool {
        self.deadline_ns.is_some()
    }

    pub fn is_runnable(&self) -> bool {
        matches!(self.state, TaskState::Ready | TaskState::Running)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerEvent {
    Initialized {
        cpus: usize,
    },
    TaskCreated {
        id: TaskId,
    },
    TaskReady {
        id: TaskId,
        cpu: usize,
    },
    TaskRunning {
        id: TaskId,
        cpu: usize,
    },
    TaskBlocked {
        id: TaskId,
    },
    TaskSleeping {
        id: TaskId,
        until_ns: u64,
    },
    TaskFinished {
        id: TaskId,
    },
    TaskMigrated {
        id: TaskId,
        from: usize,
        to: usize,
    },
    DeadlineMissed {
        id: TaskId,
        deadline_ns: u64,
        now_ns: u64,
    },
    WatchdogWarning {
        id: TaskId,
        runtime_ns: u64,
    },
    RescheduleRequested {
        cpu: usize,
    },
}

impl SchedulerEvent {
    pub fn task_id(&self) -> Option<TaskId> {
        match *self {
            Self::Initialized { .. } => None,
            Self::TaskCreated { id }
            | Self::TaskReady { id, .. }
            | Self::TaskRunning { id, .. }
            | Self::TaskBlocked { id }
            | Self::TaskSleeping { id, .. }
            | Self::TaskFinished { id }
            | Self::TaskMigrated { id, .. }
            | Self::DeadlineMissed { id, .. }
            | Self::WatchdogWarning { id, .. } => Some(id),
            Self::RescheduleRequested { .. } => None,
        }
    }

    pub fn is_fault_like(&self) -> bool {
        matches!(
            self,
            Self::DeadlineMissed { .. } | Self::WatchdogWarning { .. }
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EventCounters {
    pub created: u64,
    pub ready: u64,
    pub running: u64,
    pub blocked: u64,
    pub sleeping: u64,
    pub finished: u64,
    pub migrated: u64,
    pub deadline_misses: u64,
    pub watchdog_warnings: u64,
    pub reschedule_requests: u64,
}

impl EventCounters {
    pub fn observe(&mut self, event: SchedulerEvent) {
        match event {
            SchedulerEvent::Initialized { .. } => {}
            SchedulerEvent::TaskCreated { .. } => self.created += 1,
            SchedulerEvent::TaskReady { .. } => self.ready += 1,
            SchedulerEvent::TaskRunning { .. } => self.running += 1,
            SchedulerEvent::TaskBlocked { .. } => self.blocked += 1,
            SchedulerEvent::TaskSleeping { .. } => self.sleeping += 1,
            SchedulerEvent::TaskFinished { .. } => self.finished += 1,
            SchedulerEvent::TaskMigrated { .. } => self.migrated += 1,
            SchedulerEvent::DeadlineMissed { .. } => self.deadline_misses += 1,
            SchedulerEvent::WatchdogWarning { .. } => self.watchdog_warnings += 1,
            SchedulerEvent::RescheduleRequested { .. } => self.reschedule_requests += 1,
        }
    }

    pub fn total(&self) -> u64 {
        self.created
            + self.ready
            + self.running
            + self.blocked
            + self.sleeping
            + self.finished
            + self.migrated
            + self.deadline_misses
            + self.watchdog_warnings
            + self.reschedule_requests
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LoadWindow {
    pub samples: u32,
    pub busy_ns: u64,
    pub idle_ns: u64,
    pub last_update_ns: u64,
}

impl LoadWindow {
    pub fn update(&mut self, busy_ns: u64, idle_ns: u64, now_ns: u64) {
        self.samples = self.samples.saturating_add(1);
        self.busy_ns = self.busy_ns.saturating_add(busy_ns);
        self.idle_ns = self.idle_ns.saturating_add(idle_ns);
        self.last_update_ns = now_ns;
    }

    pub fn total_ns(&self) -> u64 {
        self.busy_ns.saturating_add(self.idle_ns)
    }

    pub fn busy_ppm(&self) -> u64 {
        let total = self.total_ns();
        if total == 0 {
            0
        } else {
            self.busy_ns.saturating_mul(1_000_000) / total
        }
    }

    pub fn reset(&mut self, now_ns: u64) {
        self.samples = 0;
        self.busy_ns = 0;
        self.idle_ns = 0;
        self.last_update_ns = now_ns;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacementScore {
    pub cpu: usize,
    pub load: u64,
    pub cache_affinity: u16,
    pub migration_cost: u16,
    pub allowed: bool,
}

impl PlacementScore {
    pub fn total_cost(&self) -> u64 {
        if !self.allowed {
            return u64::MAX;
        }
        self.load
            .saturating_add(self.migration_cost as u64 * 256)
            .saturating_sub(self.cache_affinity as u64 * 64)
    }

    pub fn better_than(&self, other: &Self) -> bool {
        self.total_cost() < other.total_cost()
    }
}

pub fn validate_task_config(config: &TaskConfig, online: CpuMask) -> Result<(), ConfigError> {
    if config.stack_size < 4096 {
        return Err(ConfigError::QuantumTooSmall);
    }
    if config.affinity.0 & online.0 == 0 {
        return Err(ConfigError::AffinityHasNoOnlineCpu);
    }
    if config.deadline.is_some() && !matches!(config.class, Class::RealTime) {
        return Err(ConfigError::DeadlineWithoutRealtimeClass);
    }
    if let Some(deadline) = config.deadline {
        if deadline.runtime_ns > deadline.deadline_ns {
            return Err(ConfigError::RuntimeExceedsDeadline);
        }
    }
    if config.budget.quota_ns != 0 && config.budget.window_ns < config.budget.quota_ns {
        return Err(ConfigError::BudgetWindowTooSmall);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_configuration_is_valid() {
        let config = SchedulerConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn online_cpu_mask_is_contiguous() {
        let config = SchedulerConfig::default().with_cpu_count(4);
        assert!(config.cpu_is_online(0));
        assert!(config.cpu_is_online(3));
        assert!(!config.cpu_is_online(4));
    }

    #[test]
    fn event_counters_track_faults() {
        let mut counters = EventCounters::default();
        counters.observe(SchedulerEvent::DeadlineMissed {
            id: TaskId(7),
            deadline_ns: 10,
            now_ns: 11,
        });
        assert_eq!(counters.deadline_misses, 1);
    }

    #[test]
    fn placement_prefers_low_cost() {
        let a = PlacementScore {
            cpu: 0,
            load: 10,
            cache_affinity: 0,
            migration_cost: 0,
            allowed: true,
        };
        let b = PlacementScore {
            cpu: 1,
            load: 20,
            cache_affinity: 0,
            migration_cost: 0,
            allowed: true,
        };
        assert!(a.better_than(&b));
    }
}
