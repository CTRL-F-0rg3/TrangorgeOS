//! Przyznawanie capabilities: grant, delegacja, dziedziczenie przy spawn,
//! granty tymczasowe z TTL.

use super::types::{Capability, CapabilitySet, CapResult, CapabilityError};
use super::hierarchy;
use super::store;
use super::audit;
use spin::Mutex;

/// Capabilities NIE dziedziczone automatycznie przy spawn
pub const NON_INHERITABLE: &[Capability] = &[
    Capability::Root,
    Capability::Admin,
    Capability::Kill,
    Capability::Debug,
    Capability::Ptrace,
    Capability::FsMount,
    Capability::IpcPrivileged,
    Capability::SyscallAll,
];

/// Przyznaj pojedynczą capability (granter -> target)
pub fn grant_cap(granter: u32, target: u32, cap: Capability) -> CapResult<()> {
    // Granter musi sam mieć tę capability
    if !store::world_has_cap(granter, cap) {
        audit::log_grant(granter, target, cap, false);
        return Err(CapabilityError { required: cap, world_id: Some(granter) });
    }

    store::add_world_cap(target, cap)
        .map_err(|_| CapabilityError { required: cap, world_id: Some(target) })?;

    audit::log_grant(granter, target, cap, true);
    Ok(())
}

/// Deleguj cały zbiór
pub fn delegate_caps(granter: u32, target: u32, caps: CapabilitySet) -> CapResult<()> {
    for cap in caps.iter() {
        grant_cap(granter, target, cap)?;
    }
    Ok(())
}

/// Zestaw dziedziczony przy spawn (parent minus NON_INHERITABLE)
pub fn inherit_set(parent_world: u32) -> CapabilitySet {
    let parent = match store::get_world_caps(parent_world) {
        Ok(c) => c,
        Err(_) => CapabilitySet::empty(),
    };

    let mut child = parent;
    for &cap in NON_INHERITABLE {
        child = child.remove(cap);
    }
    child
}

/// Zarejestruj child world dziedziczący po parent
pub fn spawn_child(parent_world: u32) -> Result<u32, &'static str> {
    let child_set = inherit_set(parent_world);
    store::register_world(Some(parent_world), child_set)
}

/* ---- granty tymczasowe (TTL) ---- */

#[derive(Clone, Copy)]
struct TempGrant {
    world_id: u32,
    cap: Capability,
    expires_tick: u64,
    active: bool,
}

const MAX_TEMP: usize = 128;

static TEMP: Mutex<[TempGrant; MAX_TEMP]> = Mutex::new([TempGrant {
    world_id: 0,
    cap: Capability::User,
    expires_tick: 0,
    active: false,
}; MAX_TEMP]);

fn now_tick() -> u64 {
    extern "C" { fn k_tick() -> u64; }
    unsafe { k_tick() }
}

/// Przyznaj capability na ograniczony czas
pub fn grant_temporary(granter: u32, target: u32, cap: Capability,
                       ttl_ticks: u64) -> CapResult<()>
{
    grant_cap(granter, target, cap)?;

    let mut t = TEMP.lock();
    let slot = t.iter_mut().find(|g| !g.active)
        .ok_or(CapabilityError { required: cap, world_id: Some(target) })?;

    *slot = TempGrant {
        world_id: target,
        cap,
        expires_tick: now_tick() + ttl_ticks,
        active: true,
    };

    Ok(())
}

/// Usuń wygasłe granty (wywołuj okresowo, np. w tick)
pub fn prune_expired() {
    let t_now = now_tick();
    let mut t = TEMP.lock();

    for g in t.iter_mut() {
        if g.active && t_now >= g.expires_tick {
            let _ = store::remove_world_cap(g.world_id, g.cap);
            audit::log_revoke(g.world_id, g.cap, true);
            g.active = false;
        }
    }
}

/// Czy capability jest grantem tymczasowym?
pub fn is_temporary(world_id: u32, cap: Capability) -> bool {
    let t = TEMP.lock();
    t.iter().any(|g| g.active && g.world_id == world_id && g.cap == cap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caps::sets;

    #[test]
    fn test_inherit() {
        crate::caps::init().unwrap();

        let parent = store::register_world(None, sets::presets::privileged_user()).unwrap();
        let child_set = inherit_set(parent);

        assert!(child_set.has(Capability::User));
        assert!(child_set.has(Capability::Spawn));
        assert!(!child_set.has(Capability::Admin));   // nie dziedziczone
        assert!(!child_set.has(Capability::SyscallAll));
    }
}