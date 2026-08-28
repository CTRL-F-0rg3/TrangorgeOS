//! Testy całego systemu capabilities.

#![cfg(test)]

use crate::caps::*;
use crate::caps::types::Capability;
use crate::caps::sets;

#[test]
fn test_full_flow() {
    init().unwrap();

    // Kernel world
    let kernel = 0;

    // Driver world
    let drv = store::register_world(Some(kernel), sets::presets::driver()).unwrap();
    assert!(store::world_has_cap(drv, Capability::DevPci));
    assert!(!store::world_has_cap(drv, Capability::Admin));

    // User world
    let usr = store::register_world(Some(kernel), sets::presets::standard_user()).unwrap();
    assert!(store::world_has_cap(usr, Capability::FsRead));
    assert!(!store::world_has_cap(usr, Capability::DevPci));

    // Grant od kernel do user
    grant::grant_cap(kernel, usr, Capability::Mmap).unwrap();
    assert!(store::world_has_cap(usr, Capability::Mmap));

    // Revoke
    revoke::revoke_from_world(usr, Capability::Mmap).unwrap();
    assert!(!store::world_has_cap(usr, Capability::Mmap));
}

#[test]
fn test_hierarchy_enforcement() {
    init().unwrap();

    let parent = store::register_world(None, sets::presets::standard_user()).unwrap();

    // Child nie może dostać DevPci (parent nie ma)
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