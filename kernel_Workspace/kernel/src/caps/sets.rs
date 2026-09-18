use super::types::{Capability, CapabilitySet};
use super::hierarchy;

pub struct CapSetBuilder {
    set: CapabilitySet,
}

impl CapSetBuilder {
    pub fn new() -> Self {
        Self { set: CapabilitySet::empty() }
    }

    pub fn empty() -> Self {
        Self::new()
    }

    pub fn add(mut self, cap: Capability) -> Self {
        self.set = self.set.add(cap);
        self
    }

    pub fn add_many(mut self, caps: &[Capability]) -> Self {
        for &cap in caps {
            self.set = self.set.add(cap);
        }
        self
    }

    pub fn add_category(mut self, cat: super::types::CapCategory) -> Self {
        for cap in Capability::iter_all() {
            if cap.category() == cat {
                self.set = self.set.add(cap);
            }
        }
        self
    }

    pub fn remove(mut self, cap: Capability) -> Self {
        self.set = self.set.remove(cap);
        self
    }

    pub fn with_hierarchy(self) -> Self {
        Self { set: hierarchy::expand_hierarchy(self.set) }
    }

    pub fn build(self) -> CapabilitySet {
        self.set
    }
}

pub mod presets {
    use super::*;

    pub fn minimal_user() -> CapabilitySet {
        CapSetBuilder::new()
            .add(Capability::User)
            .add(Capability::Mmap)
            .add(Capability::IpcSend)
            .add(Capability::IpcRecv)
            .add(Capability::FsRead)
            .add(Capability::Spawn)
            .add(Capability::SyscallRestricted)
            .build()
    }

    pub fn standard_user() -> CapabilitySet {
        CapSetBuilder::new()
            .add(Capability::User)
            .add(Capability::Mmap)
            .add(Capability::Protect)
            .add(Capability::Spawn)
            .add(Capability::Kill)  
            .add(Capability::IpcSend)
            .add(Capability::IpcRecv)
            .add(Capability::IpcBroadcast)
            .add(Capability::FsRead)
            .add(Capability::FsWrite)
            .add(Capability::FsCreate)
            .add(Capability::SyscallRestricted)
            .build()
    }

    pub fn privileged_user() -> CapabilitySet {
        CapSetBuilder::new()
            .add(Capability::User)
            .add_many(&[
                Capability::Mmap, Capability::Protect, Capability::Spawn,
                Capability::Kill, Capability::IpcSend, Capability::IpcRecv,
                Capability::IpcBroadcast, Capability::IpcPrivileged,
                Capability::FsRead, Capability::FsWrite,
                Capability::FsCreate, Capability::FsMount,
                Capability::Sched, Capability::SyscallAll,
            ])
            .build()
    }

    pub fn driver() -> CapabilitySet {
        CapSetBuilder::new()
            .add(Capability::Driver)
            .add_many(&[
                Capability::PhysAlloc, Capability::VirtMap, Capability::Dma,
                Capability::DevPci, Capability::DevPort,
                Capability::DevMmio, Capability::DevIrq,
                Capability::IpcSend, Capability::IpcRecv,
                Capability::IpcPrivileged, Capability::SyscallAll,
            ])
            .build()
    }

    pub fn kernel() -> CapabilitySet {
        CapabilitySet::all()
    }

    pub fn sandbox() -> CapabilitySet {
        CapSetBuilder::new()
            .add(Capability::User)
            .add(Capability::FsRead)
            .add(Capability::IpcRecv)
            .add(Capability::SyscallRestricted)
            .build()
    }
}

pub fn validate_hierarchy(set: CapabilitySet) -> Result<(), &'static str> {
    for cap in set.iter() {
        let mut cur = cap;
        while let Some(p) = hierarchy::parent(cur) {
            if p != Capability::Root && !set.has(p) {
                return Err("hierarchy inconsistency: child without parent");
            }
            cur = p;
            if cur == Capability::Root {
                break;
            }
        }
    }
    Ok(())
}

pub fn effective(set: CapabilitySet) -> CapabilitySet {
    hierarchy::expand_hierarchy(set)
}

pub fn effective_diff(a: CapabilitySet, b: CapabilitySet) -> CapabilitySet {
    let a_eff = effective(a);
    let b_eff = effective(b);
    a_eff.diff(b_eff)
}

pub fn is_subset_with_hierarchy(subset: CapabilitySet, superset: CapabilitySet) -> bool {
    let sub_eff = effective(subset);
    let sup_eff = effective(superset);
    sup_eff.contains(sub_eff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets() {
        let user = presets::minimal_user();
        assert!(user.has(Capability::User));
        assert!(!user.has(Capability::Driver));
        assert!(!user.has(Capability::DevPci));

        let drv = presets::driver();
        assert!(drv.has(Capability::Driver));
        assert!(drv.has(Capability::DevPci));
    }

    #[test]
    fn test_effective() {
        let set = CapabilitySet::single(Capability::DevPci);
        let eff = effective(set);

        assert!(eff.has(Capability::DevPci));
        assert!(eff.has(Capability::Driver));
        assert!(eff.has(Capability::Ring0));
        assert!(eff.has(Capability::Root));
    }
}