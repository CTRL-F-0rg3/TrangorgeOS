// driverspace_workspace/crates/ds-registry/src/state.rs

/// Stan urządzenia w systemie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    /// Urządzenie wykryte, ale sterownik jeszcze nie załadowany.
    Discovered,
    /// Sterownik załadowany i zainicjalizowany.
    Attached,
    /// Urządzenie odłączone (hotunplug lub błąd).
    Detached,
    /// Wystąpił błąd krytyczny (sterownik się wysypał, sprzęt nie odpowiada).
    Error,
}

/// Struktura przechowująca stan konkretnego urządzenia.
pub struct DeviceState {
    pub vendor_id: u16,
    pub device_id: u16,
    pub status: DeviceStatus,
    pub driver_id: Option<u32>, // ID sterownika, który obsługuje to urządzenie
}

impl DeviceState {
    pub const fn new(vendor_id: u16, device_id: u16) -> Self {
        Self {
            vendor_id,
            device_id,
            status: DeviceStatus::Discovered,
            driver_id: None,
        }
    }

    pub fn set_attached(&mut self, driver_id: u32) {
        self.status = DeviceStatus::Attached;
        self.driver_id = Some(driver_id);
    }

    pub fn set_detached(&mut self) {
        self.status = DeviceStatus::Detached;
        self.driver_id = None;
    }

    pub fn set_error(&mut self) {
        self.status = DeviceStatus::Error;
    }
}