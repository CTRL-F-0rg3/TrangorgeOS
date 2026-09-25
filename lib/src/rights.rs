//! Capability rights (a bitmask over a `u32`).

/// The set of rights a capability holder possesses over an object.
///
/// A request is only authorized when the capability it names carries at least
/// the rights the opcode requires (see [`crate::opcodes::Opcode::required`]).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rights(pub u32);

impl Rights {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXEC: Self = Self(1 << 2);
    pub const MAP: Self = Self(1 << 3);
    pub const GRANT: Self = Self(1 << 4);
    pub const TRANSFER: Self = Self(1 << 5);
    pub const SEND: Self = Self(1 << 6);
    pub const RECV: Self = Self(1 << 7);
    pub const CALL: Self = Self(1 << 8);
    pub const MANAGE: Self = Self(1 << 9);
    pub const ALL: Self = Self(0x3FF);

    /// `self` contains every bit set in `other`.
    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub const fn intersect(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[inline]
    pub const fn remove(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
