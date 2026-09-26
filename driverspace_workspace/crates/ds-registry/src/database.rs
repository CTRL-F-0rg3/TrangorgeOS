// driverspace_workspace/crates/ds-registry/src/database.rs

pub type DriverId = u32;
pub struct RegistryEntry {
    pub vendor_id: u16,
    pub device_id: u16,
    pub driver_id: DriverId,
    pub driver_name: &'static str,
}

pub struct Registry {
    entries: &'static [RegistryEntry],
}

impl Registry {
    pub const fn new(entries: &'static [RegistryEntry]) -> Self {
        Self { entries }
    }

    pub fn entries(&self) -> &[RegistryEntry] {
        self.entries
    }

    pub fn find(&self, vendor_id: u16, device_id: u16) -> Option<&RegistryEntry> {
        self.entries.iter().find(|e| e.vendor_id == vendor_id && e.device_id == device_id)
    }
}

pub static DEFAULT_REGISTRY: Registry = Registry::new(&[
    RegistryEntry {
        vendor_id: 0x1234,
        device_id: 0x1111,
        driver_id: 1,
        driver_name: "qemu-vga",
    },
    RegistryEntry {
        vendor_id: 0xCAFE,
        device_id: 0xBABE,
        driver_id: 2,
        driver_name: "vgpu",
    },
    RegistryEntry {
        vendor_id: 0x8086,
        device_id: 0x2668,
        driver_id: 3,
        driver_name: "intel-hda",
    },
    // Intel 82801 (AC97 / ICH4), 82801EB (ICH5) and the AMD Hudson family 10h
    // controller: the audio function of the southbridge, which is what
    // `Alsa-TrangorgeOS` drives through its AC97 backend.
    RegistryEntry {
        vendor_id: 0x8086,
        device_id: 0x2415,
        driver_id: 4,
        driver_name: "Alsa-TrangorgeOS",
    },
    RegistryEntry {
        vendor_id: 0x8086,
        device_id: 0x2425,
        driver_id: 4,
        driver_name: "Alsa-TrangorgeOS",
    },
    RegistryEntry {
        vendor_id: 0x1022,
        device_id: 0x7807,
        driver_id: 4,
        driver_name: "Alsa-TrangorgeOS",
    },
]);