use super::types::{Capability, CapabilitySet, CapResult, CapabilityError};
use super::hierarchy;
use super::store;
use super::audit;
use alloc::vec::Vec;

pub fn revoke_from_world(world_id: u32, cap: Capability) -> CapResult<()> {
    store::remove_world_cap(world_id, cap)
        .map_err(|_| CapabilityError { required: cap, world_id: Some(world_id) })?;

    audit::log_revoke(world_id, cap, true);
    Ok(())
}

pub fn revoke_subtree(world_id: u32, cap: Capability) -> CapResult<()> {
    for c in hierarchy::subtree(cap) {
        let _ = store::remove_world_cap(world_id, c);
    }
    audit::log_revoke(world_id, cap, true);
    Ok(())
}

pub fn revoke_global(cap: Capability) {
    store::add_global_revoked(cap);
    audit::log_revoke(0, cap, true);
}

pub fn restore_global(cap: Capability) {
    store::remove_global_revoked(cap);
}

pub fn revoked_list() -> Vec<Capability> {
    let mut out = Vec::new();
    let mut bits = CapabilitySet::empty();

    let revoked = CapabilitySet::all().diff(store::global_caps());
    for cap in revoked.iter() {
        out.push(cap);
    }
    out
}

pub fn is_globally_revoked(cap: Capability) -> bool {
    !store::global_caps().has(cap)
}

pub fn lockdown(world_id: u32) {
    let keep = CapabilitySet::empty()
        .add(Capability::User)
        .add(Capability::FsRead)
        .add(Capability::IpcRecv);

    for cap in Capability::iter_all() {
        if !keep.has(cap) {
            let _ = store::remove_world_cap(world_id, cap);
        }
    }

    audit::log_revoke(world_id, Capability::Root, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caps::sets;

    #[test]
    fn test_revoke() {
        crate::caps::init().unwrap();

        let wid = store::register_world(None, sets::presets::driver()).unwrap();
        assert!(store::world_has_cap(wid, Capability::DevPci));

        revoke_from_world(wid, Capability::Driver).unwrap();

        assert!(!store::world_has_cap(wid, Capability::Driver));
    }
}