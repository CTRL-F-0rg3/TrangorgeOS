//! Podstawowe typy: Capability, CapID, CapabilitySet.
//!
//! Każda capability to bit w u32 (32 możliwości, używamy 30).
//! CapabilitySet to bitmap tych bitów.

use core::fmt;

/// ID capability (0..31)
pub type CapID = u8;

/// Maksymalna liczba capabilities (32 = rozmiar u32)
pub const MAX_CAPS: usize = 32;

/// Numer capability w systemie (0..29 używane)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Capability {
    // === Ring/Privilege (0-4) ===
    /// Superuser - robi wszystko, parent wszystkich
    Root = 0,
    /// Dostęp do operacji ring0
    Ring0 = 1,
    /// Może być driverspace
    Driver = 2,
    /// Implicit - userspace
    User = 3,
    /// Administracyjne (zmiana polityki)
    Admin = 4,

    // === Memory (5-10) ===
    /// Alokacja ramek fizycznych
    PhysAlloc = 5,
    /// Mapowanie pamięci wirtualnej
    VirtMap = 6,
    /// DMA / contiguous alloc
    Dma = 7,
    /// mmap z userspace
    Mmap = 8,
    /// mprotect (zmiana uprawnień stron)
    Protect = 9,
    /// Huge pages
    HugePage = 10,

    // === Process (11-15) ===
    /// Tworzenie procesów
    Spawn = 11,
    /// Zabicie procesu
    Kill = 12,
    /// Debugowanie innego procesu
    Debug = 13,
    /// ptrace-like
    Ptrace = 14,
    /// Zmiana priorytetów schedulera
    Sched = 15,

    // === IPC (16-19) ===
    /// Wysyłanie IPC
    IpcSend = 16,
    /// Odbieranie IPC
    IpcRecv = 17,
    /// Broadcast IPC
    IpcBroadcast = 18,
    /// IPC do uprzywilejowanych
    IpcPrivileged = 19,

    // === FS (20-23) ===
    /// Odczyt plików
    FsRead = 20,
    /// Zapis plików
    FsWrite = 21,
    /// Tworzenie plików
    FsCreate = 22,
    /// Mount/unmount
    FsMount = 23,

    // === Device (24-27) ===
    /// Dostęp do PCI
    DevPci = 24,
    /// I/O port access
    DevPort = 25,
    /// MMIO
    DevMmio = 26,
    /// Rejestracja IRQ
    DevIrq = 27,

    // === Syscalls (28-29) ===
    /// Wszystkie syscalls
    SyscallAll = 28,
    /// Subset syscalls (bezpieczne)
    SyscallRestricted = 29,
}

impl Capability {
    /// Bit w CapabilitySet
    pub const fn bit(self) -> u32 {
        1u32 << (self as u8)
    }

    /// CapID
    pub const fn id(self) -> CapID {
        self as u8
    }

    /// Nazwa capability (do debug/audit)
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

    /// Iteracja po wszystkich capabilities
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

    /// Z kategorii (do grupowania)
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

/// Kategoria capability (do grupowania w UI/debug)
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

/// Zbiór capabilities (bitmap 32-bit)
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct CapabilitySet {
    bits: u32,
}

impl CapabilitySet {
    /// Pusty zbiór
    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Pełny zbiór (wszystkie 30 capabilities)
    pub const fn all() -> Self {
        // Pierwsze 30 bitów ustawione
        Self { bits: 0x3FFF_FFFF }
    }

    /// Tylko jedna capability
    pub const fn single(cap: Capability) -> Self {
        Self { bits: cap.bit() }
    }

    /// Z bitmapy (bez walidacji)
    pub const fn from_bits(bits: u32) -> Self {
        Self { bits }
    }

    /// Czy zbiór zawiera capability
    pub const fn has(self, cap: Capability) -> bool {
        (self.bits & cap.bit()) != 0
    }

    /// Dodaj capability
    pub const fn add(self, cap: Capability) -> Self {
        Self { bits: self.bits | cap.bit() }
    }

    /// Usuń capability
    pub const fn remove(self, cap: Capability) -> Self {
        Self { bits: self.bits & !cap.bit() }
    }

    /// Przecięcie zbiorów (AND)
    pub const fn intersect(self, other: Self) -> Self {
        Self { bits: self.bits & other.bits }
    }

    /// Suma zbiorów (OR)
    pub const fn union(self, other: Self) -> Self {
        Self { bits: self.bits | other.bits }
    }

    /// Różnica zbiorów
    pub const fn diff(self, other: Self) -> Self {
        Self { bits: self.bits & !other.bits }
    }

    /// Czy zbiór jest pusty
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    /// Czy zbiór zawiera wszystkie capabilities z `other`
    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// Liczba ustawionych capabilities
    pub const fn count(self) -> usize {
        self.bits.count_ones() as usize
    }

    /// Bitmapa (do eksportu)
    pub const fn bits(self) -> u32 {
        self.bits
    }

    /// Iteracja po capabilities w zbiorze
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

/// Błąd braku capability
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

/// Wynik operacji wymagającej capability
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