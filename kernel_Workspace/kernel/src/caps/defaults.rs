use super::types::CapabilitySet;
use super::sets;
use super::store;
use super::audit;

pub fn default_for_ring(ring: u8) -> CapabilitySet {
    match ring {
        0 => sets::presets::kernel(),
        1 => sets::presets::driver(),
        2 => sets::presets::privileged_user(),
        _ => sets::presets::standard_user(),
    }
}

pub fn install_defaults() -> Result<(), &'static str> {
    let id = store::register_world(None, sets::presets::kernel())?;
    crate::caps::check::set_kernel_world_id(id);
    audit::log_register(id);
    Ok(())
}

pub fn register_for_ring(ring: u8, parent: Option<u32>) -> Result<u32, &'static str> {
    let set = default_for_ring(ring);
    let wid = store::register_world(parent, set)?;
    audit::log_register(wid);
    Ok(wid)
}