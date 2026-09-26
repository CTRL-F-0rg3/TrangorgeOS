//! Typed identifiers for the power framework.
//!
//! On the wire these are a plain `u32` index. The newtype exists so the compiler
//! can stop a caller from handing a source index to something that wants a
//! stream handle - there is no such confusion available here, and that is
//! deliberate: a power source has no lifetime beyond the driver's.

use kapi_abi::DsError;

/// An index into the driver's source list.
///
/// Not a handle. Sources are enumerated at start-up and live as long as the
/// driver does, and a source that is unplugged stays at its index and reports
/// absent. That is deliberate too: a hot-unplugged pack must not renumber the
/// list, or a UI that cached index 1 as "the battery" would end up watching the
/// charger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SourceId(pub u32);

impl SourceId {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }

    /// Reject an index the driver never issued.
    #[inline]
    pub const fn check(self, count: usize) -> Result<(), DsError> {
        if (self.0 as usize) < count {
            Ok(())
        } else {
            Err(DsError::DeviceNotFound)
        }
    }
}

impl From<u32> for SourceId {
    #[inline]
    fn from(v: u32) -> Self {
        Self(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_index_inside_the_list_is_accepted() {
        assert!(SourceId::new(0).check(1).is_ok());
        assert!(SourceId::new(2).check(3).is_ok(), "the last valid index");
    }

    #[test]
    fn an_index_past_the_list_is_not_found() {
        assert_eq!(SourceId::new(3).check(3).unwrap_err(), DsError::DeviceNotFound);
    }

    #[test]
    fn an_empty_list_accepts_nothing() {
        // The desktop case: no battery at all must be "not found", not a panic
        // and not source 0.
        assert_eq!(SourceId::new(0).check(0).unwrap_err(), DsError::DeviceNotFound);
    }

    #[test]
    fn ids_round_trip() {
        assert_eq!(SourceId::from(7u32), SourceId::new(7));
        assert_eq!(SourceId::new(7).index(), 7);
    }
}
