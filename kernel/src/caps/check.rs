//! Runtime capability checks: require_cap(), has_cap(), guards.

use super::types::{Capability, CapabilitySet, CapabilityError, CapResult};
use super::store;
use super::audit;
use core::sync::atomic::{AtomicU32, Ordering};

/// World kernela: pierwszy zarejestrowany przez `defaults::install_defaults`
/// (store nadaje ID od 1). Aktualizowany przy instalacji domyślnych worldów.
static KERNEL_WORLD: AtomicU32 = AtomicU32::new(1);

/// Bieżący world (kontekst wykonania). Ustawiany przy wejściu w world —
/// integracja z `trampoline_rings` nastąpi w ramach portu x86_64 ringów;
/// dopóki to punkt konfigurowany ręcznie (`set_current_world`).
static CURRENT_WORLD: AtomicU32 = AtomicU32::new(1);

/// ID worlda kernela (ma pełny zestaw capabilities).
pub fn kernel_world_id() -> u32 {
    KERNEL_WORLD.load(Ordering::Relaxed)
}

/// Rejestruj ID worlda kernela (wywoływane przez defaults::install_defaults).
pub fn set_kernel_world_id(id: u32) {
    KERNEL_WORLD.store(id, Ordering::Relaxed);
}

/// ID bieżącego worlda (publikowane dla syscalls/export/policy).
pub fn current_world_id_pub() -> u32 {
    CURRENT_WORLD.load(Ordering::Relaxed)
}

/// Przełącz bieżący world (punkt integracji z trampoline_rings / schedulerem).
pub fn set_current_world(world_id: u32) {
    CURRENT_WORLD.store(world_id, Ordering::Relaxed);
}

/// Current world ID (wewnętrzny alias).
fn current_world_id() -> u32 {
    current_world_id_pub()
}

/// Sprawdź czy current world ma capability
pub fn has_cap(cap: Capability) -> bool {
    let wid = current_world_id();
    store::world_has_cap(wid, cap)
}

/// Sprawdź czy dany world ma capability
pub fn world_has_cap(world_id: u32, cap: Capability) -> bool {
    store::world_has_cap(world_id, cap)
}

/// Wymagaj capability (zwraca error jeśli brak)
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

/// Wymagaj capability od konkretnego worlda
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

/// Wymagaj wielu capabilities
pub fn require_caps(caps: &[Capability]) -> CapResult<()> {
    for &cap in caps {
        require_cap(cap)?;
    }
    Ok(())
}

/// Guard: wykonaj `f` jeśli ma capability, inaczej error
pub fn with_cap<T, F>(cap: Capability, f: F) -> CapResult<T>
where
    F: FnOnce() -> T,
{
    require_cap(cap)?;
    Ok(f())
}

/// Guard: wykonaj `f` jeśli ma WSZYSTKIE capabilities
pub fn with_caps<T, F>(caps: &[Capability], f: F) -> CapResult<T>
where
    F: FnOnce() -> T,
{
    require_caps(caps)?;
    Ok(f())
}

/// Macro dla wygodnego require
#[macro_export]
macro_rules! require_cap {
    ($cap:expr) => {
        $crate::caps::check::require_cap($cap)?
    };
}

/// Macro dla wielu capabilities
#[macro_export]
macro_rules! require_caps {
    ($($cap:expr),+) => {
        $crate::caps::check::require_caps(&[$($cap),+])?
    };
}

/// Assert capability (panic jeśli brak - tylko dla debug)
pub fn assert_cap(cap: Capability) {
    if !has_cap(cap) {
        panic!("CAPABILITY VIOLATION: {} required", cap.name());
    }
}

/// Check bez auditu (dla hot paths)
#[inline(always)]
pub fn fast_check(cap: Capability) -> bool {
    store::world_has_cap(current_world_id(), cap)
}

/// Conditional: jeśli ma capability, wykonaj f, inaczej default
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

/// Tymczasowe rozszerzenie capabilities (w bloku)
pub struct TemporaryCaps {
    world_id: u32,
    original: CapabilitySet,
}

impl TemporaryCaps {
    pub fn enter(extra: CapabilitySet) -> CapResult<Self> {
        let wid = current_world_id();
        let original = store::get_world_caps(wid)
            .map_err(|_| CapabilityError { required: Capability::User, world_id: Some(wid) })?;

        // Tylko jeśli current world ma capability do modyfikacji własnych caps
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

        // Symuluj że to current world
        // (w teście pomijamy trampoline, używamy world_has_cap)
        assert!(store::world_has_cap(wid, Capability::User));
        assert!(!store::world_has_cap(wid, Capability::Driver));
    }
}