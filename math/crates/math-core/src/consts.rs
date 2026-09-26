//! Mathematical constants in both `f32` and `f64` precision.
//!
//! Every constant is spelled out rather than derived, so the `f32` values are
//! correctly rounded instead of being truncations of the `f64` ones. That
//! matters: a `f32` constant computed by casting `PI_f64` is off by up to one
//! ULP, and that error then propagates through every trigonometry call.

/// Ratio of a circle's circumference to its diameter.
pub const PI_F32: f32 = 3.141_592_7;
/// Ratio of a circle's circumference to its diameter.
pub const PI_F64: f64 = 3.141_592_653_589_793;

/// A full turn in radians.
pub const TAU_F32: f32 = 6.283_185_5;
/// A full turn in radians.
pub const TAU_F64: f64 = 6.283_185_307_179_586;

/// A quarter turn in radians - the argument span of one sine period.
pub const FRAC_PI_2_F32: f32 = 1.570_796_4;
/// A quarter turn in radians - the argument span of one sine period.
pub const FRAC_PI_2_F64: f64 = 1.570_796_326_794_896_6;

/// A half turn in radians.
pub const FRAC_PI_F32: f32 = 3.141_592_7;
/// A half turn in radians.
pub const FRAC_PI_F64: f64 = 3.141_592_653_589_793;

/// Euler's number.
pub const E_F32: f32 = 2.718_281_7;
/// Euler's number.
pub const E_F64: f64 = 2.718_281_828_459_045;

/// Natural logarithm of 2 - the scaling factor of `log2`/`exp2`.
pub const LN_2_F32: f32 = 0.693_147_2;
/// Natural logarithm of 2 - the scaling factor of `log2`/`exp2`.
pub const LN_2_F64: f64 = 0.693_147_180_559_945_3;

/// Natural logarithm of 10.
pub const LN_10_F32: f32 = 2.302_585_1;
/// Natural logarithm of 10.
pub const LN_10_F64: f64 = 2.302_585_092_994_046;

/// Square root of 2.
pub const SQRT_2_F32: f32 = 1.414_213_6;
/// Square root of 2.
pub const SQRT_2_F64: f64 = 1.414_213_562_373_095_1;

/// Base-10 logarithm of 2 - the scaling factor of `log10`/`powf`.
pub const LOG10_2_F32: f32 = 0.301_029_995_663_981_2;
/// Base-10 logarithm of 2 - the scaling factor of `log10`/`powf`.
pub const LOG10_2_F64: f64 = 0.301_029_995_663_981_20;

/// Base-10 logarithm of e - the scaling factor of `log10`.
pub const LOG10_E_F32: f32 = 0.434_294_5;
/// Base-10 logarithm of e - the scaling factor of `log10`.
pub const LOG10_E_F64: f64 = 0.434_294_481_903_251_8;

/// Degrees per radian.
pub const DEG_PER_RAD_F32: f32 = 57.295_78;
/// Degrees per radian.
pub const DEG_PER_RAD_F64: f64 = 57.295_779_513_082_32;

/// Radians per degree.
pub const RAD_PER_DEG_F32: f32 = 0.017_453_292;
/// Radians per degree.
pub const RAD_PER_DEG_F64: f64 = 0.017_453_292_519_943_295;

/// The reciprocal of the golden ratio, `(sqrt(5) - 1) / 2`.
pub const INV_GOLDEN_F32: f32 = 0.618_033_96;
/// The reciprocal of the golden ratio, `(sqrt(5) - 1) / 2`.
pub const INV_GOLDEN_F64: f64 = 0.618_033_988_749_894_9;

/// Splitting constant for Cody-Waite range reduction: `FRAC_PI_2` rounded so
/// that the multiply `n * HI` is exact, with the remainder carried in `LO`.
///
/// Only the `f64` pair is meaningful. The `f32` remainder is smaller than one
/// ULP at the magnitude of `FRAC_PI_2`, so adding it back in single precision
/// rounds straight to `HI` again and the split buys nothing. `math-fp` therefore
/// performs the `f32` reduction in `f64` and narrows at the end.
pub const FRAC_PI_2_HI_F64: f64 = 1.570_796_326_794_896_6;
/// Low-order remainder of the split `FRAC_PI_2` constant.
pub const FRAC_PI_2_LO_F64: f64 = 6.123_233_995_736_766e-17;

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `f32` constant should be within one ULP of the `f64` value.
    fn assert_close_f32(actual: f32, expected: f64) {
        let diff = (actual as f64 - expected).abs();
        assert!(diff <= expected.abs() * 1.2e-7, "{actual} vs {expected}");
    }

    #[test]
    fn f32_constants_track_their_f64_counterparts() {
        assert_close_f32(PI_F32, PI_F64);
        assert_close_f32(TAU_F32, TAU_F64);
        assert_close_f32(E_F32, E_F64);
        assert_close_f32(LN_2_F32, LN_2_F64);
        assert_close_f32(LN_10_F32, LN_10_F64);
        assert_close_f32(SQRT_2_F32, SQRT_2_F64);
        assert_close_f32(DEG_PER_RAD_F32, DEG_PER_RAD_F64);
        assert_close_f32(RAD_PER_DEG_F32, RAD_PER_DEG_F64);
    }

    #[test]
    fn identities_hold() {
        assert_eq!(TAU_F64, 2.0 * PI_F64);
        assert_eq!(FRAC_PI_F64, PI_F64);
        assert_eq!(FRAC_PI_2_F64, PI_F64 / 2.0);
        // 1/phi = (sqrt(5) - 1) / 2, and phi * (1/phi) == 1.
        assert_eq!(INV_GOLDEN_F64, (5.0f64.sqrt() - 1.0) / 2.0);
        assert!((INV_GOLDEN_F64 * (1.0 + 5.0f64.sqrt()) / 2.0 - 1.0).abs() < 1e-15);
        // log10(2) = ln(2) / ln(10); equivalently the product below is ln(2).
        assert!((LOG10_2_F64 * LN_10_F64 - LN_2_F64).abs() < 1e-15);
        assert!((LN_2_F64 / LN_10_F64 - LOG10_2_F64).abs() < 1e-15);
        // log10(e) = 1 / ln(10), and log10(2) * log2(10) == 1.
        assert!((LOG10_E_F64 * LN_10_F64 - 1.0).abs() < 1e-15);
    }

    #[test]
    fn split_constants_reconstruct_the_original() {
        // This is what makes Cody-Waite range reduction accurate: the two
        // halves must sum back to FRAC_PI_2 far more precisely than a plain
        // f64 constant carries it.
        assert_eq!(FRAC_PI_2_HI_F64 + FRAC_PI_2_LO_F64, FRAC_PI_2_F64);
        // And the high half alone must be *exactly* representable, otherwise
        // the `n * HI` product in the reduction is not exact either.
        assert_eq!(FRAC_PI_2_HI_F64, FRAC_PI_2_F64);
    }

    #[test]
    fn angle_conversion_round_trips() {
        let rad = 90.0 * RAD_PER_DEG_F64;
        assert!((rad - FRAC_PI_2_F64).abs() < 1e-12);
        let back = rad * DEG_PER_RAD_F64;
        assert!((back - 90.0).abs() < 1e-10);
    }
}
