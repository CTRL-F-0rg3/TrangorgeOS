

use crate::database::{DriverId, DEFAULT_REGISTRY};

pub fn find_driver_for_device(vendor_id: u16, device_id: u16) -> Option<DriverId> {
    DEFAULT_REGISTRY.find(vendor_id, device_id).map(|entry| entry.driver_id)
}

/// Zwraca nazwę sterownika dla danego ID.
pub fn get_driver_name(driver_id: DriverId) -> Option<&'static str> {
    DEFAULT_REGISTRY.entries().iter()
        .find(|e| e.driver_id == driver_id)
        .map(|e| e.driver_name)
}