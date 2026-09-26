//! Float classification and field extraction, without `std`.
//!
//! Rust's `f32` / `f64` expose the primitives we need (`is_nan`, `is_infinite`,
//! `to_bits`, `from_bits`) but nothing for pulling apart the encoding. The
//! higher-level libraries need that: [`math-fp`](../../math-fp) reads the
//! exponent to implement `exp2`, and the DSP code reads the significand for
//! fixed-point conversion.
//!
//! Everything here is built on the safe `to_bits` / `from_bits` pair, so the
//! module needs no `unsafe` and the results are exactly reproducible.

use core::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr};

/// Sign of a float.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sign {
    Positive,
    Negative,
}

impl Sign {
    /// `+1.0` or `-1.0`.
    #[inline]
    pub const fn to_f64(self) -> f64 {
        match self {
            Self::Positive => 1.0,
            Self::Negative => -1.0,
        }
    }

    /// Flip the sign of `magnitude`.
    #[inline]
    pub const fn apply(self, magnitude: f64) -> f64 {
        match self {
            Self::Positive => magnitude,
            Self::Negative => -magnitude,
        }
    }
}

/// A float type whose encoding we can read and rebuild.
pub trait FloatBits: Copy {
    /// The unsigned integer with the same width as this float's encoding.
    type Bits: Copy
        + Ord
        + BitAnd<Output = Self::Bits>
        + BitOr<Output = Self::Bits>
        + BitXor<Output = Self::Bits>
        + Not<Output = Self::Bits>
        + Shl<u32, Output = Self::Bits>
        + Shr<u32, Output = Self::Bits>;

    /// Width of the significand field, excluding the implicit leading bit.
    const MANTISSA_BITS: u32;
    /// Width of the exponent field.
    const EXPONENT_BITS: u32;
    /// Bit pattern of a positive zero.
    const ZERO_BITS: Self::Bits;
    /// The sign bit mask.
    const SIGN_MASK: Self::Bits;
    /// Mask covering the magnitude (exponent plus significand).
    const MAGNITUDE_MASK: Self::Bits;
    /// Mask covering the significand field.
    const MANTISSA_MASK: Self::Bits;
    /// Mask covering the exponent field.
    const EXPONENT_MASK: Self::Bits;

    /// The raw encoding.
    fn to_bits(self) -> Self::Bits;

    /// Rebuild a float from its encoding.
    fn from_bits(bits: Self::Bits) -> Self;
}

impl FloatBits for f32 {
    type Bits = u32;

    const MANTISSA_BITS: u32 = 23;
    const EXPONENT_BITS: u32 = 8;
    const ZERO_BITS: u32 = 0;
    const SIGN_MASK: u32 = 1 << 31;
    const MAGNITUDE_MASK: u32 = 0x7FFF_FFFF;
    const MANTISSA_MASK: u32 = 0x007F_FFFF;
    const EXPONENT_MASK: u32 = 0x7F80_0000;

    #[inline]
    fn to_bits(self) -> u32 {
        self.to_bits()
    }

    #[inline]
    fn from_bits(bits: u32) -> Self {
        f32::from_bits(bits)
    }
}

impl FloatBits for f64 {
    type Bits = u64;

    const MANTISSA_BITS: u32 = 52;
    const EXPONENT_BITS: u32 = 11;
    const ZERO_BITS: u64 = 0;
    const SIGN_MASK: u64 = 1 << 63;
    const MAGNITUDE_MASK: u64 = 0x7FFF_FFFF_FFFF_FFFF;
    const MANTISSA_MASK: u64 = 0x000F_FFFF_FFFF_FFFF;
    const EXPONENT_MASK: u64 = 0x7FF0_0000_0000_0000;

    #[inline]
    fn to_bits(self) -> u64 {
        self.to_bits()
    }

    #[inline]
    fn from_bits(bits: u64) -> Self {
        f64::from_bits(bits)
    }
}

/// Sign of `value`; `-0.0` counts as negative.
#[inline]
pub fn sign<T: FloatBits>(value: T) -> Sign {
    if (value.to_bits() & T::SIGN_MASK) != T::ZERO_BITS {
        Sign::Negative
    } else {
        Sign::Positive
    }
}

/// The stored exponent field, still biased. `NaN` and infinity yield their
/// saturated field value.
#[inline]
pub fn exponent<T: FloatBits>(value: T) -> T::Bits {
    (value.to_bits() & T::EXPONENT_MASK) >> T::MANTISSA_BITS
}

/// True for `NaN`.
#[inline]
pub fn is_nan<T: FloatBits>(value: T) -> bool {
    (value.to_bits() & T::EXPONENT_MASK) == T::EXPONENT_MASK
        && (value.to_bits() & T::MANTISSA_MASK) != T::ZERO_BITS
}

/// True for `+inf` and `-inf`.
#[inline]
pub fn is_infinite<T: FloatBits>(value: T) -> bool {
    (value.to_bits() & T::EXPONENT_MASK) == T::EXPONENT_MASK
        && (value.to_bits() & T::MANTISSA_MASK) == T::ZERO_BITS
}

/// True for everything that is neither infinite nor `NaN` (zero, subnormal and
/// normal included).
#[inline]
pub fn is_finite<T: FloatBits>(value: T) -> bool {
    (value.to_bits() & T::EXPONENT_MASK) != T::EXPONENT_MASK
}

/// True for a normal value: finite with a non-zero exponent field.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_matches_std_semantics() {
        assert!(is_nan(f64::NAN));
        assert!(!is_nan(0.0));
        assert!(!is_nan(f64::INFINITY));

        assert!(is_infinite(f64::INFINITY));
        assert!(is_infinite(f64::NEG_INFINITY));
        assert!(!is_infinite(f64::NAN));
        assert!(!is_infinite(f64::MAX));

        assert!(is_finite(0.0));
        assert!(is_finite(f64::MAX));
        assert!(!is_finite(f64::NEG_INFINITY));
        assert!(!is_finite(f64::NAN));
    }

    #[test]
    fn normals_and_subnormals_are_distinguished() {
        assert!(is_normal(1.0));
        assert!(!is_normal(0.0));
        assert!(!is_normal(f64::INFINITY));

        // The smallest positive subnormal for f64 is a single mantissa bit.
        let tiny = f64::from_bits(1);
        assert!(is_subnormal(tiny));
        assert!(!is_subnormal(0.0));
        assert!(!is_subnormal(1.0));
    }

    #[test]
    fn negative_zero_keeps_its_sign_bit() {
        assert_eq!(sign(f64::from_bits(0)), Sign::Positive);
        assert_eq!(sign(negative_zero::<f64>()), Sign::Negative);
        assert_eq!(sign(-1.0), Sign::Negative);
        // Numerically still zero, so arithmetic cannot tell it from +0.0.
        assert_eq!(negative_zero::<f64>() + 0.0, 0.0);
    }

    #[test]
    fn exponent_field_is_the_biased_value() {
        // f64 stores 1023 for 1.0, f32 stores 127.
        assert_eq!(exponent(1.0f64), 1023);
        assert_eq!(exponent(2.0f64), 1024);
        assert_eq!(exponent(1.0f32), 127);
        assert_eq!(exponent(f64::NAN), 2047);
    }

    #[test]
    fn magnitude_clears_the_sign() {
        assert_eq!(magnitude(-3.5), 3.5);
        assert_eq!(magnitude(3.5), 3.5);
        // Clearing the sign of -0.0 yields +0.0, not a NaN pattern.
        assert!(!is_nan(magnitude(negative_zero::<f64>())));
        assert_eq!(magnitude(negative_zero::<f64>()), 0.0);
    }

    #[test]
    fn sign_applies_to_magnitudes() {
        assert_eq!(Sign::Positive.apply(2.0), 2.0);
        assert_eq!(Sign::Negative.apply(2.0), -2.0);
        assert_eq!(Sign::Positive.to_f64(), 1.0);
        assert_eq!(Sign::Negative.to_f64(), -1.0);
    }
}

#[inline]
pub fn is_normal<T: FloatBits>(value: T) -> bool {
    let biased = value.to_bits() & T::EXPONENT_MASK;
    biased != T::EXPONENT_MASK && biased != T::ZERO_BITS
}

/// True for a subnormal: finite, non-zero, all-zero exponent field.
#[inline]
pub fn is_subnormal<T: FloatBits>(value: T) -> bool {
    let biased = value.to_bits() & T::EXPONENT_MASK;
    biased == T::ZERO_BITS && (value.to_bits() & T::MAGNITUDE_MASK) != T::ZERO_BITS
}

/// Positive zero.
#[inline]
pub fn positive_zero<T: FloatBits>() -> T {
    T::from_bits(T::ZERO_BITS)
}

/// Negative zero.
#[inline]
pub fn negative_zero<T: FloatBits>() -> T {
    T::from_bits(T::SIGN_MASK)
}

/// The absolute value: the encoding with the sign bit cleared.
#[inline]
pub fn magnitude<T: FloatBits>(value: T) -> T {
    T::from_bits(value.to_bits() & T::MAGNITUDE_MASK)
}
