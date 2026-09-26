//! Turning a noisy fuel-gauge reading into a number a user can trust.
//!
//! # Why this module exists at all
//!
//! A battery gauge does not report charge. It reports an *estimate*, from a
//! model of the cell's internal resistance, corrected by whatever the firmware
//! last observed. The estimate is good to a few percent, and it is not stable:
//! plugging in a charger, waking from suspend, or a load step can move it by
//! twenty points in one sample.
//!
//! Present that raw and the indicator does what every bad battery indicator
//! does - it jumps from 40% to 15% while the user is watching, and they stop
//! believing it. Worse, a low-battery warning that fires and un-fires on
//! consecutive samples is ignored exactly when it matters.
//!
//! So the driver does not forward the raw value. It runs it through three
//! filters, each aimed at one specific way the raw value lies:
//!
//! | Filter | The lie it removes |
//! |--------|--------------------|
//! | clamping | a gauge reporting 140%, or a negative |
//! | slew rate limiting | a 20-point jump between two samples |
//! | exponential averaging | small per-sample jitter that never settles |
//!
//! Plus [`Smoother::is_low`], which has hysteresis so a warning fires once
//! rather than every sample.
//!
//! # All integer, all fixed-point
//!
//! The average is kept in per-mille rather than as a float. This is a driver
//! crate: a float here means an FPU context save on every sample, and the
//! arithmetic is a weighted sum. `update` is allocation free, and a `Smoother`
//! is 20 bytes so a driver can keep one per source without thinking about it.

/// How many parts out of 1000 the average is worth, when a reading arrives.
///
/// `512` is a 1/2 weight: the new sample moves the average halfway towards it.
/// The usual compromise - fast enough that a real discharge is visible within a
/// few samples, slow enough that one bad reading does not move the indicator.
const WEIGHT_PER_MILLE: u32 = 512;

/// The largest change one [`Smoother::update`] may report, in percent.
///
/// A real 60 Hz discharge is about 0.02% per frame. Five is generous enough
/// that a genuine drop from full to empty is not visibly slow, and small enough
/// that no user perceives a single jump.
const DEFAULT_MAX_STEP_PCT: u32 = 5;

/// The highest percentage that means anything.
const MAX_PERCENT: u32 = 100;

/// The highest threshold a low-battery warning may be given.
///
/// One below [`MAX_PERCENT`], because the latch fires on `<=` and a full battery
/// is not a low battery. See [`Smoother::with_limits`].
const MAX_LOW_PERCENT: u32 = 99;

/// Scale from percent to the per-mille domain the average lives in.
const PER_MILLE: u32 = 1000;

/// The default low-battery threshold, as a percentage.
///
/// Fifteen rather than ten: a user warned at 10% on a laptop has already lost
/// the working time they needed, and the warning arrives when the consequence
/// is no longer preventable.
pub const DEFAULT_LOW_PERCENT: u32 = 15;

/// Filters one power source's charge readings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Smoother {
    /// The running average, in per-mille. `None` until the first sample, which
    /// is why the first sample is passed through untouched: averaging against
    /// nothing would invent a number.
    average: Option<u32>,
    /// The value [`Smoother::update`] last returned, in percent.
    ///
    /// Also `None` until the first sample, and for the same reason. It looks
    /// like it could start at zero, but the slew limiter would then treat the
    /// first real reading as a step change from zero and report `max_step`
    /// instead - so a fresh gauge would read 5% while reporting 63%.
    last: Option<u32>,
    /// Ceiling on the change per sample.
    max_step: u32,
    /// The percentage at or below which [`Smoother::is_low`] becomes true.
    low_percent: u32,
    /// Whether the low warning is currently *asserted*.
    ///
    /// Latched, so a reading that dips to the threshold and comes straight back
    /// fires the warning once rather than twice.
    low_armed: bool,
}

impl Smoother {
    /// A smoother with the default limits.
    #[inline]
    pub const fn new() -> Self {
        Self {
            average: None,
            last: None,
            max_step: DEFAULT_MAX_STEP_PCT,
            low_percent: DEFAULT_LOW_PERCENT,
            low_armed: false,
        }
    }

    /// A smoother with an explicit slew limit and low-battery threshold.
    ///
    /// Both are clamped rather than rejected, because the caller is a driver
    /// reading firmware and a firmware value of 200 should not be able to build
    /// a dead gauge. A `max_step` of zero would freeze the indicator, so it
    /// becomes 1.
    ///
    /// The threshold is capped at **99, not 100**, and that distinction is the
    /// whole reason this is a clamp rather than a `min`. The latch fires on
    /// `value <= threshold`, so a threshold clamped to 100 is met by a full
    /// battery: the indicator would announce "low" on a laptop that just finished
    /// charging. 100% is not a low battery under any threshold, so the effective
    /// ceiling is 99.
    #[inline]
    pub const fn with_limits(max_step: u32, low_percent: u32) -> Self {
        Self {
            average: None,
            last: None,
            max_step: if max_step == 0 { 1 } else { max_step },
            low_percent: if low_percent > MAX_LOW_PERCENT { MAX_LOW_PERCENT } else { low_percent },
            low_armed: false,
        }
    }

    /// Feed one raw reading, and get back the value to show a user.
    ///
    /// `raw` is the gauge's own percentage. Out-of-range values are clamped
    /// rather than rejected: a gauge reporting 140% is telling the truth about
    /// *its* model, and the driver's job is to present something sane, not to
    /// declare the hardware broken.
    pub fn update(&mut self, raw: u32) -> u32 {
        let raw = raw.min(MAX_PERCENT);
        let raw_mille = raw * PER_MILLE;

        let target = match self.average {
            // First sample: there is nothing to average against, so take it as
            // the truth and start the average from here.
            None => raw_mille,
            Some(avg) => {
                // `avg + (raw - avg) * w / 1000`, in unsigned arithmetic.
                //
                // The sign of `raw - avg` is the trap here. Subtracting in `u32`
                // and multiplying wraps, so the difference is computed signed
                // first. A wrapped weighted average produces a plausible number
                // that is wrong, which is the worst kind of wrong for a value a
                // user is looking at.
                let diff = raw_mille as i64 - avg as i64;
                (avg as i64 + diff * WEIGHT_PER_MILLE as i64 / PER_MILLE as i64) as u32
            }
        };
        self.average = Some(target);

        // Slew limit. The average has already moved most of the way; this stops
        // the last few percent of that movement from being visible.
        //
        // The first sample skips the limiter entirely, for the same reason the
        // average starts from it: there is no previous value to limit a change
        // from, and applying the limit anyway would report `max_step` for a
        // gauge that is reporting its true charge.
        let candidate = target / PER_MILLE;
        self.last = Some(match self.last {
            None => candidate,
            Some(previous) => {
                let step = if candidate > previous {
                    candidate - previous
                } else {
                    previous - candidate
                };
                if step <= self.max_step {
                    candidate
                } else if candidate > previous {
                    previous + self.max_step
                } else {
                    previous.saturating_sub(self.max_step)
                }
            }
        });

        self.rearm_low();
        self.last.unwrap_or(0)
    }

    /// The last value returned by [`Smoother::update`], or 0 before the first.
    #[inline]
    pub fn value(&self) -> u32 {
        self.last.unwrap_or(0)
    }

    /// How far above the threshold the value must recover before the warning
    /// can fire again.
    ///
    /// Four points, not one: one point is smaller than the gauge's own error, so
    /// a one-point rearm would strobe as badly as no hysteresis at all.
    pub const HYSTERESIS: u32 = 4;

    /// Has the smoothed value fallen to or below the threshold?
    ///
    /// Latched: once true it stays true until the value recovers above
    /// `threshold + HYSTERESIS`.
    #[inline]
    pub const fn is_low(&self) -> bool {
        self.low_armed
    }

    /// Recompute the low-battery latch from the current value.
    fn rearm_low(&mut self) {
        let current = self.last.unwrap_or(0);
        if current <= self.low_percent {
            self.low_armed = true;
        } else if current >= self.low_percent.saturating_add(Self::HYSTERESIS) {
            self.low_armed = false;
        }
    }

    /// Forget the average, keeping the configured limits.
    ///
    /// For a hot-unplugged pack or a firmware reload: the average models *this*
    /// pack's state, and carrying it across a different pack is how an indicator
    /// shows 3% on a freshly inserted full battery.
    pub fn reset(&mut self) {
        let max_step = self.max_step;
        let low_percent = self.low_percent;
        *self = Self { max_step, low_percent, ..Self::new() };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_reading_is_taken_at_face_value() {
        // Averaging against nothing would invent a number, and inventing a
        // number about a battery is not a small thing.
        let mut s = Smoother::new();
        assert_eq!(s.update(63), 63);
        assert_eq!(s.value(), 63);
    }

    #[test]
    fn an_out_of_range_reading_is_clamped_rather_than_escaped() {
        let mut s = Smoother::new();
        assert_eq!(s.update(140), 100, "a gauge reporting 140% must not reach a user");
        let mut s = Smoother::new();
        assert_eq!(s.update(0), 0);
    }

    #[test]
    fn one_sample_cannot_move_the_indicator_far() {
        // The whole point: a 20-point jump must arrive as a series of small steps.
        let mut s = Smoother::with_limits(5, 15);
        s.update(90);
        let first = s.update(40);
        assert!(first >= 85, "a 50-point drop must not be shown, got {first}");

        let mut previous = first;
        for _ in 0..4 {
            let next = s.update(40);
            assert!(previous - next <= 5, "slew limit violated: {previous} -> {next}");
            previous = next;
        }
    }

    #[test]
    fn a_discharge_eventually_gets_through() {
        // A filter that hides a real change is as broken as one that shows noise.
        let mut s = Smoother::new();
        s.update(100);
        for _ in 0..200 {
            s.update(10);
        }
        assert_eq!(s.value(), 10, "a sustained real change must converge");
    }

    #[test]
    fn noise_around_a_true_value_settles_near_that_value() {
        let mut s = Smoother::new();
        s.update(50);
        // A realistic noisy sequence hovering at 50, +/- 3.
        for raw in [47, 52, 48, 53, 49, 51, 48, 52, 50, 47] {
            s.update(raw);
        }
        let v = s.value();
        assert!((45..=55).contains(&v), "the average should sit near the truth, got {v}");
    }

    #[test]
    fn a_deep_discharge_converges_faster_than_a_shallow_one() {
        // Sanity check on the weighting: if a 90-point drop and a 1-point drop
        // moved the average equally, the weight would be doing nothing.
        let mut big = Smoother::new();
        let mut small = Smoother::new();
        big.update(100);
        small.update(50);
        for _ in 0..10 {
            big.update(10);
            small.update(49);
        }
        assert!(
            100 - big.value() > 50 - small.value(),
            "big drop moved {}, small moved {}",
            100 - big.value(),
            50 - small.value()
        );
    }

    #[test]
    fn the_low_warning_fires_only_below_the_threshold() {
        let mut s = Smoother::new();
        s.update(80);
        assert!(!s.is_low());
        for _ in 0..100 {
            s.update(5);
        }
        assert!(s.is_low(), "a genuinely low battery must raise the warning");
    }

    #[test]
    fn a_gauge_sitting_on_the_threshold_does_not_strobe() {
        // The failure this prevents: a warning that fires and un-fires every
        // sample is a warning that gets ignored.
        let mut s = Smoother::with_limits(5, 20);
        s.update(21);
        for _ in 0..4 {
            s.update(20);
        }
        assert!(s.is_low(), "it should have fired at 20%");
        for _ in 0..40 {
            s.update(20);
        }
        assert!(s.is_low(), "and it must stay asserted while the value holds");
    }

    #[test]
    fn the_low_warning_rearms_only_after_a_real_recovery() {
        let mut s = Smoother::with_limits(5, 20);
        s.update(20);
        for _ in 0..4 {
            s.update(20);
        }
        assert!(s.is_low());

        // Recovering to exactly the threshold plus the hysteresis band is not a
        // rearm: a gauge that hovers there would strobe.
        for _ in 0..10 {
            s.update(23);
        }
        assert!(
            s.is_low(),
            "a recovery inside the hysteresis band must not rearm, or the \
             warning strobes"
        );

        for _ in 0..100 {
            s.update(95);
        }
        assert!(!s.is_low(), "a real recovery must rearm the warning");
    }

    #[test]
    fn charging_clears_the_warning_the_way_it_should() {
        let mut s = Smoother::new();
        for _ in 0..100 {
            s.update(3);
        }
        assert!(s.is_low());
        for _ in 0..200 {
            s.update(100);
        }
        assert!(!s.is_low());
    }

    #[test]
    fn a_zero_slew_limit_is_clamped_rather_than_freezing_the_gauge() {
        // A driver reading firmware must not be able to build a dead indicator.
        let mut s = Smoother::with_limits(0, 15);
        s.update(50);
        for _ in 0..100 {
            s.update(90);
        }
        assert!(s.value() > 50, "a zero limit must become 1, not freeze at 50");
    }

    #[test]
    fn a_threshold_above_one_hundred_is_clamped() {
        let mut s = Smoother::with_limits(5, 200);
        for _ in 0..100 {
            s.update(100);
        }
        assert!(!s.is_low(), "an unreachable threshold must not read as met");
    }

    #[test]
    fn reset_forgets_the_average_so_a_new_pack_starts_fresh() {
        let mut s = Smoother::new();
        for _ in 0..100 {
            s.update(3);
        }
        assert_eq!(s.value(), 3);
        s.reset();
        assert_eq!(s.value(), 0, "reset clears the state");
        assert_eq!(s.update(97), 97, "a fresh pack reads at its true value");
    }

    #[test]
    fn reset_keeps_the_configured_limits() {
        // A reset that restored the defaults would quietly change the driver's
        // behaviour, which is the sort of thing nobody notices for a month.
        let mut s = Smoother::with_limits(2, 30);
        s.reset();
        s.update(50);
        for _ in 0..100 {
            s.update(10);
        }
        assert!((8..=10).contains(&s.value()), "got {}", s.value());
    }
}

