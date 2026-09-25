//! Approximate float comparison.
//!
//! `==` on floats is the wrong tool for geometry and DSP: an accumulation of
//! rounding errors means two mathematically equal results can differ by many
//! ULPs. These helpers compare against an absolute/relative tolerance instead.
//!
//! Both `NaN`-safety and the zero case are handled deliberately:
//!
//! - two `NaN`s are **not** approximately equal (they represent distinct
//!   invalid results, and treating them as equal hides real bugs);
//! - an `inf` equals only itself, so infinity never "approximately" matches a
//!   large finite number.

/// Comparison with an explicit tolerance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Epsilon {
    /// Absolute tolerance, used when the values are near zero.
    pub abs: f64,
    /// Relative tolerance, used once the values are large.
    pub rel: f64,
}

impl Epsilon {
    /// A tolerance with both components set to the same value.
    #[inline]
    pub const fn new(value: f64) -> Self {
        Self { abs: value, rel: value }
    }

    /// Split absolute and relative tolerances.
    #[inline]
    pub const fn split(abs: f64, rel: f64) -> Self {
        Self { abs, rel }
    }

    /// A tolerance chosen for single-precision geometry: a few ULP of `1.0`,
    /// which is ~1.19e-7.
    #[inline]
    pub const fn f32() -> Self {
        Self { abs: 1e-5, rel: 1e-5 }
    }

    /// A tolerance chosen for double-precision geometry.
    #[inline]
    pub const fn f64() -> Self {
        Self { abs: 1e-12, rel: 1e-12 }
    }

    /// The tolerance that applies to a pair of values: the larger of the two
    /// criteria, so the test degrades gracefully from absolute to relative.
    #[inline]
    pub fn tolerance_for(&self, a: f64, b: f64) -> f64 {
        let scale = a.abs().max(b.abs());
        self.abs.max(self.rel * scale)
    }
}

impl Default for Epsilon {
    #[inline]
    fn default() -> Self {
        Self::f64()
    }
}

/// Approximate comparison for a float.
pub trait ApproxEq: Copy {
    /// Compare with an explicit tolerance.
    fn approx_eq_eps(self, other: Self, eps: Epsilon) -> bool;

    /// Compare with the type's default tolerance.
    fn approx_eq(self, other: Self) -> bool {
        self.approx_eq_eps(other, Epsilon::default())
    }

    /// Clamp `value` into `[low, high]` and report whether it was already
    /// inside. Used to stop accumulated error from pushing a result past a
    /// bound, which would otherwise flip a comparison.
    fn clamp_checked(self, low: Self, high: Self) -> (Self, bool);
}

macro_rules! impl_approx_for_float {
    ($ty:ty) => {
        impl ApproxEq for $ty {
            #[inline]
            fn approx_eq_eps(self, other: Self, eps: Epsilon) -> bool {
                // NaN never matches, including another NaN.
                if self.is_nan() || other.is_nan() {
                    return false;
                }
                // Infinities match only themselves.
                if self.is_infinite() || other.is_infinite() {
                    return self == other;
                }
                let diff = (self as f64 - other as f64).abs();
                diff <= eps.tolerance_for(self as f64, other as f64)
            }

            #[inline]
            fn clamp_checked(self, low: Self, high: Self) -> (Self, bool) {
                if self < low {
                    (low, false)
                } else if self > high {
                    (high, false)
                } else {
                    (self, true)
                }
            }
        }
    };
}

impl_approx_for_float!(f32);
impl_approx_for_float!(f64);

/// Sign-agnostic linear interpolation.
///
/// Unlike `a + (b - a) * t` this keeps precision when `a` and `b` are far apart
/// in magnitude, because the difference is never formed.
#[inline]
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

/// Sign-agnostic linear interpolation.
#[inline]
pub fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {

    (1.0 - t) * a + t * b
}

/// Map `value` from one range to another, clamped to the output range.
#[inline]
pub fn remap_f32(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let span = in_max - in_min;
    if span == 0.0 {
        return out_min;
    }
    let t = ((value - in_min) / span).clamp(0.0, 1.0);
    out_min + t * (out_max - out_min)
}

/// `true` when `value` lies within `tolerance` of `target`.
#[inline]
pub fn near_f64(value: f64, target: f64, tolerance: f64) -> bool {
    value.approx_eq_eps(target, Epsilon::new(tolerance))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_values_are_equal() {
        assert!(1.0f64.approx_eq(1.0));
        assert!(0.0f32.approx_eq(0.0));
        assert!((-2.5f64).approx_eq(-2.5));
    }

    #[test]
    fn nan_never_compares_equal() {
        // Including against another NaN, which would otherwise hide bugs.
        assert!(!f64::NAN.approx_eq(f64::NAN));
        assert!(!f64::NAN.approx_eq(0.0));
        assert!(!f64::NAN.approx_eq_eps(0.0, Epsilon::new(1000.0)));
    }

    #[test]
    fn infinity_matches_only_itself() {
        assert!(f64::INFINITY.approx_eq(f64::INFINITY));
        assert!(!f64::INFINITY.approx_eq(f64::MAX));
        assert!(!f64::NEG_INFINITY.approx_eq(f64::INFINITY));
    }

    #[test]
    fn relative_tolerance_scales_with_magnitude() {
        let eps = Epsilon::new(1e-9);
        // At 1.0 the relative term is 1e-9, so a 1e-12 difference passes.
        assert!((1.0f64 + 1e-12).approx_eq_eps(1.0, eps));
        // At 1e12 the same *relative* tolerance now allows 1e3, so a 1.0
        // difference still passes - the test is about scaling, not rejection.
        assert!((1e12f64 + 1.0).approx_eq_eps(1e12, eps));
        // A difference beyond the scaled tolerance (1e6 > 1e3) is rejected.
        assert!(!(1e12f64 + 1e6).approx_eq_eps(1e12, eps));
    }

    #[test]
    fn absolute_tolerance_covers_the_near_zero_range() {
        // Both are effectively zero, so only an absolute test is sensible.
        let eps = Epsilon::new(1e-9);
        assert!((1e-12f64).approx_eq_eps(0.0, eps));
        assert!(!(-1e-3f64).approx_eq_eps(0.0, eps));
    }

    #[test]
    fn clamp_reports_whether_a_clamp_happened() {
        assert_eq!((0.5f64).clamp_checked(0.0, 1.0), (0.5, true));
        assert_eq!((1.5f64).clamp_checked(0.0, 1.0), (1.0, false));
        assert_eq!((-1.0f64).clamp_checked(0.0, 1.0), (0.0, false));
    }

    #[test]
    fn lerp_and_remap_behave_at_the_endpoints() {
        assert_eq!(lerp_f64(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp_f64(0.0, 10.0, 1.0), 10.0);
        assert!(lerp_f64(2.0, 4.0, 0.5).approx_eq(3.0));

        assert!(remap_f32(0.5, 0.0, 1.0, 0.0, 100.0).approx_eq(50.0));
        // Out-of-range input clamps rather than extrapolating.
        assert!(remap_f32(2.0, 0.0, 1.0, 0.0, 100.0).approx_eq(100.0));
        assert!(remap_f32(-1.0, 0.0, 1.0, 0.0, 100.0).approx_eq(0.0));
        // A degenerate input range yields the output minimum instead of NaN.
        assert!(remap_f32(5.0, 1.0, 1.0, 7.0, 9.0).approx_eq(7.0));
    }

    #[test]
    fn near_uses_a_single_tolerance() {
        assert!(near_f64(1.0, 1.0 + 1e-10, 1e-9));
        assert!(!near_f64(1.0, 1.1, 1e-9));
    }

    #[test]
    fn split_tolerances_are_independent() {
        let eps = Epsilon::split(0.0, 1e-3);
        // Zero absolute tolerance means only the relative one applies.
        assert!(!eps.abs.eq(&0.0) == false);
        assert!(1.0005f64.approx_eq_eps(1.0, eps));
    }
}
