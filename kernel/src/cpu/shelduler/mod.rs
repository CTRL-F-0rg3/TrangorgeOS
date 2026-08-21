#![allow(dead_code)]

pub mod context;
pub mod control;
pub mod policy;
pub mod predictor;
pub mod queue;
pub mod stats;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use spin::Mutex;

pub use control::{EventCounters, SchedulerConfig, SchedulerEvent, SchedulerMode};
pub use policy::{Class, Policy, Priority, SchedulingKey};
pub use predictor::{BurstClass, BurstPredictor, Prediction};
pub use queue::{QueueSnapshot, RunQueue};
pub use stats::{CpuStats, SchedulerSnapshot};

const MAX_CPUS: usize = 32;
const DEFAULT_QUANTUM_NS: u64 = 4_000_000;
const MIN_QUANTUM_NS: u64 = 500_000;
const MAX_QUANTUM_NS: u64 = 50_000_000;
const PRIORITY_LEVELS: usize = 32;
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);
static TICKS: AtomicU64 = AtomicU64::new(0);
static ONLINE_CPUS: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskId(pub u64);
impl TaskId {
    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    New,
    Ready,
    Running,
    Sleeping,
    Blocked,
    Finished,
    Zombie,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakeReason {
    Spawn,
    Timer,
    Io,
    Explicit,
    Unblock,
    Migration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpuMask(pub u64);
impl CpuMask {
    pub const fn all() -> Self {
        Self(u64::MAX)
    }
    pub const fn only(cpu: usize) -> Self {
        if cpu < 64 {
            Self(1u64 << cpu)
        } else {
            Self(0)
        }
    }
    pub const fn allows(self, cpu: usize) -> bool {
        cpu < 64 && (self.0 & (1u64 << cpu)) != 0
    }
    pub fn set(&mut self, cpu: usize, value: bool) {
        if cpu < 64 {
            let bit = 1u64 << cpu;
            if value {
                self.0 |= bit;
            } else {
                self.0 &= !bit;
            }
        }
    }
    pub fn weight(self) -> u32 {
        self.0.count_ones()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deadline {
    pub deadline_ns: u64,
    pub period_ns: u64,
    pub runtime_ns: u64,
}
impl Deadline {
    pub const fn new(deadline_ns: u64, period_ns: u64, runtime_ns: u64) -> Self {
        Self {
            deadline_ns,
            period_ns,
            runtime_ns,
        }
    }
    pub fn expired(self, now: u64) -> bool {
        now >= self.deadline_ns
    }
    pub fn slack(self, now: u64) -> u64 {
        self.deadline_ns.saturating_sub(now)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskBudget {
    pub quota_ns: u64,
    pub used_ns: u64,
    pub window_ns: u64,
    pub window_start: u64,
}
impl TaskBudget {
    pub const fn unlimited() -> Self {
        Self {
            quota_ns: 0,
            used_ns: 0,
            window_ns: 0,
            window_start: 0,
        }
    }
    pub fn limited(quota_ns: u64, window_ns: u64, now: u64) -> Self {
        Self {
            quota_ns,
            used_ns: 0,
            window_ns,
            window_start: now,
        }
    }
    pub fn account(&mut self, delta: u64, now: u64) {
        self.roll(now);
        self.used_ns = self.used_ns.saturating_add(delta);
    }
    pub fn roll(&mut self, now: u64) {
        if self.window_ns != 0 && now.saturating_sub(self.window_start) >= self.window_ns {
            self.window_start = now;
            self.used_ns = 0;
        }
    }
    pub fn exhausted(&self) -> bool {
        self.quota_ns != 0 && self.used_ns >= self.quota_ns
    }
    pub fn remaining(&self) -> u64 {
        if self.quota_ns == 0 {
            u64::MAX
        } else {
            self.quota_ns.saturating_sub(self.used_ns)
        }
    }
}

pub struct Task {
    pub id: TaskId,
    pub name: &'static str,
    pub ctx: context::Context,
    pub state: TaskState,
    pub class: Class,
    pub priority: Priority,
    pub base_priority: Priority,
    pub policy: Policy,
    pub affinity: CpuMask,
    pub deadline: Option<Deadline>,
    pub budget: TaskBudget,
    pub predictor: BurstPredictor,
    pub stats: TaskStats,
    pub is_idle: bool,
    pub preempt_disable: u32,
    pub wake_generation: u64,
    stack: Vec<u8>,
    entry: Option<Box<dyn FnOnce() + Send + 'static>>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TaskStats {
    pub runtime_ns: u64,
    pub wait_ns: u64,
    pub sleep_ns: u64,
    pub switches: u64,
    pub migrations: u64,
    pub voluntary_yields: u64,
    pub involuntary_preempts: u64,
    pub wakeups: u64,
    pub last_start_ns: u64,
    pub last_wake_ns: u64,
}
impl TaskStats {
    pub fn on_run(&mut self, now: u64) {
        self.last_start_ns = now;
        self.switches = self.switches.saturating_add(1);
    }
    pub fn on_stop(&mut self, now: u64) {
        self.runtime_ns = self
            .runtime_ns
            .saturating_add(now.saturating_sub(self.last_start_ns));
    }
    pub fn on_wake(&mut self, now: u64) {
        self.wait_ns = self
            .wait_ns
            .saturating_add(now.saturating_sub(self.last_wake_ns));
        self.last_wake_ns = now;
        self.wakeups = self.wakeups.saturating_add(1);
    }
}

impl Task {
    pub fn new<F>(name: &'static str, f: F) -> Box<Self>
    where
        F: FnOnce() + Send + 'static,
    {
        Self::with_config(name, f, TaskConfig::default())
    }
    pub fn with_config<F>(name: &'static str, f: F, config: TaskConfig) -> Box<Self>
    where
        F: FnOnce() + Send + 'static,
    {
        let id = TaskId(NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed));
        let mut stack = Vec::with_capacity(config.stack_size);
        unsafe {
            stack.set_len(config.stack_size);
        }
        let top = (stack.as_ptr() as u64 + config.stack_size as u64 - 8) & !15u64;
        let mut task = Box::new(Self {
            id,
            name,
            ctx: context::Context::empty(),
            state: TaskState::New,
            class: config.class,
            priority: config.priority,
            base_priority: config.priority,
            policy: config.policy,
            affinity: config.affinity,
            deadline: config.deadline,
            budget: config.budget,
            predictor: BurstPredictor::new(),
            stats: TaskStats::default(),
            is_idle: config.is_idle,
            preempt_disable: 0,
            wake_generation: 0,
            stack,
            entry: Some(Box::new(f)),
        });
        task.ctx = context::Context::bootstrap(top, task_entry as usize as u64, task.id.0);
        task
    }
    pub fn new_idle(name: &'static str) -> Box<Self> {
        Self::with_config(
            name,
            || loop {
                x86_64::instructions::hlt();
                yield_now();
            },
            TaskConfig {
                is_idle: true,
                class: Class::Idle,
                priority: Priority::lowest(),
                policy: Policy::Idle,
                ..TaskConfig::default()
            },
        )
    }
    pub fn effective_priority(&self, now: u64) -> Priority {
        self.priority
            .aged(now.saturating_sub(self.stats.last_wake_ns), self.class)
    }
    pub fn is_runnable(&self) -> bool {
        matches!(self.state, TaskState::Ready | TaskState::Running)
    }
    pub fn mark_ready(&mut self, now: u64, reason: WakeReason) {
        self.state = TaskState::Ready;
        self.stats.on_wake(now);
        self.wake_generation = self.wake_generation.wrapping_add(1);
        if reason == WakeReason::Explicit {
            self.predictor.on_wakeup();
        }
    }
    pub fn mark_running(&mut self, now: u64) {
        self.state = TaskState::Running;
        self.stats.on_run(now);
    }
    pub fn mark_finished(&mut self) {
        self.state = TaskState::Finished;
    }
    pub fn account(&mut self, delta: u64, now: u64) {
        self.stats.runtime_ns = self.stats.runtime_ns.saturating_add(delta);
        self.budget.account(delta, now);
        self.predictor.observe_run(delta);
    }
    pub fn quantum(&self) -> u64 {
        self.predictor
            .quantum(DEFAULT_QUANTUM_NS, self.class, self.priority)
            .clamp(MIN_QUANTUM_NS, MAX_QUANTUM_NS)
    }
    pub fn disable_preemption(&mut self) {
        self.preempt_disable = self.preempt_disable.saturating_add(1);
    }
    pub fn enable_preemption(&mut self) {
        self.preempt_disable = self.preempt_disable.saturating_sub(1);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TaskConfig {
    pub stack_size: usize,
    pub class: Class,
    pub priority: Priority,
    pub policy: Policy,
    pub affinity: CpuMask,
    pub deadline: Option<Deadline>,
    pub budget: TaskBudget,
    pub is_idle: bool,
}
impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            stack_size: 64 * 1024,
            class: Class::Normal,
            priority: Priority::new(16),
            policy: Policy::Fair,
            affinity: CpuMask::all(),
            deadline: None,
            budget: TaskBudget::unlimited(),
            is_idle: false,
        }
    }
}

#[repr(C)]
pub struct PerCpuScheduler {
    pub cpu_id: usize,
    pub current: Option<TaskId>,
    pub idle: Option<TaskId>,
    pub need_reschedule: bool,
    pub preempt_count: u32,
    pub clock_ns: u64,
    pub rq: RunQueue,
    pub stats: CpuStats,
    pub next_deadline: u64,
}
impl PerCpuScheduler {
    pub fn new(cpu_id: usize) -> Self {
        Self {
            cpu_id,
            current: None,
            idle: None,
            need_reschedule: false,
            preempt_count: 0,
            clock_ns: 0,
            rq: RunQueue::new(cpu_id),
            stats: CpuStats::default(),
            next_deadline: u64::MAX,
        }
    }
}

pub struct Scheduler {
    pub cpus: Vec<PerCpuScheduler>,
    pub tasks: Vec<Box<Task>>,
    pub global_epoch: u64,
    pub load_balance_cursor: usize,
}
impl Scheduler {
    pub fn new(cpu_count: usize) -> Self {
        let count = cpu_count.clamp(1, MAX_CPUS);
        let mut cpus = Vec::with_capacity(count);
        for id in 0..count {
            cpus.push(PerCpuScheduler::new(id));
        }
        Self {
            cpus,
            tasks: Vec::new(),
            global_epoch: 0,
            load_balance_cursor: 0,
        }
    }
    pub fn spawn(&mut self, mut task: Box<Task>, cpu_hint: Option<usize>, now: u64) -> TaskId {
        let id = task.id;
        task.mark_ready(now, WakeReason::Spawn);
        let cpu = self.choose_cpu(&task, cpu_hint);
        self.cpus[cpu].rq.push(
            task.id,
            task.effective_priority(now),
            task.policy,
            task.deadline,
            now,
        );
        self.tasks.push(task);
        self.cpus[cpu].stats.on_enqueue();
        id
    }
    pub fn choose_cpu(&self, task: &Task, hint: Option<usize>) -> usize {
        if let Some(c) = hint {
            if c < self.cpus.len() && task.affinity.allows(c) {
                return c;
            }
        }
        self.cpus
            .iter()
            .filter(|c| task.affinity.allows(c.cpu_id))
            .min_by_key(|c| c.rq.len())
            .map(|c| c.cpu_id)
            .unwrap_or(0)
    }
    pub fn find(&self, id: TaskId) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id).map(|t| &**t)
    }
    pub fn find_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id).map(|t| &mut **t)
    }
    pub fn enqueue(&mut self, id: TaskId, cpu: usize, now: u64, reason: WakeReason) -> bool {
        if cpu >= self.cpus.len() {
            return false;
        }
        let Some(task) = self.find(id) else {
            return false;
        };
        if !task.affinity.allows(cpu) {
            return false;
        }
        let Some(task) = self.find_mut(id) else {
            return false;
        };
        task.mark_ready(now, reason);
        let key = task.effective_priority(now);
        let policy = task.policy;
        let deadline = task.deadline;
        self.cpus[cpu].rq.push(id, key, policy, deadline, now);
        self.cpus[cpu].stats.on_enqueue();
        true
    }
    pub fn dequeue_next(&mut self, cpu: usize, now: u64) -> Option<TaskId> {
        if cpu >= self.cpus.len() {
            return None;
        }
        let id = self.cpus[cpu].rq.pop(now)?;
        let task = self.find_mut(id)?;
        task.mark_running(now);
        self.cpus[cpu].current = Some(id);
        self.cpus[cpu].stats.on_switch();
        Some(id)
    }
    pub fn tick(&mut self, cpu: usize, now: u64, elapsed: u64) -> bool {
        if cpu >= self.cpus.len() {
            return false;
        }
        self.global_epoch = self.global_epoch.wrapping_add(1);
        self.cpus[cpu].clock_ns = now;
        self.cpus[cpu].stats.on_tick(elapsed);
        if let Some(id) = self.cpus[cpu].current {
            if let Some(t) = self.find_mut(id) {
                t.account(elapsed, now);
                if t.preempt_disable == 0
                    && (t.budget.exhausted()
                        || t.quantum() <= elapsed
                        || self.global_epoch % 8 == 0)
                {
                    self.cpus[cpu].need_reschedule = true;
                }
            }
        }
        self.cpus[cpu].rq.promote_aged(now);
        if self.global_epoch % 64 == 0 {
            self.rebalance(now);
        }
        self.cpus[cpu].need_reschedule
    }
    pub fn yield_current(&mut self, cpu: usize, now: u64) -> Option<TaskId> {
        let id = self.cpus.get(cpu)?.current?;
        if let Some(t) = self.find_mut(id) {
            t.stats.voluntary_yields = t.stats.voluntary_yields.saturating_add(1);
            t.state = TaskState::Ready;
            let key = t.effective_priority(now);
            let p = t.policy;
            let d = t.deadline;
            self.cpus[cpu].rq.push(id, key, p, d, now);
        }
        self.cpus[cpu].need_reschedule = true;
        self.dequeue_next(cpu, now)
    }
    pub fn preempt_current(&mut self, cpu: usize, now: u64) -> Option<TaskId> {
        let id = self.cpus.get(cpu)?.current?;
        if let Some(t) = self.find_mut(id) {
            if t.preempt_disable != 0 {
                return Some(id);
            }
            t.stats.involuntary_preempts = t.stats.involuntary_preempts.saturating_add(1);
            t.state = TaskState::Ready;
            let key = t.effective_priority(now);
            let p = t.policy;
            let d = t.deadline;
            self.cpus[cpu].rq.push(id, key, p, d, now);
        }
        self.cpus[cpu].need_reschedule = false;
        self.dequeue_next(cpu, now)
    }
    pub fn block_current(&mut self, cpu: usize) -> Option<TaskId> {
        let id = self.cpus.get(cpu)?.current?;
        if let Some(t) = self.find_mut(id) {
            t.state = TaskState::Blocked;
        }
        self.cpus[cpu].current = None;
        self.dequeue_next(cpu, self.cpus[cpu].clock_ns)
    }
    pub fn wake(&mut self, id: TaskId, now: u64, reason: WakeReason) -> bool {
        let cpu = self.find(id).map(|t| self.choose_cpu(t, None)).unwrap_or(0);
        self.enqueue(id, cpu, now, reason)
    }
    pub fn sleep_until(&mut self, cpu: usize, id: TaskId, _deadline: u64) -> bool {
        if let Some(t) = self.find_mut(id) {
            t.state = TaskState::Sleeping;
            if self.cpus.get(cpu).and_then(|c| c.current) == Some(id) {
                self.cpus[cpu].current = None;
            }
            return true;
        }
        false
    }
    pub fn rebalance(&mut self, now: u64) {
        if self.cpus.len() < 2 {
            return;
        }
        let source = self.load_balance_cursor % self.cpus.len();
        self.load_balance_cursor = self.load_balance_cursor.wrapping_add(1);
        let target = self
            .cpus
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| c.rq.len())
            .map(|(i, _)| i)
            .unwrap_or(source);
        if source == target || self.cpus[source].rq.len() <= self.cpus[target].rq.len() + 1 {
            return;
        }
        let Some(id) = self.cpus[source].rq.steal_one(now) else {
            return;
        };
        let Some(task) = self.find(id) else {
            return;
        };
        let allowed = task.affinity.allows(target);
        let key = task.effective_priority(now);
        let policy = task.policy;
        let deadline = task.deadline;
        if allowed {
            if let Some(task) = self.find_mut(id) {
                task.stats.migrations = task.stats.migrations.saturating_add(1);
            }
            self.cpus[target].rq.push(id, key, policy, deadline, now);
            self.cpus[target].stats.migrations =
                self.cpus[target].stats.migrations.saturating_add(1);
        } else {
            self.cpus[source].rq.push(id, key, policy, deadline, now);
        }
    }
    pub fn snapshot(&self, now: u64) -> SchedulerSnapshot {
        let mut snap = SchedulerSnapshot::default();
        snap.now_ns = now;
        snap.cpu_count = self.cpus.len();
        snap.task_count = self.tasks.len();
        snap.ready_count = self.cpus.iter().map(|c| c.rq.len()).sum();
        snap.running_count = self.cpus.iter().filter(|c| c.current.is_some()).count();
        snap.load = self.cpus.iter().map(|c| c.rq.load_score()).sum();
        snap
    }
}

static GLOBAL: Mutex<Option<Scheduler>> = Mutex::new(None);

fn with_global<R>(f: impl FnOnce(&mut Scheduler) -> R) -> Option<R> {
    let mut guard = GLOBAL.lock();
    let scheduler = guard.as_mut()?;
    Some(f(scheduler))
}

pub fn init(cpu_count: usize) {
    let mut g = GLOBAL.lock();
    if g.is_none() {
        *g = Some(Scheduler::new(cpu_count));
        ONLINE_CPUS.store(cpu_count.clamp(1, MAX_CPUS), Ordering::Release);
        INITIALIZED.store(true, Ordering::Release);
    }
}
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Acquire)
}
pub fn total_cpus() -> usize {
    ONLINE_CPUS.load(Ordering::Acquire)
}
pub fn spawn<F>(name: &'static str, f: F, config: TaskConfig) -> Option<TaskId>
where
    F: FnOnce() + Send + 'static,
{
    with_global(|s| s.spawn(Task::with_config(name, f, config), None, clock_now()))
}
pub fn tick(cpu: usize, elapsed_ns: u64) -> bool {
    TICKS.fetch_add(1, Ordering::Relaxed);
    with_global(|s| s.tick(cpu, clock_now(), elapsed_ns)).unwrap_or(false)
}
pub fn yield_now() {
    let _ = with_global(|s| {
        let cpu = current_cpu();
        let now = clock_now();
        s.yield_current(cpu, now);
    });
}
pub fn exit_current_task() -> ! {
    let _ = with_global(|s| {
        let cpu = current_cpu();
        if let Some(id) = s.cpus[cpu].current {
            if let Some(t) = s.find_mut(id) {
                t.mark_finished();
            }
            s.cpus[cpu].current = None;
        }
        s.dequeue_next(cpu, clock_now());
    });
    loop {
        x86_64::instructions::hlt();
    }
}
pub fn schedule_from_interrupt() -> bool {
    tick(current_cpu(), DEFAULT_QUANTUM_NS)
}

pub fn request_reschedule(cpu: usize) {
    let _ = with_global(|scheduler| {
        if let Some(per_cpu) = scheduler.cpus.get_mut(cpu) {
            per_cpu.need_reschedule = true;
        }
    });
}

pub fn preempt_current_if_allowed(cpu: usize) -> Option<TaskId> {
    with_global(|scheduler| scheduler.preempt_current(cpu, clock_now())).flatten()
}

pub fn disable_preemption() {
    let _ = with_global(|scheduler| {
        let cpu = current_cpu();
        if let Some(id) = scheduler.cpus.get(cpu).and_then(|state| state.current) {
            if let Some(task) = scheduler.find_mut(id) {
                task.disable_preemption();
            }
        }
    });
}

pub fn enable_preemption() {
    let _ = with_global(|scheduler| {
        let cpu = current_cpu();
        if let Some(id) = scheduler.cpus.get(cpu).and_then(|state| state.current) {
            if let Some(task) = scheduler.find_mut(id) {
                task.enable_preemption();
            }
        }
    });
}
pub fn current_cpu() -> usize {
    0
}
pub fn clock_now() -> u64 {
    TICKS
        .load(Ordering::Relaxed)
        .saturating_mul(DEFAULT_QUANTUM_NS)
}
pub fn snapshot() -> Option<SchedulerSnapshot> {
    with_global(|s| s.snapshot(clock_now()))
}

unsafe extern "C" fn task_entry(_id: u64) -> ! {
    let _ = with_global(|s| {
        let cpu = current_cpu();
        let id = s.cpus[cpu].current?;
        let task = s.find_mut(id)?;
        let entry = task.entry.take()?;
        Some(entry())
    });
    exit_current_task()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn affinity_restricts_cpu_choice() {
        let s = Scheduler::new(4);
        let mut c = TaskConfig::default();
        c.affinity = CpuMask::only(2);
        let t = Task::with_config("x", || {}, c);
        assert_eq!(s.choose_cpu(&t, None), 2);
    }
    #[test]
    fn budget_rolls_and_expires() {
        let mut b = TaskBudget::limited(10, 100, 0);
        b.account(11, 1);
        assert!(b.exhausted());
        b.roll(100);
        assert!(!b.exhausted());
    }
    #[test]
    fn task_state_transitions() {
        let mut t = Task::new("x", || {});
        t.mark_ready(1, WakeReason::Spawn);
        t.mark_running(2);
        assert_eq!(t.state, TaskState::Running);
        t.mark_finished();
        assert_eq!(t.state, TaskState::Finished);
    }
}
