//! Revocation: odbieranie capabilities od worldów i globalnie.

use super::types::{Capability, CapabilitySet, CapResult, CapabilityError};
use super::hierarchy;
use super::store;
use super::audit;
use alloc::vec::Vec;

/// Odbierz capability od worlda
pub fn revoke_from_world(world_id: u32, cap: Capability) -> CapResult<()> {
    store::remove_world_cap(world_id, cap)
        .map_err(|_| CapabilityError { required: cap, world_id: Some(world_id) })?;

    audit::log_revoke(world_id, cap, true);
    Ok(())
}

/// Odbierz capability i całe poddrzewo (wszystkie child)
pub fn revoke_subtree(world_id: u32, cap: Capability) -> CapResult<()> {
    for c in hierarchy::subtree(cap) {
        let _ = store::remove_world_cap(world_id, c);
    }
    audit::log_revoke(world_id, cap, true);
    Ok(())
}

/// Globalne revocation (wszystkim)
pub fn revoke_global(cap: Capability) {
    store::add_global_revoked(cap);
    audit::log_revoke(0, cap, true);
}

/// Przywróć globalnie
pub fn restore_global(cap: Capability) {
    store::remove_global_revoked(cap);
}

/// Lista globalnie revoked
pub fn revoked_list() -> Vec<Capability> {
    let mut out = Vec::new();
    let mut bits = CapabilitySet::empty();

    // Odczyt przez global_caps: all minus revoked => revoked = all diff global
    let revoked = CapabilitySet::all().diff(store::global_caps());
    for cap in revoked.iter() {
        out.push(cap);
    }
    out
}

/// Czy capability jest globalnie revoked?
pub fn is_globally_revoked(cap: Capability) -> bool {
    !store::global_caps().has(cap)
}

/// Emergency: odbierz wszystko poza podstawowymi od worlda
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

        // Driver revoked => child (DevPci) nadal w secie, ale bez parent
        // hierarchia nie implikuje; sprawdzamy explicit
        assert!(!store::world_has_cap(wid, Capability::Driver));
    }
}