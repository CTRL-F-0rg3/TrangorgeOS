pub const PAGE_SIZE: usize = 4096;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;
pub const PAGE_SHIFT: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Ring {
    Kernel = 0,
    Driver = 1,
    User   = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ProtFlags(pub u32);

impl ProtFlags {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXEC: Self = Self(1 << 2);
    pub const USER: Self = Self(1 << 3);
    pub const DEVICE: Self = Self(1 << 4);

    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl core::ops::BitOr for ProtFlags {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct MapFlags(pub u32);

impl MapFlags {
    pub const NONE: Self = Self(0);
    pub const FIXED: Self = Self(1 << 0);
    pub const ANONYMOUS: Self = Self(1 << 1);
    pub const SHARED: Self = Self(1 << 2);
    pub const PRIVATE: Self = Self(1 << 3);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct IrqFlags(pub u32);

impl IrqFlags {
    pub const NONE: Self = Self(0);
    pub const LEVEL_TRIGGERED: Self = Self(1 << 0);
    pub const ACTIVE_LOW: Self = Self(1 << 1);
    pub const SHARED: Self = Self(1 << 2);
}