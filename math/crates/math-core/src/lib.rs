//! Shared substrate for the TrangorgeOS math stack.
//!
//! Every crate under `math/` depends on this one. It exists so the higher-level
//! libraries do not each re-derive the same five things:
//!
//! - **Float bit utilities** ([`bits`]) - classification and field extraction
//!   without `std`, built on the safe `to_bits` / `from_bits` pair.
//! - **Constants** ([`consts`]) - shared in `f32` and `f64` precision.
//! - **Approximate comparison** ([`approx`]) - the epsilon comparisons every
//!   geometry and DSP crate needs.
//! - **Error vocabulary** ([`error`]) - mapped onto `kapi_abi::Status`, so a
//!   math failure travels through the system like any other error.
//! - **Target capabilities** ([`feature`]) - what the current build target
//!   actually offers (FPU, SIMD width), decided at compile time.
//!
//! # No `unsafe`
//!
//! This crate is `#![forbid(unsafe_code)]`, and so is every crate in the
//! workspace (enforced through `[workspace.lints]`). Float bit manipulation is
//! done with the safe `to_bits` / `from_bits` pair, so the whole math stack
//! stays auditable: every result is a function of its input, with no
//! platform-specific escape hatch hiding a rounding decision.

#![no_std]
#![forbid(unsafe_code)]

pub mod approx;
pub mod bits;
pub mod consts;
pub mod error;
pub mod feature;

pub use approx::{ApproxEq, Epsilon};
pub use bits::{FloatBits, Sign, exponent, is_finite, is_infinite, is_nan, is_normal, is_subnormal};
pub use consts::*;
pub use error::MathError;
pub use feature::{FloatWidth, SimdWidth, TargetFeatures};

/// Floating-point types the stack operates on.
///
/// Width-related metadata (`MANTISSA_BITS`, `EXPONENT_BITS`) is **not** repeated
/// here: it is inherited from [`FloatBits`], so there is exactly one definition
/// per float and no ambiguity at the call site.
pub trait Float: FloatBits + ApproxEq + core::fmt::Debug {
    /// Additive identity.
    const ZERO: Self;
    /// Multiplicative identity.
    const ONE: Self;
    /// Largest finite value.
    const MAX: Self;
    /// Machine epsilon for this precision.
    const EPSILON: Self;
    /// Positive infinity.
    const INFINITY: Self;
    /// Not-a-number.
    const NAN: Self;

    /// Narrowing conversion from `f64`.
    fn from_f64(value: f64) -> Self;
    /// Widening conversion to `f64`.
    fn to_f64(self) -> f64;
}

impl Float for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const MAX: Self = f32::MAX;
    const EPSILON: Self = f32::EPSILON;
    const INFINITY: Self = f32::INFINITY;
    const NAN: Self = f32::NAN;

    #[inline]
    fn from_f64(value: f64) -> Self {
        value as f32
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl Float for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const MAX: Self = f64::MAX;
    const EPSILON: Self = f64::EPSILON;
    const INFINITY: Self = f64::INFINITY;
    const NAN: Self = f64::NAN;

    #[inline]
    fn from_f64(value: f64) -> Self {
        value
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_metadata_matches_the_target() {
        assert_eq!(f32::MANTISSA_BITS, 23);
        assert_eq!(f64::MANTISSA_BITS, 52);
        assert_eq!(f32::EXPONENT_BITS, 8);
        assert_eq!(f64::EXPONENT_BITS, 11);
    }

    #[test]
    fn constants_have_the_expected_identity_semantics() {
        assert_eq!(f64::ZERO, 0.0);
        assert_eq!(f64::ONE, 1.0);
        assert!(f64::MAX.is_finite());
        assert!(!f64::INFINITY.is_finite());
        assert!(f64::NAN.is_nan());
        // The trait and the inherent `f64::NAN` must agree.
        assert!(<f64 as Float>::NAN.is_nan());
    }

    #[test]
    fn conversions_round_trip_within_precision() {
        let value = 0.5f64;
        assert_eq!(f32::from_f64(value).to_f64(), value);
        assert_eq!(f64::from_f64(1.25).to_f64(), 1.25);
    }
}
