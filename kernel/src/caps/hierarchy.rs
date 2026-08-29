//! Hierarchia capabilities: parent zawiera wszystkie child capabilities.
//!
//! Drzewo:
//!   CAP_ROOT (parent wszystkich)
//!   ├─ CAP_RING0
//!   │  ├─ CAP_DRIVER
//!   │  │  ├─ CAP_PHYS_ALLOC
//!   │  │  ├─ CAP_VIRT_MAP
//!   │  │  ├─ CAP_DMA
//!   │  │  ├─ CAP_DEV_PCI
//!   │  │  ├─ CAP_DEV_PORT
//!   │  │  ├─ CAP_DEV_MMIO
//!   │  │  ├─ CAP_DEV_IRQ
//!   │  │  └─ CAP_SYSCALL_ALL
//!   │  └─ CAP_ADMIN
//!   └─ CAP_USER (implicit dla ring3)
//!      ├─ CAP_MMAP
//!      ├─ CAP_PROTECT
//!      ├─ CAP_SPAWN
//!      ├─ CAP_KILL (własność procesu sprawdzana osobno)
//!      ├─ CAP_IPC_SEND
//!      ├─ CAP_IPC_RECV
//!      ├─ CAP_FS_READ
//!      ├─ CAP_FS_WRITE
//!      └─ CAP_SYSCALL_RESTRICTED

use super::types::{Capability, CapabilitySet};
use alloc::vec;
use alloc::vec::Vec;

/// Rodzic capability (None dla ROOT)
pub fn parent(cap: Capability) -> Option<Capability> {
    Some(match cap {
        Capability::Root => return None,

        // Ring/Privilege
        Capability::Ring0 => Capability::Root,
        Capability::Driver => Capability::Ring0,
        Capability::User => Capability::Root,
        Capability::Admin => Capability::Ring0,

        // Memory
        Capability::PhysAlloc => Capability::Driver,
        Capability::VirtMap => Capability::Driver,
        Capability::Dma => Capability::Driver,
        Capability::Mmap => Capability::User,
        Capability::Protect => Capability::User,
        Capability::HugePage => Capability::PhysAlloc,

        // Process
        Capability::Spawn => Capability::User,
        // Kill należy pod User: presety userspace przyznają Kill ("tylko
        // własne procesy" — własność sprawdzana na poziomie syscalla).
        // Pod Admin (→ Ring0) capability Kill powodowała, że zwykły user
        // hierarchicznie implikował całe poddrzewo drivera (eskalacja).
        Capability::Kill => Capability::User,
        Capability::Debug => Capability::Admin,
        Capability::Ptrace => Capability::Debug,
        Capability::Sched => Capability::Admin,

        // IPC
        Capability::IpcSend => Capability::User,
        Capability::IpcRecv => Capability::User,
        Capability::IpcBroadcast => Capability::IpcSend,
        Capability::IpcPrivileged => Capability::IpcSend,

        // FS
        Capability::FsRead => Capability::User,
        Capability::FsWrite => Capability::User,
        Capability::FsCreate => Capability::FsWrite,
        Capability::FsMount => Capability::Admin,

        // Device
        Capability::DevPci => Capability::Driver,
        Capability::DevPort => Capability::Driver,
        Capability::DevMmio => Capability::Driver,
        Capability::DevIrq => Capability::Driver,

        // Syscalls
        Capability::SyscallAll => Capability::Driver,
        Capability::SyscallRestricted => Capability::User,
    })
}

/// Czy `cap` implikuje `required` (parent zawiera child)?
pub fn implies(held: Capability, required: Capability) -> bool {
    if held == required {
        return true;
    }

    let mut cur = required;
    while let Some(p) = parent(cur) {
        if p == held {
            return true;
        }
        cur = p;
    }
    false
}

/// Czy zbiór `held` implikuje `required`?
pub fn set_implies(held: CapabilitySet, required: Capability) -> bool {
    for cap in held.iter() {
        if implies(cap, required) {
            return true;
        }
    }
    false
}

/// Rozszerz zbiór do pełnej hierarchii (wszystkie parent capabilities)
pub fn expand_hierarchy(set: CapabilitySet) -> CapabilitySet {
    let mut result = set;

    for cap in set.iter() {
        let mut cur = cap;
        while let Some(p) = parent(cur) {
            result = result.add(p);
            cur = p;
        }
    }

    result
}

/// Ścieżka od capability do ROOT
pub fn path_to_root(cap: Capability) -> Vec<Capability> {
    let mut path = vec![cap];
    let mut cur = cap;
    while let Some(p) = parent(cur) {
        path.push(p);
        cur = p;
    }
    path.reverse();
    path
}

/// Głębokość capability w hierarchii (ROOT = 0)
pub fn depth(cap: Capability) -> usize {
    let mut d = 0;
    let mut cur = cap;
    while let Some(p) = parent(cur) {
        d += 1;
        cur = p;
    }
    d
}

/// Wszystkie capabilities pod daną w drzewie (inclusive)
pub fn subtree(root: Capability) -> Vec<Capability> {
    let mut result = vec![root];
    let mut i = 0;

    while i < result.len() {
        let cur = result[i];
        for cap in Capability::iter_all() {
            if parent(cap) == Some(cur) && !result.contains(&cap) {
                result.push(cap);
            }
        }
        i += 1;
    }

    result
}

/// Czy `candidate` może być delegowany jeśli ma `holder`?
/// (Nie można delegować capability wyższej niż się posiada)
pub fn can_delegate(holder: CapabilitySet, candidate: Capability) -> bool {
    set_implies(holder, candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy() {
        assert!(implies(Capability::Root, Capability::Driver));
        assert!(implies(Capability::Driver, Capability::DevPci));
        assert!(!implies(Capability::DevPci, Capability::Driver));

        let driver_set = CapabilitySet::single(Capability::Driver);
        assert!(set_implies(driver_set, Capability::DevPci));
        assert!(!set_implies(driver_set, Capability::Admin));

        let expanded = expand_hierarchy(CapabilitySet::single(Capability::DevPci));
        assert!(expanded.has(Capability::DevPci));
        assert!(expanded.has(Capability::Driver));
        assert!(expanded.has(Capability::Ring0));
        assert!(expanded.has(Capability::Root));
    }

    #[test]
    fn test_depth() {
        assert_eq!(depth(Capability::Root), 0);
        assert_eq!(depth(Capability::Driver), 2);
        assert_eq!(depth(Capability::DevPci), 3);
    }
}