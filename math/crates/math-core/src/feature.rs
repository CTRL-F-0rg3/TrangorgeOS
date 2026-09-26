//! What the current build target actually offers.
//!
//! The math stack is used from very different places: a userspace compositor
//! with a full FPU and AVX, a kernel built for `x86_64-unknown-none` with SSE
//! only, a RISC-V core with no FPU at all. Code that hardcodes "assume hardware
//! `sqrt`" or "assume 4-wide SIMD" is correct on exactly one of those targets.
//!
//! Everything here is decided at **compile time** from `cfg`, so the cost is
//! zero at runtime and there is no feature-detection dance at boot. A kernel
//! that wants runtime detection can gate on the *possibilities* reported here
//! and verify them once during init.

/// Width of the floating-point unit the target is built for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatWidth {
    /// No hardware FPU: all arithmetic must go through soft-float.
    None,
    /// 32-bit only.
    Single,
    /// 64-bit capable.
    Double,
}

impl FloatWidth {
    /// The largest significand width the target handles in hardware.
    pub const fn max_precision(self) -> Option<u64> {
        match self {
            Self::None => None,
            Self::Single => Some(23),
            Self::Double => Some(52),
        }
    }

    /// `true` when the target can execute `f64` arithmetic natively.
    pub const fn has_double(self) -> bool {
        matches!(self, Self::Double)
    }
}

/// SIMD register width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimdWidth {
    /// No SIMD.
    None,
    /// 64-bit.
    W64,
    /// 128-bit (SSE2 / NEON).
    W128,
    /// 256-bit (AVX).
    W256,
    /// 512-bit (AVX-512).
    W512,
}

impl SimdWidth {
    /// The width in bits.
    pub const fn bits(self) -> u32 {
        match self {
            Self::None => 0,
            Self::W64 => 64,
            Self::W128 => 128,
            Self::W256 => 256,
            Self::W512 => 512,
        }
    }
}

/// The capabilities this build targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetFeatures {
    /// Floating-point unit width.
    pub float: FloatWidth,
    /// SIMD register width.
    pub simd: SimdWidth,
    /// `true` when the target has an integer multiply-accumulate, which the
    /// fixed-point and dot-product paths use heavily.
    pub has_mac: bool,
}

impl TargetFeatures {
    /// The capabilities of the current build.
    pub const fn current() -> Self {
        Self { float: float_width(), simd: simd_width(), has_mac: has_mac() }
    }

    /// A short description, for boot logs and capability reporting.
    pub const fn describe(self) -> &'static str {
        match (self.float, self.simd) {
            (FloatWidth::None, SimdWidth::None) => "soft-float, no simd",
            (FloatWidth::None, _) => "soft-float, simd",
            (FloatWidth::Single, SimdWidth::None) => "fpu32, no simd",
            (FloatWidth::Single, _) => "fpu32, simd",
            (FloatWidth::Double, SimdWidth::None) => "fpu64, no simd",
            (FloatWidth::Double, SimdWidth::W64) => "fpu64, simd64",
            (FloatWidth::Double, SimdWidth::W128) => "fpu64, simd128",
            (FloatWidth::Double, SimdWidth::W256) => "fpu64, simd256",
            (FloatWidth::Double, SimdWidth::W512) => "fpu64, simd512",
        }
    }
}

// One const per architecture rather than a chain of `#[cfg]` blocks: an
// architecture not listed below fails to compile, which is the behaviour we
// want. Guessing a new target's FPU width would be worse than a build error.

// ── FPU width ───────────────────────────────────────────────────────
#[cfg(target_arch = "x86_64")]
const FLOAT_WIDTH: FloatWidth = FloatWidth::Double;
#[cfg(target_arch = "aarch64")]
const FLOAT_WIDTH: FloatWidth = FloatWidth::Double;
#[cfg(target_arch = "riscv64")]
const FLOAT_WIDTH: FloatWidth = FloatWidth::Double;
#[cfg(target_arch = "powerpc64")]
const FLOAT_WIDTH: FloatWidth = FloatWidth::Double;
#[cfg(target_arch = "s390x")]
const FLOAT_WIDTH: FloatWidth = FloatWidth::Double;
#[cfg(target_arch = "x86")]
const FLOAT_WIDTH: FloatWidth = FloatWidth::Single;
#[cfg(target_arch = "arm")]
const FLOAT_WIDTH: FloatWidth = FloatWidth::Single;
#[cfg(target_arch = "riscv32")]
const FLOAT_WIDTH: FloatWidth = FloatWidth::Single;

// ── SIMD width ──────────────────────────────────────────────────────
#[cfg(target_feature = "avx512f")]
const SIMD_WIDTH: SimdWidth = SimdWidth::W512;
#[cfg(all(target_feature = "avx", not(target_feature = "avx512f")))]
const SIMD_WIDTH: SimdWidth = SimdWidth::W256;
#[cfg(all(target_feature = "sse2", not(target_feature = "avx")))]
const SIMD_WIDTH: SimdWidth = SimdWidth::W128;
#[cfg(target_arch = "aarch64")]
const SIMD_WIDTH: SimdWidth = SimdWidth::W128;
#[cfg(not(any(
    target_feature = "sse2",
    target_feature = "avx",
    target_feature = "avx512f",
    target_arch = "aarch64"
)))]
const SIMD_WIDTH: SimdWidth = SimdWidth::None;

// ── Integer multiply-accumulate ─────────────────────────────────────
// Every 64-bit target TrangorgeOS supports has one; soft-float embedded cores
// generally do not.
#[cfg(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "powerpc64",
    target_arch = "s390x"
))]
const HAS_MAC: bool = true;
#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "powerpc64",
    target_arch = "s390x"
)))]
const HAS_MAC: bool = false;

/// The FPU width the target was built for.
pub const fn float_width() -> FloatWidth {
    FLOAT_WIDTH
}

/// The SIMD width the target was built for.
pub const fn simd_width() -> SimdWidth {
    SIMD_WIDTH
}

/// Whether the target provides a hardware integer multiply-accumulate.
pub const fn has_mac() -> bool {
    HAS_MAC
}
