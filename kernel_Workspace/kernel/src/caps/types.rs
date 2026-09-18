use core::fmt;

pub type CapID = u8;

pub const MAX_CAPS: usize = 32;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Capability {
    Root = 0,
    Ring0 = 1,
    Driver = 2,
    User = 3,
    Admin = 4,

    PhysAlloc = 5,
    VirtMap = 6,
    Dma = 7,
    Mmap = 8,
    Protect = 9,
    HugePage = 10,

    Spawn = 11,
    Kill = 12,
    Debug = 13,
    Ptrace = 14,
    Sched = 15,

    IpcSend = 16,
    IpcRecv = 17,
    IpcBroadcast = 18,
    IpcPrivileged = 19,

    FsRead = 20,
    FsWrite = 21,
    FsCreate = 22,
    FsMount = 23,

    DevPci = 24,
    DevPort = 25,
    DevMmio = 26,
    DevIrq = 27,

    SyscallAll = 28,
    SyscallRestricted = 29,
}

impl Capability {
    pub const fn bit(self) -> u32 {
        1u32 << (self as u8)
    }

    pub const fn id(self) -> CapID {
        self as u8
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Root => "ROOT",
            Self::Ring0 => "RING0",
            Self::Driver => "DRIVER",
            Self::User => "USER",
            Self::Admin => "ADMIN",

            Self::PhysAlloc => "PHYS_ALLOC",
            Self::VirtMap => "VIRT_MAP",
            Self::Dma => "DMA",
            Self::Mmap => "MMAP",
            Self::Protect => "PROTECT",
            Self::HugePage => "HUGE_PAGE",

            Self::Spawn => "SPAWN",
            Self::Kill => "KILL",
            Self::Debug => "DEBUG",
            Self::Ptrace => "PTRACE",
            Self::Sched => "SCHED",

            Self::IpcSend => "IPC_SEND",
            Self::IpcRecv => "IPC_RECV",
            Self::IpcBroadcast => "IPC_BROADCAST",
            Self::IpcPrivileged => "IPC_PRIVILEGED",

            Self::FsRead => "FS_READ",
            Self::FsWrite => "FS_WRITE",
            Self::FsCreate => "FS_CREATE",
            Self::FsMount => "FS_MOUNT",

            Self::DevPci => "DEV_PCI",
            Self::DevPort => "DEV_PORT",
            Self::DevMmio => "DEV_MMIO",
            Self::DevIrq => "DEV_IRQ",

            Self::SyscallAll => "SYSCALL_ALL",
            Self::SyscallRestricted => "SYSCALL_RESTRICTED",
        }
    }

    pub fn iter_all() -> impl Iterator<Item = Capability> {
        [
            Self::Root, Self::Ring0, Self::Driver, Self::User, Self::Admin,
            Self::PhysAlloc, Self::VirtMap, Self::Dma, Self::Mmap,
            Self::Protect, Self::HugePage,
            Self::Spawn, Self::Kill, Self::Debug, Self::Ptrace, Self::Sched,
            Self::IpcSend, Self::IpcRecv, Self::IpcBroadcast, Self::IpcPrivileged,
            Self::FsRead, Self::FsWrite, Self::FsCreate, Self::FsMount,
            Self::DevPci, Self::DevPort, Self::DevMmio, Self::DevIrq,
            Self::SyscallAll, Self::SyscallRestricted,
        ].into_iter()
    }

    pub fn category(self) -> CapCategory {
        match self as u8 {
            0..=4 => CapCategory::Ring,
            5..=10 => CapCategory::Memory,
            11..=15 => CapCategory::Process,
            16..=19 => CapCategory::Ipc,
            20..=23 => CapCategory::Fs,
            24..=27 => CapCategory::Device,
            28..=29 => CapCategory::Syscall,
            _ => CapCategory::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapCategory {
    Ring,
    Memory,
    Process,
    Ipc,
    Fs,
    Device,
    Syscall,
    Unknown,
}

impl CapCategory {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ring => "Ring/Privilege",
            Self::Memory => "Memory",
            Self::Process => "Process",
            Self::Ipc => "IPC",
            Self::Fs => "Filesystem",
            Self::Device => "Device",
            Self::Syscall => "Syscalls",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct CapabilitySet {
    bits: u32,
}

impl CapabilitySet {
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn all() -> Self {
        Self { bits: 0x3FFF_FFFF }
    }

    pub const fn single(cap: Capability) -> Self {
        Self { bits: cap.bit() }
    }

    pub const fn from_bits(bits: u32) -> Self {
        Self { bits }
    }

    pub const fn has(self, cap: Capability) -> bool {
        (self.bits & cap.bit()) != 0
    }

    pub const fn add(self, cap: Capability) -> Self {
        Self { bits: self.bits | cap.bit() }
    }

    pub const fn remove(self, cap: Capability) -> Self {
        Self { bits: self.bits & !cap.bit() }
    }

    pub const fn intersect(self, other: Self) -> Self {
        Self { bits: self.bits & other.bits }
    }

    pub const fn union(self, other: Self) -> Self {
        Self { bits: self.bits | other.bits }
    }

    pub const fn diff(self, other: Self) -> Self {
        Self { bits: self.bits & !other.bits }
    }

    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    pub const fn count(self) -> usize {
        self.bits.count_ones() as usize
    }

    pub const fn bits(self) -> u32 {
        self.bits
    }

    pub fn iter(self) -> impl Iterator<Item = Capability> {
        Capability::iter_all().filter(move |&c| self.has(c))
    }
}

impl fmt::Debug for CapabilitySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CapSet[")?;
        let mut first = true;
        for cap in self.iter() {
            if !first { write!(f, ",")?; }
            write!(f, "{}", cap.name())?;
            first = false;
        }
        write!(f, "]")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CapabilityError {
    pub required: Capability,
    pub world_id: Option<u32>,
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.world_id {
            Some(wid) => write!(f, "capability {} required (world {})",
                                self.required.name(), wid),
            None => write!(f, "capability {} required", self.required.name()),
        }
    }
}

pub type CapResult<T> = Result<T, CapabilityError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_operations() {
        let s1 = CapabilitySet::empty().add(Capability::Root);
        let s2 = CapabilitySet::empty().add(Capability::Driver);

        assert!(s1.has(Capability::Root));
        assert!(!s1.has(Capability::Driver));
        assert_eq!(s1.union(s2).count(), 2);
        assert_eq!(s1.intersect(s2).count(), 0);
    }
}