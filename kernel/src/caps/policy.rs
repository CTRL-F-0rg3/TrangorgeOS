//! Most między capabilities a polityką (`policy/`).
//!
//! capabilities = co WOLNO (statycznie, per-world)
//! policy       = co ZROBIĆ (dynamicznie, reguły)
//!
//! Decyzja końcowa = capability OK AND policy OK.

use super::types::{Capability, CapResult, CapabilityError};
use super::store;

/// Hook polityki (ustawiany przez policy/)
pub type PolicyHook = fn(world: u32, cap: Capability) -> bool;

static mut HOOK: Option<PolicyHook> = None;

/// Zarejestruj hook polityki
pub fn set_hook(h: PolicyHook) {
    unsafe { HOOK = Some(h); }
}

/// Czy polityka zezwala (domyślnie tak)
fn policy_allows(world: u32, cap: Capability) -> bool {
    unsafe {
        match HOOK {
            Some(h) => h(world, cap),
            None => true,
        }
    }
}

/// Pełna decyzja: capability + polityka
pub fn enforce(world: u32, cap: Capability) -> CapResult<()> {
    if !store::world_has_cap(world, cap) {
        return Err(CapabilityError { required: cap, world_id: Some(world) });
    }

    if !policy_allows(world, cap) {
        return Err(CapabilityError { required: cap, world_id: Some(world) });
    }

    Ok(())
}

/// Wersja dla current world
pub fn enforce_self(cap: Capability) -> CapResult<()> {
    let wid = super::check::current_world_id_pub();
    enforce(wid, cap)
}

/// Czy operacja dozwolona (bool, bez error)
pub fn allowed(world: u32, cap: Capability) -> bool {
    enforce(world, cap).is_ok()
}