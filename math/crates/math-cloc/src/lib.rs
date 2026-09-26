#![no_std]
#![deny(clippy::integer_arithmetic)]

pub mod ffi;

/// Universal time representation.
/// Memory layout is strictly guaranteed for FFI.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClocTime {
    pub secs: u64,
    pub nanos: u32,
}

impl ClocTime {
    /// Safely adds two time values.
    /// Returns None on integer overflow to prevent silent wrap-around.
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let mut total_nanos = self.nanos.checked_add(rhs.nanos)?;
        let mut total_secs = self.secs;

        if total_nanos >= 1_000_000_000 {
            total_nanos -= 1_000_000_000;
            total_secs = total_secs.checked_add(1)?;
        }
        
        total_secs = total_secs.checked_add(rhs.secs)?;

        Some(Self {
            secs: total_secs,
            nanos: total_nanos,
        })
    }

    /// Validates monotonicity to prevent TOCTOU and time-rewind attacks.
    /// Returns true if `self` is >= `previous`.
    pub fn is_monotonic(self, previous: Self) -> bool {
        if self.secs > previous.secs {
            return true;
        }
        if self.secs == previous.secs && self.nanos >= previous.nanos {
            return true;
        }
        false
    }
}