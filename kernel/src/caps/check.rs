use super::types::{Capability, CapabilitySet, CapabilityError, CapResult};
use super::store;
use super::audit;
use core::sync::atomic::{AtomicU32, Ordering};

static KERNEL_WORLD: AtomicU32 = AtomicU32::new(1);

static CURRENT_WORLD: AtomicU32 = AtomicU32::new(1);

pub fn kernel_world_id() -> u32 {
    KERNEL_WORLD.load(Ordering::Relaxed)
}

pub fn set_kernel_world_id(id: u32) {
    KERNEL_WORLD.store(id, Ordering::Relaxed);
}

pub fn current_world_id_pub() -> u32 {
    CURRENT_WORLD.load(Ordering::Relaxed)
}

pub fn set_current_world(world_id: u32) {
    CURRENT_WORLD.store(world_id, Ordering::Relaxed);
}

fn current_world_id() -> u32 {
    current_world_id_pub()
}

pub fn has_cap(cap: Capability) -> bool {
    let wid = current_world_id();
    store::world_has_cap(wid, cap)
}

pub fn world_has_cap(world_id: u32, cap: Capability) -> bool {
    store::world_has_cap(world_id, cap)
}

pub fn require_cap(cap: Capability) -> CapResult<()> {
    let wid = current_world_id();

    if store::world_has_cap(wid, cap) {
        audit::log_check(wid, cap, true);
        Ok(())
    } else {
        audit::log_check(wid, cap, false);
        Err(CapabilityError {
            required: cap,
            world_id: Some(wid),
        })
    }
}

pub fn require_world_cap(world_id: u32, cap: Capability) -> CapResult<()> {
    if store::world_has_cap(world_id, cap) {
        audit::log_check(world_id, cap, true);
        Ok(())
    } else {
        audit::log_check(world_id, cap, false);
        Err(CapabilityError {
            required: cap,
            world_id: Some(world_id),
        })
    }
}

pub fn require_caps(caps: &[Capability]) -> CapResult<()> {
    for &cap in caps {
        require_cap(cap)?;
    }
    Ok(())
}

pub fn with_cap<T, F>(cap: Capability, f: F) -> CapResult<T>
where
    F: FnOnce() -> T,
{
    require_cap(cap)?;
    Ok(f())
}

pub fn with_caps<T, F>(caps: &[Capability], f: F) -> CapResult<T>
where
    F: FnOnce() -> T,
{
    require_caps(caps)?;
    Ok(f())
}

#[macro_export]
macro_rules! require_cap {
    ($cap:expr) => {
        $crate::caps::check::require_cap($cap)?
    };
}

#[macro_export]
macro_rules! require_caps {
    ($($cap:expr),+) => {
        $crate::caps::check::require_caps(&[$($cap),+])?
    };
}

pub fn assert_cap(cap: Capability) {
    if !has_cap(cap) {
        panic!("CAPABILITY VIOLATION: {} required", cap.name());
    }
}

#[inline(always)]
pub fn fast_check(cap: Capability) -> bool {
    store::world_has_cap(current_world_id(), cap)
}

pub fn if_cap<T, F>(cap: Capability, f: F, default: T) -> T
where
    F: FnOnce() -> T,
{
    if has_cap(cap) {
        f()
    } else {
        default
    }
}

pub struct TemporaryCaps {
    world_id: u32,
    original: CapabilitySet,
}

impl TemporaryCaps {
    pub fn enter(extra: CapabilitySet) -> CapResult<Self> {
        let wid = current_world_id();
        let original = store::get_world_caps(wid)
            .map_err(|_| CapabilityError { required: Capability::User, world_id: Some(wid) })?;

        require_cap(Capability::Admin)?;

        store::set_world_caps(wid, original.union(extra))
            .map_err(|_| CapabilityError { required: Capability::Admin, world_id: Some(wid) })?;

        Ok(Self { world_id: wid, original })
    }
}

impl Drop for TemporaryCaps {
    fn drop(&mut self) {
        let _ = store::set_world_caps(self.world_id, self.original);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caps::sets;

    #[test]
    fn test_require() {
        crate::caps::init().unwrap();

        let wid = store::register_world(None, sets::presets::standard_user()).unwrap();

        assert!(store::world_has_cap(wid, Capability::User));
        assert!(!store::world_has_cap(wid, Capability::Driver));
    }
}