#![no_std]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Handle(pub u32);

impl Handle {
    pub const INVALID: Self = Self(u32::MAX);
    pub const KERNEL: Self = Self(0);
    pub const MANAGER: Self = Self(1);

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CapId(pub u64);

impl CapId {
    pub const NONE: Self = Self(0);
    pub const KERNEL_ALL: Self = Self(u64::MAX);

    pub const MEM_ALLOC: Self = Self(1 << 0);
    pub const MEM_MAP: Self = Self(1 << 1);
    pub const MEM_DMA: Self = Self(1 << 2);
    pub const IPC_CREATE: Self = Self(1 << 8);
    pub const IPC_SEND: Self = Self(1 << 9);
    pub const DEV_REGISTER: Self = Self(1 << 16);
    pub const DEV_MMIO: Self = Self(1 << 17);
    pub const DEV_IRQ: Self = Self(1 << 18);

    /// IOMMU: enumerate controllers / query descriptors.
    pub const IOMMU_ENUMERATE: Self = Self(1 << 20);
    /// IOMMU: create and destroy address-space domains.
    pub const IOMMU_DOMAIN: Self = Self(1 << 21);
    /// IOMMU: bind / unbind a PCI requester.
    pub const IOMMU_BIND: Self = Self(1 << 22);
    /// IOMMU: establish / tear down IOVA mappings.
    pub const IOMMU_MAP: Self = Self(1 << 23);
    pub const GFX_FRAMEBUFFER: Self = Self(1 << 24);
    pub const GFX_COMMAND: Self = Self(1 << 25);
    pub const AUDIO_PLAY: Self = Self(1 << 28);
    pub const AUDIO_CAPTURE: Self = Self(1 << 29);

    #[inline]
    pub const fn has(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub const fn remove(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Ok = 0,
    Unknown = -1,
    InvalidArg = -2,
    NoMemory = -3,
    PermissionDenied = -4,
    NotFound = -5,
    Busy = -6,
    Timeout = -7,
    InvalidHandle = -8,
    BufferTooSmall = -9,
    DeviceFault = -10,
    NotSupported = -11,
    AlreadyExists = -12,
    RingFull = -13,
    StaleId = -14,
    Throttled = -15,
}

impl Status {
    #[inline]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }

    #[inline]
    pub const fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::Ok,
            -1 => Self::Unknown,
            -2 => Self::InvalidArg,
            -3 => Self::NoMemory,
            -4 => Self::PermissionDenied,
            -5 => Self::NotFound,
            -6 => Self::Busy,
            -7 => Self::Timeout,
            -8 => Self::InvalidHandle,
            -9 => Self::BufferTooSmall,
            -10 => Self::DeviceFault,
            -11 => Self::NotSupported,
            -12 => Self::AlreadyExists,
            -13 => Self::RingFull,
            -14 => Self::StaleId,
            -15 => Self::Throttled,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct DeviceId(pub u64);

impl DeviceId {
    #[inline]
    pub const fn pci(vendor: u16, device: u16) -> Self {
        Self(((vendor as u64) << 16) | (device as u64))
    }

    #[inline]
    pub const fn usb(vendor: u16, product: u16) -> Self {
        Self(0x0100_0000_0000_0000 | ((vendor as u64) << 16) | (product as u64))
    }

    #[inline]
    pub const fn vendor(self) -> u16 {
        ((self.0 >> 16) & 0xFFFF) as u16
    }

    #[inline]
    pub const fn device(self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }

    #[inline]
    pub const fn bus_type(self) -> u8 {
        ((self.0 >> 56) & 0xFF) as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BusType {
    Pci = 0,
    Usb = 1,
    Acpi = 2,
    Platform = 3,
    Virtio = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DriverState {
    Unloaded = 0,
    Loading = 1,
    Running = 2,
    Suspended = 3,
    Error = 4,
    Unloading = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DeviceState {
    Discovered = 0,
    Attached = 1,
    Detached = 2,
    Error = 3,
}