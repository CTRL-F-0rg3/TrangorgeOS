//! Domyślne zestawy capabilities per ring przy rejestracji worlda.

use super::types::CapabilitySet;
use super::sets;
use super::store;
use super::audit;

/// Domyślny zestaw dla danego ringu
pub fn default_for_ring(ring: u8) -> CapabilitySet {
    match ring {
        0 => sets::presets::kernel(),
        1 => sets::presets::driver(),
        2 => sets::presets::privileged_user(),
        _ => sets::presets::standard_user(),
    }
}

/// Zainstaluj world kernela (ring0) przy starcie
pub fn install_defaults() -> Result<(), &'static str> {
    // World 0 = kernel (pełne uprawnienia)
    store::register_world(None, sets::presets::kernel())?;
    audit::log_register(0);
    Ok(())
}

/// Zarejestruj world dla ringu (używane przy spawn/enter)
pub fn register_for_ring(ring: u8, parent: Option<u32>) -> Result<u32, &'static str> {
    let set = default_for_ring(ring);
    let wid = store::register_world(parent, set)?;
    audit::log_register(wid);
    Ok(wid)
}