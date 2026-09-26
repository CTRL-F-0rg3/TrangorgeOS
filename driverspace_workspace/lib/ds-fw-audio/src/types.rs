//! Typed identifiers used across the audio framework.
//!
//! On the wire these values are plain `u32`; the newtypes exist purely to give
//! the compiler a way to stop a driver and a caller from confusing a PCM
//! index with a stream handle or a mixer element.

use kapi_abi::DsError;

/// An open PCM handle, returned by `AudioDevice::open`.
///
/// A handle is meaningful only to the service that issued it and dies with the
/// stream; [`PcmHandle::INVALID`] is the "not a handle" value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PcmHandle(pub u32);

impl PcmHandle {
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }
}

impl From<u32> for PcmHandle {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// A card index, ALSA's `N` in `hw:N,M`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CardId(pub u32);

impl CardId {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl From<u32> for CardId {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// A PCM endpoint index within a card, ALSA's `M` in `hw:N,M`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PcmId(pub u32);

impl PcmId {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl From<u32> for PcmId {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// A mixer element index, as reported by `AudioDevice::mixer_count`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MixerId(pub u32);

impl MixerId {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl From<u32> for MixerId {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// A jack / plug index, as reported by `AudioDevice::jack_count`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct JackId(pub u32);

impl JackId {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl From<u32> for JackId {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// A hwdep node index, as reported by `AudioDevice::hwdep_count`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct HwdepId(pub u32);

impl HwdepId {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl From<u32> for HwdepId {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Whether a completed transfer is a clean end or a short one.
///
/// ALSA returns a frame count and reports underruns separately; this enum
/// keeps the two apart so a caller cannot mistake a short write for a full
/// one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Transfer {
    /// All requested frames moved.
    Complete,
    /// Fewer frames moved than requested; the stream is now in `XRun`.
    Underrun,
    /// The stream was closed or dropped before the transfer completed.
    Aborted,
}

impl Transfer {
    /// Build a transfer outcome from a frame count against the request.
    #[inline]
    pub const fn from_frames(transferred: u32, requested: u32) -> Self {
        if transferred == requested {
            Self::Complete
        } else {
            Self::Underrun
        }
    }

    #[inline]
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }
}

/// Whether a `drop` request should stop immediately or play out the buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum DropMode {
    /// Stop now; the tail of the buffer is discarded.
    Drop,
    /// Play out what is already buffered, then stop.
    Drain,
}

impl DropMode {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Drain,
            _ => Self::Drop,
        }
    }
}

/// Reject a handle the service does not own, before any driver call happens.
///
/// The framework applies this uniformly, which is what stops a client from
/// steering a stream through a handle that was never issued.
#[inline]
pub const fn check_handle(handle: PcmHandle) -> Result<(), DsError> {
    if handle.is_valid() {
        Ok(())
    } else {
        Err(DsError::InvalidHandle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_distinguish_valid_from_invalid() {
        assert!(PcmHandle::new(0).is_valid());
        assert!(PcmHandle::new(7).is_valid());
        assert!(!PcmHandle::INVALID.is_valid());
        assert_eq!(PcmHandle::from(3u32), PcmHandle::new(3));
        assert_eq!(PcmHandle::new(3).raw(), 3);
    }

    #[test]
    fn check_handle_rejects_the_sentinel() {
        assert!(check_handle(PcmHandle::new(1)).is_ok());
        assert_eq!(
            check_handle(PcmHandle::INVALID).unwrap_err(),
            DsError::InvalidHandle
        );
    }

    #[test]
    fn transfer_classifies_short_writes_as_underruns() {
        assert_eq!(Transfer::from_frames(512, 512), Transfer::Complete);
        assert!(Transfer::from_frames(512, 512).is_complete());
        assert_eq!(Transfer::from_frames(0, 512), Transfer::Underrun);
        assert!(!Transfer::from_frames(256, 512).is_complete());
    }

    #[test]
    fn drop_mode_defaults_to_drop() {
        assert_eq!(DropMode::from_u32(0), DropMode::Drop);
        assert_eq!(DropMode::from_u32(1), DropMode::Drain);
        assert_eq!(DropMode::from_u32(99), DropMode::Drop);
    }

    #[test]
    fn identifier_newtypes_convert_both_ways() {
        assert_eq!(CardId::from(2u32).index(), 2);
        assert_eq!(PcmId::new(1).index(), 1);
        assert_eq!(MixerId::new(5).index(), 5);
        assert_eq!(JackId::new(0).index(), 0);
        assert_eq!(HwdepId::new(4).index(), 4);
    }
}
