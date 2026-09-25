//! IOMMU 框架使用的类型化标识与区间。
//!
//! 线缆上这些值都是朴素的 `u32` / `u64`；这里只做**编译期约束**，让驱动和
//! 调用方不会把控制器索引、域 id 与 requester id 互相搞混。

use core::fmt;

/// 一个 IOMMU 控制器（DMAR 中的一个 remapping unit、IVRS 中的一个 IVHD……）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ControllerId(pub u32);

impl ControllerId {
    pub const FIRST: Self = Self(0);

    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl From<u32> for ControllerId {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// 一个 IOMMU 地址空间域（ASID / VMID / domain id）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct DomainId(pub u32);

impl DomainId {
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

impl From<u32> for DomainId {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// 一个 PCI requester 的线缆表示：`(segment << 16) | bdf`。
///
/// 与驱动内部 `firm::pcie::PciDevice` 的内存表示一致，所以两边转换是零成本的。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct RequesterId(pub u32);

impl RequesterId {
    /// “不属于任何 requester”，固件保留区与故障记录里使用。
    pub const NONE: Self = Self(u32::MAX);

    #[inline]
    pub const fn new(segment: u16, bus: u8, device: u8, function: u8) -> Self {
        Self(((segment as u32) << 16) | ((bus as u32) << 8) | ((device as u32) << 3) | (function as u32))
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }

    #[inline]
    pub const fn segment(self) -> u16 {
        (self.0 >> 16) as u16
    }

    #[inline]
    pub const fn bdf(self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }

    #[inline]
    pub const fn bus(self) -> u8 {
        ((self.0 >> 8) & 0xFF) as u8
    }

    #[inline]
    pub const fn device(self) -> u8 {
        ((self.0 >> 3) & 0x1F) as u8
    }

    #[inline]
    pub const fn function(self) -> u8 {
        (self.0 & 0x07) as u8
    }
}

impl From<u32> for RequesterId {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl fmt::Display for RequesterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_valid() {
            return f.write_str("none");
        }
        write!(
            f,
            "{:04x}:{:02x}:{:02x}.{:01x}",
            self.segment(),
            self.bus(),
            self.device(),
            self.function()
        )
    }
}

/// 一段设备可见地址（IOVA）区间。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct IoRange {
    pub iova: u64,
    pub size: u64,
}

impl IoRange {
    pub const EMPTY: Self = Self { iova: 0, size: 0 };

    #[inline]
    pub const fn new(iova: u64, size: u64) -> Self {
        Self { iova, size }
    }

    /// 非空且不溢出的区间。
    #[inline]
    pub const fn is_valid(self) -> bool {
        self.size != 0 && self.iova.checked_add(self.size).is_some()
    }

    /// 区间是否按给定粒度对齐。
    ///
    /// `granule` 必须是 2 的幂，否则一律视为不对齐。
    #[inline]
    pub const fn is_aligned(self, granule: u64) -> bool {
        if granule == 0 || !granule.is_power_of_two() {
            return false;
        }
        self.iova & (granule - 1) == 0 && self.size & (granule - 1) == 0
    }

    /// 区间包含的页数。
    #[inline]
    pub const fn pages(self, granule: u64) -> Option<u64> {
        if granule == 0 || !granule.is_power_of_two() || !self.is_aligned(granule) {
            return None;
        }
        Some(self.size / granule)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requester_id_round_trips_bdf() {
        let requester = RequesterId::new(0x0002, 0x2a, 5, 3);
        assert_eq!(requester.segment(), 2);
        assert_eq!(requester.bus(), 0x2a);
        assert_eq!(requester.device(), 5);
        assert_eq!(requester.function(), 3);
        assert!(requester.is_valid());
        assert!(!RequesterId::NONE.is_valid());
    }

    #[test]
    fn io_range_validates_alignment_and_overflow() {
        let range = IoRange::new(0x1000, 0x2000);
        assert!(range.is_valid());
        assert!(range.is_aligned(0x1000));
        assert!(!range.is_aligned(0x4000));
        assert_eq!(range.pages(0x1000), Some(2));

        assert!(!IoRange::EMPTY.is_valid());
        assert!(IoRange::new(u64::MAX, 2).is_valid().eq(&false));
        assert_eq!(range.pages(0x3000), None);
    }
}
