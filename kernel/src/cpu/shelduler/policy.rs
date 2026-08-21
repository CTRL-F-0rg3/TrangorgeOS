#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Idle,
    RealTime,
    Interactive,
    Normal,
    Background,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    Idle,
    Fifo,
    Edf,
    Fair,
    Batch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Priority(pub u8);
impl Priority {
    pub const fn new(v: u8) -> Self {
        Self(if v > 31 { 31 } else { v })
    }
    pub const fn lowest() -> Self {
        Self(0)
    }
    pub const fn highest() -> Self {
        Self(31)
    }
    pub fn value(self) -> u8 {
        self.0
    }
    pub fn aged(self, wait_ns: u64, class: Class) -> Self {
        let step = (wait_ns / 2_000_000).min(15) as u8;
        let bonus = match class {
            Class::RealTime => 8,
            Class::Interactive => 4,
            Class::Background => 0,
            Class::Idle => 0,
            Class::Normal => 2,
        };
        Self::new(self.0.saturating_add(step).saturating_add(bonus))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchedulingKey {
    pub class_rank: u8,
    pub priority: u8,
    pub deadline: u64,
    pub virtual_runtime: u64,
    pub sequence: u64,
}
impl SchedulingKey {
    pub fn for_task(
        priority: Priority,
        policy: Policy,
        deadline: Option<u64>,
        now: u64,
        sequence: u64,
    ) -> Self {
        let class_rank = match policy {
            Policy::Edf => 5,
            Policy::Fifo => 4,
            Policy::Fair => 3,
            Policy::Batch => 2,
            Policy::Idle => 0,
        };
        Self {
            class_rank,
            priority: priority.value(),
            deadline: deadline.unwrap_or(u64::MAX),
            virtual_runtime: now,
            sequence,
        }
    }
    pub fn better_than(self, other: Self) -> bool {
        if self.class_rank != other.class_rank {
            return self.class_rank > other.class_rank;
        }
        if self.class_rank == 5 && self.deadline != other.deadline {
            return self.deadline < other.deadline;
        }
        if self.priority != other.priority {
            return self.priority > other.priority;
        }
        if self.virtual_runtime != other.virtual_runtime {
            return self.virtual_runtime < other.virtual_runtime;
        }
        self.sequence < other.sequence
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edf_precedes_later_deadline() {
        let a = SchedulingKey::for_task(Priority::new(1), Policy::Edf, Some(5), 0, 0);
        let b = SchedulingKey::for_task(Priority::new(31), Policy::Edf, Some(9), 0, 1);
        assert!(a.better_than(b));
    }
    #[test]
    fn aging_is_bounded() {
        assert_eq!(Priority::new(1).aged(u64::MAX, Class::Normal).value(), 18);
    }
}
