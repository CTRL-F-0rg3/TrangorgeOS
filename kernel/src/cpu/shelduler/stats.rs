//! Low-cost scheduler accounting and observability.
#[derive(Clone, Copy, Debug, Default)]
pub struct CpuStats {
    pub ticks: u64,
    pub runtime_ns: u64,
    pub idle_ns: u64,
    pub switches: u64,
    pub preemptions: u64,
    pub voluntary: u64,
    pub enqueues: u64,
    pub migrations: u64,
    pub missed_deadlines: u64,
}
impl CpuStats {
    pub fn on_tick(&mut self, ns: u64) {
        self.ticks += 1;
        self.runtime_ns = self.runtime_ns.saturating_add(ns)
    }
    pub fn on_switch(&mut self) {
        self.switches += 1
    }
    pub fn on_enqueue(&mut self) {
        self.enqueues += 1
    }
    pub fn on_preempt(&mut self) {
        self.preemptions += 1
    }
    pub fn on_idle(&mut self, ns: u64) {
        self.idle_ns = self.idle_ns.saturating_add(ns)
    }
    pub fn utilization_ppm(&self) -> u64 {
        if self.runtime_ns == 0 {
            0
        } else {
            self.runtime_ns.saturating_mul(1_000_000)
                / (self.runtime_ns.saturating_add(self.idle_ns).max(1))
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct SchedulerSnapshot {
    pub now_ns: u64,
    pub cpu_count: usize,
    pub task_count: usize,
    pub ready_count: usize,
    pub running_count: usize,
    pub load: u64,
}
