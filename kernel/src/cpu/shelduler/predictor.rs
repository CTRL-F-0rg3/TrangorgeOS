use super::policy::{Class, Priority};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BurstClass {
    Tiny,
    Short,
    Medium,
    Long,
    Blocking,
    Unknown,
}
#[derive(Clone, Copy, Debug)]
pub struct Prediction {
    pub mean_ns: u64,
    pub variance_ns: u64,
    pub confidence: u8,
    pub class: BurstClass,
}
#[derive(Clone, Copy, Debug)]
pub struct BurstPredictor {
    mean: u64,
    variance: u64,
    samples: u32,
    last_burst: u64,
    blocking_score: u32,
}
impl BurstPredictor {
    pub const fn new() -> Self {
        Self {
            mean: 1_000_000,
            variance: 1_000_000,
            samples: 0,
            last_burst: 0,
            blocking_score: 0,
        }
    }
    pub fn observe_run(&mut self, burst: u64) {
        let burst = burst.max(1);
        if self.samples == 0 {
            self.mean = burst;
            self.variance = burst / 2;
        } else {
            let delta = burst as i128 - self.mean as i128;
            self.mean = (self.mean as i128 + delta / 4).max(1) as u64;
            let dev = delta.unsigned_abs() as u64;
            self.variance = (self.variance.saturating_mul(3) / 4).saturating_add(dev / 4);
        }
        self.last_burst = burst;
        self.samples = self.samples.saturating_add(1);
    }
    pub fn on_wakeup(&mut self) {
        self.blocking_score = self.blocking_score.saturating_add(1).min(100);
    }
    pub fn mark_nonblocking(&mut self) {
        self.blocking_score = self.blocking_score.saturating_sub(1);
    }
    pub fn predict(&self) -> Prediction {
        let class = if self.blocking_score > 8 {
            BurstClass::Blocking
        } else if self.mean < 500_000 {
            BurstClass::Tiny
        } else if self.mean < 2_000_000 {
            BurstClass::Short
        } else if self.mean < 10_000_000 {
            BurstClass::Medium
        } else {
            BurstClass::Long
        };
        Prediction {
            mean_ns: self.mean,
            variance_ns: self.variance,
            confidence: (self.samples.min(255)) as u8,
            class,
        }
    }
    pub fn quantum(&self, base: u64, class: Class, p: Priority) -> u64 {
        let x = self.predict();
        let mut q = base;
        q = match x.class {
            BurstClass::Tiny => q / 2,
            BurstClass::Short => q * 3 / 4,
            BurstClass::Long => q * 2,
            _ => q,
        };
        q = match class {
            Class::RealTime => q / 2,
            Class::Interactive => q * 3 / 4,
            Class::Background => q * 2,
            _ => q,
        };
        q.saturating_add((p.value() as u64) * 50_000)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn learns_short_bursts() {
        let mut p = BurstPredictor::new();
        for _ in 0..10 {
            p.observe_run(100_000);
        }
        assert_eq!(p.predict().class, BurstClass::Tiny);
    }
}
