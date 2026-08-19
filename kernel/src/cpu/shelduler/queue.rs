//! Bounded-allocation run queue with per-policy lanes.
use super::policy::{Policy, Priority, SchedulingKey};
use super::{Deadline, TaskId};
use alloc::collections::VecDeque;
use alloc::vec::Vec;
#[derive(Clone, Copy, Debug, Default)]
pub struct QueueSnapshot {
    pub ready: usize,
    pushes: u64,
    pops: u64,
    steals: u64,
    aged: u64,
}
#[derive(Debug)]
pub struct RunQueue {
    cpu: usize,
    lanes: [VecDeque<TaskId>; 32],
    keys: Vec<(TaskId, SchedulingKey)>,
    sequence: u64,
    snapshot: QueueSnapshot,
}
impl RunQueue {
    pub fn new(cpu: usize) -> Self {
        Self {
            cpu,
            lanes: core::array::from_fn(|_| VecDeque::new()),
            keys: Vec::new(),
            sequence: 0,
            snapshot: QueueSnapshot::default(),
        }
    }
    pub fn len(&self) -> usize {
        self.lanes.iter().map(|x| x.len()).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn push(
        &mut self,
        id: TaskId,
        p: Priority,
        policy: Policy,
        deadline: Option<Deadline>,
        now: u64,
    ) {
        let k = SchedulingKey::for_task(
            p,
            policy,
            deadline.map(|d| d.deadline_ns),
            now,
            self.sequence,
        );
        self.sequence = self.sequence.wrapping_add(1);
        self.lanes[p.value() as usize].push_back(id);
        self.keys.push((id, k));
        self.snapshot.pushes += 1;
    }
    pub fn pop(&mut self, now: u64) -> Option<TaskId> {
        let mut best = None;
        for (i, (id, k)) in self.keys.iter().enumerate() {
            let mut key = *k;
            key.virtual_runtime = key.virtual_runtime.min(now);
            if best
                .map(|b: usize| key.better_than(self.keys[b].1))
                .unwrap_or(true)
            {
                best = Some(i);
            }
        }
        let i = best?;
        let id = self.keys.swap_remove(i).0;
        for lane in &mut self.lanes {
            if let Some(pos) = lane.iter().position(|x| *x == id) {
                lane.remove(pos);
                break;
            }
        }
        self.snapshot.pops += 1;
        Some(id)
    }
    pub fn steal_one(&mut self, _now: u64) -> Option<TaskId> {
        let id = self.lanes.iter_mut().rev().find_map(|l| l.pop_back())?;
        if let Some(pos) = self.keys.iter().position(|(x, _)| *x == id) {
            self.keys.swap_remove(pos);
        }
        self.snapshot.steals += 1;
        Some(id)
    }
    pub fn promote_aged(&mut self, _now: u64) {
        self.snapshot.aged = self.snapshot.aged.saturating_add(1);
    }
    pub fn load_score(&self) -> u64 {
        self.len() as u64 * 1024 + self.snapshot.pushes.saturating_sub(self.snapshot.pops)
    }
    pub fn snapshot(&self) -> QueueSnapshot {
        self.snapshot
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fifo_lane_is_drained() {
        let mut q = RunQueue::new(0);
        q.push(TaskId(1), Priority::new(3), Policy::Fair, None, 0);
        q.push(TaskId(2), Priority::new(3), Policy::Fair, None, 0);
        assert!(q.pop(0).is_some());
        assert_eq!(q.len(), 1);
    }
}
