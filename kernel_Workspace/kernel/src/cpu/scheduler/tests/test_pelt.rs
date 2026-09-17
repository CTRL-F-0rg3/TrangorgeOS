use crate::cpu::scheduler::entities::task::{LoadAvg, NICE_0_LOAD};

#[test]
fn default_load_avg_is_zero() {
    let la = LoadAvg::default();
    assert_eq!(la.load_avg, 0);
    assert_eq!(la.util_avg, 0);
    assert_eq!(la.load_sum, 0);
    assert_eq!(la.util_sum, 0);
    assert_eq!(la.last_update_time, 0);
}

#[test]
fn accumulate_zero_delta_does_not_change_state_except_time() {
    let mut la = LoadAvg::default();
    la.accumulate(100, 0, NICE_0_LOAD, true);
    assert_eq!(la.last_update_time, 100);
    assert_eq!(la.load_avg, 0);
    assert_eq!(la.util_avg, 0);
}

#[test]
fn accumulate_running_task_increases_util_and_load() {
    let mut la = LoadAvg::default();
    la.accumulate(1_000_000_000, 1_000_000_000, NICE_0_LOAD, true);
    assert!(la.load_avg > 0);
    assert!(la.util_avg > 0);
    assert_eq!(la.load_sum, 1_000_000_000);
    assert_eq!(la.util_sum, 1_000_000_000);
    assert_eq!(la.last_update_time, 1_000_000_000);
}

#[test]
fn accumulate_idle_task_increases_load_but_not_util() {
    let mut la = LoadAvg::default();
    la.accumulate(1_000_000_000, 1_000_000_000, NICE_0_LOAD, false);
    assert!(la.load_avg > 0);
    assert_eq!(la.util_avg, 0);
    assert_eq!(la.load_sum, 1_000_000_000);
    assert_eq!(la.util_sum, 0);
}

#[test]
fn decay_reduces_load_avg_over_time() {
    let mut la = LoadAvg::default();
    la.accumulate(1_000_000_000, 1_000_000_000, NICE_0_LOAD, true);
    let peak_load = la.load_avg;
    let peak_util = la.util_avg;

    la.accumulate(2_000_000_000, 1_000_000_000, 0, false);
    assert!(la.load_avg < peak_load);
    assert!(la.util_avg < peak_util);
}

#[test]
fn load_avg_never_exceeds_theoretical_maximum() {
    let mut la = LoadAvg::default();
    for i in 0..100 {
        let now = (i + 1) * 1_000_000_000;
        la.accumulate(now, 1_000_000_000, NICE_0_LOAD, true);
    }
    assert!(la.load_avg <= 1024);
    assert!(la.util_avg <= 1024);
}

#[test]
fn higher_weight_produces_higher_load_avg() {
    let mut la_low = LoadAvg::default();
    let mut la_high = LoadAvg::default();
    
    la_low.accumulate(1_000_000_000, 1_000_000_000, 100, true);
    la_high.accumulate(1_000_000_000, 1_000_000_000, 10000, true);
    
    assert!(la_high.load_avg > la_low.load_avg);
}

#[test]
fn saturating_add_prevents_overflow_on_huge_delta() {
    let mut la = LoadAvg::default();
    la.accumulate(u64::MAX, u64::MAX, u64::MAX, true);
    assert_eq!(la.last_update_time, u64::MAX);
    assert!(la.load_sum <= u64::MAX);
}

#[test]
fn alternating_running_and_idle_smooths_util_avg() {
    let mut la = LoadAvg::default();
    let mut time = 0;
    for i in 0..20 {
        time += 1_000_000;
        let running = i % 2 == 0;
        la.accumulate(time, 1_000_000, NICE_0_LOAD, running);
    }
    assert!(la.util_avg > 0);
    assert!(la.util_avg < 1024);
}

#[test]
fn period_contrib_increments_on_every_accumulate() {
    let mut la = LoadAvg::default();
    la.accumulate(100, 10, NICE_0_LOAD, true);
    let c1 = la.period_contrib;
    la.accumulate(200, 10, NICE_0_LOAD, true);
    let c2 = la.period_contrib;
    assert!(c2 > c1);
}

#[test]
fn util_sum_accumulates_only_when_running() {
    let mut la = LoadAvg::default();
    la.accumulate(1000, 500, NICE_0_LOAD, false);
    assert_eq!(la.util_sum, 0);
    la.accumulate(2000, 500, NICE_0_LOAD, true);
    assert_eq!(la.util_sum, 500);
}

#[test]
fn load_sum_accumulates_regardless_of_running_state() {
    let mut la = LoadAvg::default();
    la.accumulate(1000, 300, NICE_0_LOAD, false);
    assert_eq!(la.load_sum, 300);
    la.accumulate(2000, 400, NICE_0_LOAD, true);
    assert_eq!(la.load_sum, 700);
}

#[test]
fn pelt_precision_under_one_millisecond() {
    let mut la = LoadAvg::default();
    la.accumulate(1000, 100, NICE_0_LOAD, true);
    assert!(la.util_avg > 0);
}

#[test]
fn pelt_handles_max_weight_without_panic() {
    let mut la = LoadAvg::default();
    la.accumulate(1_000_000_000, 1_000_000_000, u64::MAX, true);
    assert!(la.load_avg > 0);
}

#[test]
fn consecutive_accumulates_update_last_time_correctly() {
    let mut la = LoadAvg::default();
    let times = [10, 50, 100, 500, 1000, 5000];
    for &t in &times {
        la.accumulate(t, 1, NICE_0_LOAD, true);
        assert_eq!(la.last_update_time, t);
    }
}