//! Magazyn capabilities: globalne + per-world/per-process.
//!
//! Każdy world/process ma własny CapabilitySet.
//! Kernel ma globalny zestaw (zawsze all).

use super::types::{Capability, CapabilitySet, CapabilityError};
use super::sets;
use super::hierarchy;
use spin::Mutex;

const MAX_WORLDS: usize = 256;

#[derive(Clone, Copy)]
pub struct WorldCaps {
    pub world_id: u32,
    pub caps: CapabilitySet,
    pub inherited_from: Option<u32>,
    pub revokable: bool,  // czy można revoke
}

static STORE: Mutex<StoreInner> = Mutex::new(StoreInner {
    worlds: [None; MAX_WORLDS],
    global_revoked: CapabilitySet::empty(),
    next_id: 1,
});

struct StoreInner {
    worlds: [Option<WorldCaps>; MAX_WORLDS],
    global_revoked: CapabilitySet,
    next_id: u32,
}

pub fn init_store() -> Result<(), &'static str> {
    let mut s = STORE.lock();
    for i in 0..MAX_WORLDS {
        s.worlds[i] = None;
    }
    Ok(())
}

/// Globalne capabilities (zawsze all minus revoked)
pub fn global_caps() -> CapabilitySet {
    let s = STORE.lock();
    CapabilitySet::all().diff(s.global_revoked)
}

/// Zarejestruj nowy world z danym zestawem
pub fn register_world(parent_world: Option<u32>, initial: CapabilitySet) -> Result<u32, &'static str> {
    let mut s = STORE.lock();

    // Walidacja: jeśli parent, child musi być podzbiorem.
    // UWAGA: celowo BEZ expand_hierarchy — rozwinięcie zbioru rodzica "w górę"
    // dodawałoby Root/Ring0 (bo User -> Root), a Root implikuje wszystko,
    // co unieważniało by całe ograniczenie dziedziczenia (eskalacja uprawnień).
    // set_implies samo wędruje w górę od wymaganej capability.
    if let Some(pwid) = parent_world {
        let parent_caps = find_world_inner(&s, pwid)?;
        let parent_eff = parent_caps.caps.diff(s.global_revoked);

        for cap in initial.iter() {
            if !hierarchy::set_implies(parent_eff, cap) {
                return Err("child capability exceeds parent");
            }
        }
    }

    // Znajdź wolny slot
    let id = s.next_id;
    s.next_id = s.next_id.wrapping_add(1);

    let slot = s.worlds.iter_mut().find(|w| w.is_none())
        .ok_or("no free world slots")?;

    *slot = Some(WorldCaps {
        world_id: id,
        caps: initial,
        inherited_from: parent_world,
        revokable: true,
    });

    Ok(id)
}

fn find_world_inner(s: &StoreInner, world_id: u32) -> Result<WorldCaps, &'static str> {
    for w in &s.worlds {
        if let Some(wc) = w {
            if wc.world_id == world_id {
                return Ok(*wc);
            }
        }
    }
    Err("world not found")
}

/// Pobierz capabilities worlda
pub fn get_world_caps(world_id: u32) -> Result<CapabilitySet, &'static str> {
    let s = STORE.lock();
    let wc = find_world_inner(&s, world_id)?;
    Ok(wc.caps.diff(s.global_revoked))
}

/// Ustaw capabilities worlda (wymaga CAP_ADMIN)
pub fn set_world_caps(world_id: u32, new_caps: CapabilitySet) -> Result<(), &'static str> {
    let mut s = STORE.lock();

    // Sprawdź parent (surowy zbiór rodzica — patrz uwaga w register_world:
    // expand_hierarchy tutaj otwierałby furtkę eskalacji przez Root).
    let wc = find_world_inner(&s, world_id)?;
    if let Some(parent_id) = wc.inherited_from {
        let parent_caps = find_world_inner(&s, parent_id)?;
        let parent_eff = parent_caps.caps.diff(s.global_revoked);

        for cap in new_caps.iter() {
            if !hierarchy::set_implies(parent_eff, cap) {
                return Err("cannot grant capability exceeding parent");
            }
        }
    }

    for w in &mut s.worlds {
        if let Some(wc) = w {
            if wc.world_id == world_id {
                wc.caps = new_caps;
                return Ok(());
            }
        }
    }

    Err("world not found")
}

/// Dodaj capability do worlda
pub fn add_world_cap(world_id: u32, cap: Capability) -> Result<(), &'static str> {
    let current = get_world_caps(world_id)?;
    set_world_caps(world_id, current.add(cap))
}

/// Usuń capability z worlda
pub fn remove_world_cap(world_id: u32, cap: Capability) -> Result<(), &'static str> {
    let mut s = STORE.lock();

    for w in &mut s.worlds {
        if let Some(wc) = w {
            if wc.world_id == world_id {
                if !wc.revokable {
                    return Err("world capabilities are not revokable");
                }
                wc.caps = wc.caps.remove(cap);
                return Ok(());
            }
        }
    }

    Err("world not found")
}

/// Usuń world (cleanup)
pub fn unregister_world(world_id: u32) -> Result<(), &'static str> {
    let mut s = STORE.lock();

    for w in &mut s.worlds {
        if let Some(wc) = w {
            if wc.world_id == world_id {
                *w = None;
                return Ok(());
            }
        }
    }

    Err("world not found")
}

/// Sprawdź czy world ma capability (z uwzględnieniem global_revoked)
pub fn world_has_cap(world_id: u32, cap: Capability) -> bool {
    let s = STORE.lock();
    if let Ok(wc) = find_world_inner(&s, world_id) {
        let eff = wc.caps.diff(s.global_revoked);
        hierarchy::set_implies(eff, cap)
    } else {
        false
    }
}

/// Liczba aktywnych worlds
pub fn world_count() -> usize {
    let s = STORE.lock();
    s.worlds.iter().filter(|w| w.is_some()).count()
}

/// Iteracja po wszystkich worlds
pub fn iter_worlds<F: FnMut(u32, CapabilitySet)>(mut f: F) {
    let s = STORE.lock();
    for w in &s.worlds {
        if let Some(wc) = w {
            f(wc.world_id, wc.caps);
        }
    }
}

/// Globalne revocation (odbrane wszystkim)
pub fn add_global_revoked(cap: Capability) {
    let mut s = STORE.lock();
    s.global_revoked = s.global_revoked.add(cap);
}

pub fn remove_global_revoked(cap: Capability) {
    let mut s = STORE.lock();
    s.global_revoked = s.global_revoked.remove(cap);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_registration() {
        init_store().unwrap();

        let wid = register_world(None, sets::presets::standard_user()).unwrap();
        assert!(world_has_cap(wid, Capability::User));
        assert!(!world_has_cap(wid, Capability::Driver));

        // Child nie może mieć więcej niż parent
        let child_result = register_world(Some(wid),
            CapabilitySet::single(Capability::DevPci));
        assert!(child_result.is_err());
    }
}