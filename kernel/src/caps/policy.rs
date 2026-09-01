use super::types::{Capability, CapResult, CapabilityError};
use super::store;
use core::sync::atomic::{AtomicPtr, Ordering};

pub type PolicyHook = fn(world: u32, cap: Capability) -> bool;

static HOOK: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());

pub fn set_hook(h: PolicyHook) {
    let raw: *mut () = unsafe { core::mem::transmute(h as PolicyHook) };
    HOOK.store(raw, Ordering::Relaxed);
}

fn policy_allows(world: u32, cap: Capability) -> bool {
    let raw = HOOK.load(Ordering::Relaxed);
    if raw.is_null() {
        return true;
    }
    let hook: PolicyHook = unsafe { core::mem::transmute(raw) };
    hook(world, cap)
}

pub fn enforce(world: u32, cap: Capability) -> CapResult<()> {
    if !store::world_has_cap(world, cap) {
        return Err(CapabilityError { required: cap, world_id: Some(world) });
    }

    if !policy_allows(world, cap) {
        return Err(CapabilityError { required: cap, world_id: Some(world) });
    }

    Ok(())
}

pub fn enforce_self(cap: Capability) -> CapResult<()> {
    let wid = super::check::current_world_id_pub();
    enforce(wid, cap)
}

pub fn allowed(world: u32, cap: Capability) -> bool {
    enforce(world, cap).is_ok()
}