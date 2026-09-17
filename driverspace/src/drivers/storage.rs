use driverspacelib as ds;
use ds::abi::DsError;
use ds::driver::{DeviceInfo, Driver};

/// Storage driver stub. It only satisfies the driver registry so that
/// driver space can boot; a real disk driver can slot in later.
pub struct StorageDrv {
    registered: bool,
}

impl StorageDrv {
    pub const fn new() -> Self {
        Self { registered: false }
    }
}

impl Driver for StorageDrv {
    fn init(&mut self, _info: &DeviceInfo) -> Result<(), DsError> {
        self.registered = true;
        Ok(())
    }
}