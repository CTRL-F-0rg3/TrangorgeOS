#![cfg(test)]

use crate::caps::*;
use crate::caps::types::Capability;
use crate::caps::sets;

#[test]
fn test_full_flow() {
    init().unwrap();

    let kernel = crate::caps::check::kernel_world_id();

    let drv = store::register_world(Some(kernel), sets::presets::driver()).unwrap();
    assert!(store::world_has_cap(drv, Capability::DevPci));
    assert!(!store::world_has_cap(drv, Capability::Admin));

    let usr = store::register_world(Some(kernel), sets::presets::standard_user()).unwrap();
    assert!(store::world_has_cap(usr, Capability::FsRead));
    assert!(!store::world_has_cap(usr, Capability::DevPci));

    grant::grant_cap(kernel, usr, Capability::Mmap).unwrap();
    assert!(store::world_has_cap(usr, Capability::Mmap));

    revoke::revoke_from_world(usr, Capability::Mmap).unwrap();
    assert!(!store::world_has_cap(usr, Capability::Mmap));
}

#[test]
fn test_hierarchy_enforcement() {
    init().unwrap();

    let parent = store::register_world(None, sets::presets::standard_user()).unwrap();

    let bad = store::register_world(Some(parent),
        CapabilitySet::single(Capability::DevPci));
    assert!(bad.is_err());
}

#[test]
fn test_global_revocation() {
    init().unwrap();

    let wid = store::register_world(None, sets::presets::driver()).unwrap();
    assert!(store::world_has_cap(wid, Capability::Dma));

    revoke::revoke_global(Capability::Dma);
    assert!(!store::world_has_cap(wid, Capability::Dma));

    revoke::restore_global(Capability::Dma);
    assert!(store::world_has_cap(wid, Capability::Dma));
}

#[test]
fn test_audit_trail() {
    init().unwrap();

    let wid = store::register_world(None, sets::presets::standard_user()).unwrap();

    store::world_has_cap(wid, Capability::User);
    assert!(audit::count() > 0);
}