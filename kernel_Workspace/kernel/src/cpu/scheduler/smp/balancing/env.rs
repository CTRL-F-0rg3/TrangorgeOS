#![allow(dead_code)]

use crate::cpu::scheduler::smp::topology::SchedDomain;

pub const LBF_ALL_PINNED: u32 = 0x01;
pub const LBF_NEED_BREAK: u32 = 0x02;
pub const LBF_SOME_PINNED: u32 = 0x04;
pub const LBF_ACTIVE_LB: u32 = 0x08;

pub const DEFAULT_LOOP_MAX: u32 = 32;

/// Kontekst pojedynczego przebiegu równoważenia obciążenia dla pary
/// CPU (src -> dst) w obrębie jednej domeny szeregowania — odpowiednik
/// `struct lb_env` z Linuksa. `calculate.rs` odpowiada za policzenie
/// nierównowagi; ten typ trzyma stan samego PRZEBIEGU przenoszenia
/// zadań (limit iteracji, flagi pinningu, licznik porażek).
#[repr(C)]
pub struct LoadBalanceEnv {
    pub sd: *mut SchedDomain,
    pub src_cpu: u32,
    pub dst_cpu: u32,
    pub flags: u32,
    pub loop_max: u32,
    pub loop_done: u32,
    pub failed: u32,
}

impl LoadBalanceEnv {
    pub const fn new(sd: *mut SchedDomain, src_cpu: u32, dst_cpu: u32) -> Self {
        Self {
            sd,
            src_cpu,
            dst_cpu,
            flags: 0,
            loop_max: DEFAULT_LOOP_MAX,
            loop_done: 0,
            failed: 0,
        }
    }

    pub fn set_flag(&mut self, flag: u32) {
        self.flags |= flag;
    }

    pub fn clear_flag(&mut self, flag: u32) {
        self.flags &= !flag;
    }

    pub fn has_flag(&self, flag: u32) -> bool {
        self.flags & flag != 0
    }

    /// Czy przebieg powinien się zatrzymać (osiągnięto limit prób
    /// przeniesienia zadań w tej iteracji `load_balance`).
    pub fn should_stop(&self) -> bool {
        self.loop_done >= self.loop_max
    }

    pub fn record_attempt(&mut self) {
        self.loop_done += 1;
    }

    pub fn record_failure(&mut self) {
        self.failed += 1;
        self.set_flag(LBF_SOME_PINNED);
    }

    pub fn reset_for_next_iteration(&mut self) {
        self.loop_done = 0;
        self.clear_flag(LBF_NEED_BREAK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ptr;

    fn env() -> LoadBalanceEnv {
        LoadBalanceEnv::new(ptr::null_mut(), 0, 1)
    }

    #[test]
    fn flags_set_clear_roundtrip() {
        let mut e = env();
        assert!(!e.has_flag(LBF_ALL_PINNED));
        e.set_flag(LBF_ALL_PINNED);
        assert!(e.has_flag(LBF_ALL_PINNED));
        e.clear_flag(LBF_ALL_PINNED);
        assert!(!e.has_flag(LBF_ALL_PINNED));
    }

    #[test]
    fn should_stop_once_loop_max_reached() {
        let mut e = env();
        e.loop_max = 3;
        assert!(!e.should_stop());
        e.record_attempt();
        e.record_attempt();
        assert!(!e.should_stop());
        e.record_attempt();
        assert!(e.should_stop());
    }

    #[test]
    fn record_failure_sets_some_pinned() {
        let mut e = env();
        assert!(!e.has_flag(LBF_SOME_PINNED));
        e.record_failure();
        assert_eq!(e.failed, 1);
        assert!(e.has_flag(LBF_SOME_PINNED));
    }

    #[test]
    fn reset_clears_loop_counter_and_need_break() {
        let mut e = env();
        e.record_attempt();
        e.set_flag(LBF_NEED_BREAK);
        e.reset_for_next_iteration();
        assert_eq!(e.loop_done, 0);
        assert!(!e.has_flag(LBF_NEED_BREAK));
    }
}