// driverspace_workspace/crates/ds-registry/src/lib.rs
#![no_std]

pub mod database;
pub mod matching;
pub mod state;

pub use database::{DriverId, Registry};
pub use matching::find_driver_for_device;
pub use state::{DeviceState, DeviceStatus};