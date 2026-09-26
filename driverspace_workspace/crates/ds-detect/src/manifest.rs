//! The on-image driver catalogue: what this image can drive, and when.
//!
//! The manifest is what makes "load only what is needed" possible. Without it
//! the system would have to start every driver and discover the ones that are
//! irrelevant; with it, the detection layer can answer "does this image even
//! have something for that device?" before touching the hardware.
//!
//! It is deliberately a plain data format (`TDXM`, the TrangorgeOS driver
//! eXecutble Manifest) so it can be produced by the build script, inspected
//! with `strings`, and diffed between builds.

use kapi_abi::{BusType, CapId, DeviceId};

/// Magic number at the head of a manifest: `TDXM`.
pub const MANIFEST_MAGIC: u32 = 0x4D58_4454;

/// Manifest format version understood by this code.
pub const MANIFEST_VERSION: u32 = 1;

/// Longest driver name a manifest entry may carry.
pub const NAME_LEN: usize = 32;

/// Longest class filter a manifest entry may carry, e.g. `"0401"`.
pub const FILTER_LEN: usize = 8;

/// When a driver should be started.
///
/// This is the policy knob that decides whether a device gets a driver at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LoadPolicy {
    /// Start only when a scanned device matches this entry.
    ///
    /// The default, and the reason the image can carry drivers for hardware
    /// that is not present.
    OnDemand = 0,
    /// Start unconditionally at boot, matched or not.
    ///
    /// For drivers with no enumerable device: a bus controller, a timer.
    Always = 1,
    /// Present in the image but never started automatically.
    Manual = 2,
}

impl LoadPolicy {
    #[inline]
    pub const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Always,
            2 => Self::Manual,
            _ => Self::OnDemand,
        }
    }
}

/// One entry: a driver the image carries, and what it claims to handle.
#[derive(Clone, Copy, Debug)]
pub struct ManifestEntry {
    /// Name the loader uses, e.g. `"Alsa-TrangorgeOS"`.
    pub name: [u8; NAME_LEN],
    /// Identifier the registry maps devices onto.
    pub driver_id: u32,
    /// Bus this driver serves.
    pub bus: BusType,
    /// `(vendor, device)` pair, or `0xFFFF` for a class-only entry.
    pub vendor_id: u16,
    pub device_id: u16,
    /// `DeviceId` this entry claims; `DeviceId::pci` by default.
    pub match_id: DeviceId,
    /// PCI class/subclass filter as hex text, or empty.
    pub class_filter: [u8; FILTER_LEN],
    /// Capabilities to grant before the driver's first request.
    pub capabilities: CapId,
    pub policy: LoadPolicy,
    /// Whether the driver must start even if no device matched.
    pub critical: bool,
}

impl ManifestEntry {
    /// Build an entry that claims exactly one `(vendor, device)` pair.
    pub const fn exact(name: &str, driver_id: u32, vendor_id: u16, device_id: u16) -> Self {
        Self {
            name: to_fixed(name),
            driver_id,
            bus: BusType::Pci,
            vendor_id,
            device_id,
            match_id: DeviceId::pci(vendor_id, device_id),
            class_filter: to_fixed(""),
            capabilities: CapId::NONE,
            policy: LoadPolicy::OnDemand,
            critical: false,
        }
    }

    /// Build an entry that claims a whole PCI class, e.g. every audio device.
    pub const fn by_class(name: &str, driver_id: u32, class: &str) -> Self {
        Self {
            name: to_fixed(name),
            driver_id,
            bus: BusType::Pci,
            vendor_id: 0xFFFF,
            device_id: 0xFFFF,
            match_id: DeviceId::pci(0xFFFF, 0xFFFF),
            class_filter: to_fixed(class),
            capabilities: CapId::NONE,
            policy: LoadPolicy::OnDemand,
            critical: false,
        }
    }

    /// The driver name, trimmed.
    pub fn name_str(&self) -> &str {
        str_field(&self.name)
    }

    /// The class filter, trimmed; empty when the entry matches by id only.
    pub fn class_filter_str(&self) -> &str {
        str_field(&self.class_filter)
    }

    /// Whether this entry claims `device`.
    ///
    /// An entry matches on **either** the exact `(vendor, device)` pair or the
    /// class filter, and never on both: a pair match is specific, a class match
    /// is a fallback for hardware whose id is not enumerated.
    pub fn matches(&self, device: &crate::hw::ScannedDevice) -> bool {
        if self.bus != device.bus_type() {
            return false;
        }
        if self.vendor_id != 0xFFFF && self.device_id != 0xFFFF {
            return device.id() == self.match_id;
        }
        let filter = self.class_filter_str();
        if filter.is_empty() {
            return false;
        }
        let Some(pci) = device.as_pci() else {
            return false;
        };
        filter_matches(filter, pci.class(), pci.subclass())
    }
}

/// Compare a `class`/`subclass` filter against a device.
///
/// `filter` is ALSA-style hex text: `"04"` (class only), `"0401"` (class and
/// subclass). A filter that is not hex matches nothing, because a typo that
/// silently matched everything would be worse than one matching nothing.
fn filter_matches(filter: &str, class: u8, subclass: u8) -> bool {
    let Ok(bits) = u32::from_str_radix(filter, 16) else {
        return false;
    };
    if bits == 0 {
        return false;
    }
    if bits <= 0xFF {
        return bits as u8 == class;
    }
    (bits as u16) == ((class as u16) << 8) | subclass as u16
}

/// Copy `value` into a fixed-size, zero-padded field.
const fn to_fixed<const N: usize>(value: &str) -> [u8; N] {
    let bytes = value.as_bytes();
    let mut field = [0u8; N];
    let mut index = 0;
    while index < bytes.len() && index < N {
        field[index] = bytes[index];
        index += 1;
    }
    field
}

/// Borrow a fixed-size, zero-padded field as `&str`.
fn str_field(field: &[u8]) -> &str {
    let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
    core::str::from_utf8(&field[..end]).unwrap_or("")
}

/// The whole catalogue, as carried by the image.
#[derive(Clone, Copy, Debug)]
pub struct Manifest {
    /// Bumped when the layout changes; the loader refuses a newer one.
    pub version: u32,
    pub entries: [Option<ManifestEntry>; Self::MAX_ENTRIES],
    /// How many `entries` slots are in use.
    pub count: usize,
}

impl Manifest {
    /// How many drivers one image may carry.
    pub const MAX_ENTRIES: usize = 16;

    /// An empty manifest.
    pub const fn empty() -> Self {
        Self { version: MANIFEST_VERSION, entries: [None; Self::MAX_ENTRIES], count: 0 }
    }

    /// Append an entry; returns `false` when the image is full.
    pub fn push(&mut self, entry: ManifestEntry) -> bool {
        if self.count >= Self::MAX_ENTRIES {
            return false;
        }
        self.entries[self.count] = Some(entry);
        self.count += 1;
        true
    }

    /// The entries in use, in image order.
    pub fn iter(&self) -> impl Iterator<Item = &ManifestEntry> {
        self.entries[..self.count].iter().filter_map(|slot| slot.as_ref())
    }

    /// Look a driver up by name.
    pub fn find(&self, name: &str) -> Option<&ManifestEntry> {
        self.iter().find(|entry| entry.name_str() == name)
    }

    /// Whether the image carries a driver that claims `device`.
    pub fn covers(&self, device: &crate::hw::ScannedDevice) -> bool {
        self.iter().any(|entry| entry.matches(device))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hw::{PciBar, PciDevice, ScannedDevice};

    fn audio_device() -> PciDevice {
        PciDevice {
            segment: 0,
            bus: 0,
            device: 0x1f,
            function: 0,
            vendor_id: 0x8086,
            device_id: 0x2415,
            class_code: 0x0401_00,
            bars: [PciBar::Absent; 6],
        }
    }

    #[test]
    fn an_exact_entry_matches_only_its_pair() {
        let entry = ManifestEntry::exact("Alsa-TrangorgeOS", 4, 0x8086, 0x2415);
        assert_eq!(entry.name_str(), "Alsa-TrangorgeOS");
        assert!(entry.matches(&ScannedDevice::Pci(audio_device())));

        // A different device id must not match.
        let other = ManifestEntry::exact("intel-hda", 3, 0x8086, 0x2668);
        assert!(!other.matches(&ScannedDevice::Pci(audio_device())));
    }

    #[test]
    fn a_class_entry_matches_by_class_code() {
        let entry = ManifestEntry::by_class("Alsa-TrangorgeOS", 4, "04");
        assert!(entry.matches(&ScannedDevice::Pci(audio_device())));

        let narrow = ManifestEntry::by_class("AudioOnly", 5, "0401");
        assert!(narrow.matches(&ScannedDevice::Pci(audio_device())));

        // A networking device is class 02, so no audio entry may claim it.
        let mut network = audio_device();
        network.class_code = 0x0200_00;
        assert!(!entry.matches(&ScannedDevice::Pci(network)));
    }

    #[test]
    fn a_non_hex_or_zero_filter_matches_nothing() {
        let mut entry = ManifestEntry::by_class("Broken", 9, "zz");
        assert!(!entry.matches(&ScannedDevice::Pci(audio_device())));
        // A zero filter would otherwise claim every device on the bus.
        entry.class_filter = to_fixed("0");
        assert!(!entry.matches(&ScannedDevice::Pci(audio_device())));
    }

    #[test]
    fn bus_must_agree_before_the_filter_is_consulted() {
        let entry = ManifestEntry::exact("Alsa-TrangorgeOS", 4, 0x8086, 0x2415);
        let usb = ScannedDevice::Usb { vendor_id: 0x8086, product_id: 0x2415, bus: 0, port: 0 };
        // Same ids, different bus: a PCI entry must not claim a USB device.
        assert!(!entry.matches(&usb));
    }

    #[test]
    fn a_full_image_refuses_more_entries() {
        let mut manifest = Manifest::empty();
        for index in 0..Manifest::MAX_ENTRIES {
            let entry = ManifestEntry::exact("drv", index as u32, 0x8086, index as u16);
            assert!(manifest.push(entry));
        }
        assert_eq!(manifest.count, Manifest::MAX_ENTRIES);
        assert!(!manifest.push(ManifestEntry::exact("overflow", 99, 0, 0)));
    }

    #[test]
    fn lookup_and_coverage_work_over_a_populated_manifest() {
        let mut manifest = Manifest::empty();
        manifest.push(ManifestEntry::by_class("Alsa-TrangorgeOS", 4, "04"));
        manifest.push(ManifestEntry::by_class("NetTrangorgeOS", 5, "02"));

        assert!(manifest.find("Alsa-TrangorgeOS").is_some());
        assert!(manifest.find("NetTrangorgeOS").is_some());
        assert!(manifest.find("Nothing").is_none());
        assert_eq!(manifest.iter().count(), 2);

        assert!(manifest.covers(&ScannedDevice::Pci(audio_device())));
        let mut network = audio_device();
        network.class_code = 0x0200_00;
        assert!(manifest.covers(&ScannedDevice::Pci(network)));

        let mut stranger = audio_device();
        stranger.class_code = 0x0300_00; // display
        assert!(!manifest.covers(&ScannedDevice::Pci(stranger)));
    }

    #[test]
    fn policies_round_trip() {
        assert_eq!(LoadPolicy::from_u8(0), LoadPolicy::OnDemand);
        assert_eq!(LoadPolicy::from_u8(1), LoadPolicy::Always);
        assert_eq!(LoadPolicy::from_u8(2), LoadPolicy::Manual);
        assert_eq!(LoadPolicy::from_u8(99), LoadPolicy::OnDemand);
    }
}
