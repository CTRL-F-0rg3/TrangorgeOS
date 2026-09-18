use super::types::{Capability, CapabilitySet};
use alloc::vec;
use alloc::vec::Vec;

pub fn parent(cap: Capability) -> Option<Capability> {
    Some(match cap {
        Capability::Root => return None,

        Capability::Ring0 => Capability::Root,
        Capability::Driver => Capability::Ring0,
        Capability::User => Capability::Root,
        Capability::Admin => Capability::Ring0,

        Capability::PhysAlloc => Capability::Driver,
        Capability::VirtMap => Capability::Driver,
        Capability::Dma => Capability::Driver,
        Capability::Mmap => Capability::User,
        Capability::Protect => Capability::User,
        Capability::HugePage => Capability::PhysAlloc,

        Capability::Spawn => Capability::User,
        Capability::Kill => Capability::User,
        Capability::Debug => Capability::Admin,
        Capability::Ptrace => Capability::Debug,
        Capability::Sched => Capability::Admin,

        Capability::IpcSend => Capability::User,
        Capability::IpcRecv => Capability::User,
        Capability::IpcBroadcast => Capability::IpcSend,
        Capability::IpcPrivileged => Capability::IpcSend,

        Capability::FsRead => Capability::User,
        Capability::FsWrite => Capability::User,
        Capability::FsCreate => Capability::FsWrite,
        Capability::FsMount => Capability::Admin,

        Capability::DevPci => Capability::Driver,
        Capability::DevPort => Capability::Driver,
        Capability::DevMmio => Capability::Driver,
        Capability::DevIrq => Capability::Driver,

        Capability::SyscallAll => Capability::Driver,
        Capability::SyscallRestricted => Capability::User,
    })
}

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

pub fn set_implies(held: CapabilitySet, required: Capability) -> bool {
    for cap in held.iter() {
        if implies(cap, required) {
            return true;
        }
    }
    false
}

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

pub fn depth(cap: Capability) -> usize {
    let mut d = 0;
    let mut cur = cap;
    while let Some(p) = parent(cur) {
        d += 1;
        cur = p;
    }
    d
}

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