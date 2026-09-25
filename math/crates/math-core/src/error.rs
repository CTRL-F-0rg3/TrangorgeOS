//! Math error vocabulary, mapped onto the system's.
//!
//! A math routine in a kernel or driver has the same failure modes as anything
//! else: a domain error, an overflow, a division by zero. Rather than inventing
//! a parallel error space, [`MathError`] converts straight into
//! [`kapi_abi::Status`], so a math failure travels through `ds-manager` and the
//! IPC layer like any other error and needs no special casing upstream.

/// Failure modes the math stack can report.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MathError {
    /// Argument outside the function's domain (`ln` of a negative number,
    /// `asin` of a value beyond 1, `pow` with a negative base and a fractional
    /// exponent).
    Domain,
    /// The result is finite but too large to represent.
    Overflow,
    /// The result is non-zero but too small to represent.
    Underflow,
    /// Division by zero, or a zero divisor where the routine requires non-zero.
    DivisionByZero,
    /// A required `sqrt`/`ln` of a value that should have been non-negative
    /// was negative - almost always an upstream arithmetic bug rather than bad
    /// input.
    NotANumber,
    /// The operation is well-formed but not implemented for these arguments.
    Unimplemented,
}

impl MathError {
    /// Convert into the system-wide status code.
    pub const fn to_status(self) -> kapi_abi::Status {
        match self {
            Self::Domain => kapi_abi::Status::InvalidArg,
            Self::Overflow => kapi_abi::Status::Overflow,
            Self::Underflow => kapi_abi::Status::Overflow,
            Self::DivisionByZero => kapi_abi::Status::DivideByZero,
            Self::NotANumber => kapi_abi::Status::InvalidArg,
            Self::Unimplemented => kapi_abi::Status::NotSupported,
        }
    }

    /// The numeric code as carried on the wire (`Status::code`).
    pub const fn to_code(self) -> i32 {
        self.to_status().code()
    }

    /// Recover an error from a system status, if it names one of ours.
    ///
    /// The mapping is lossy in one direction only: a status of `InvalidArg`
    /// could equally have come from a caller-side argument check, so
    /// `Status -> MathError` deliberately reports `Domain` for it rather than
    /// pretending to know.
    pub const fn from_status(status: kapi_abi::Status) -> Option<Self> {
        match status {
            kapi_abi::Status::Overflow => Some(Self::Overflow),
            kapi_abi::Status::DivideByZero => Some(Self::DivisionByZero),
            kapi_abi::Status::NotSupported => Some(Self::Unimplemented),
            kapi_abi::Status::InvalidArg => Some(Self::Domain),
            _ => None,
        }
    }

    /// A short, stable description - useful in logs and in `no_std` error
    /// paths where formatting is not available.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Domain => "argument outside the function domain",
            Self::Overflow => "result overflows the representable range",
            Self::Underflow => "result underflows to zero",
            Self::DivisionByZero => "division by zero",
            Self::NotANumber => "operand is not a number",
            Self::Unimplemented => "not implemented for these arguments",
        }
    }
}

impl core::fmt::Display for MathError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Result alias for math operations that can fail.
pub type MathResult<T> = core::result::Result<T, MathError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_error_maps_to_a_status() {
        for error in [
            MathError::Domain,
            MathError::Overflow,
            MathError::Underflow,
            MathError::DivisionByZero,
            MathError::NotANumber,
            MathError::Unimplemented,
        ] {
            let status = error.to_status();
            assert_ne!(status, kapi_abi::Status::Ok, "{error:?} must not map to Ok");
            assert_eq!(error.to_code(), status.code());
            assert!(!error.as_str().is_empty());
        }
    }

    #[test]
    fn round_trips_through_status_where_meaningful() {
        assert_eq!(
            MathError::from_status(MathError::Overflow.to_status()),
            Some(MathError::Overflow)
        );
        assert_eq!(
            MathError::from_status(MathError::DivisionByZero.to_status()),
            Some(MathError::DivisionByZero)
        );
        // Success and unrelated statuses carry no math meaning.
        assert_eq!(MathError::from_status(kapi_abi::Status::Ok), None);
        assert_eq!(MathError::from_status(kapi_abi::Status::NotFound), None);
    }

    #[test]
    fn display_matches_as_str() {
        // Rendered into a fixed buffer: a `no_std` crate has no `ToString`, and
        // pulling in `alloc` just to test a one-line `Display` would be silly.
        fn render(error: MathError) -> [u8; 96] {
            use core::fmt::Write;
            struct Buf {
                bytes: [u8; 96],
                len: usize,
            }
            impl Write for Buf {
                fn write_str(&mut self, s: &str) -> core::fmt::Result {
                    let end = self.len + s.len();
                    if end > self.bytes.len() {
                        return Err(core::fmt::Error);
                    }
                    self.bytes[self.len..end].copy_from_slice(s.as_bytes());
                    self.len = end;
                    Ok(())
                }
            }
            let mut buf = Buf { bytes: [0; 96], len: 0 };
            write!(buf, "{error}").expect("the buffer is large enough");
            let mut out = [0u8; 96];
            out[..buf.len].copy_from_slice(&buf.bytes[..buf.len]);
            out
        }

        let rendered = render(MathError::Domain);
        let text = core::str::from_utf8(&rendered[..rendered.iter().position(|b| *b == 0).unwrap()])
            .expect("Display emitted valid UTF-8");
        assert_eq!(text, MathError::Domain.as_str());
        assert!(text.contains("domain"));
    }
}
