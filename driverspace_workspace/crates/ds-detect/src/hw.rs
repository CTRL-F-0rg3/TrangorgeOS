//! Hardware topology: what the machine actually contains.
//!
//! The scanner in [`crate::scan`] produces these values; nothing here talks to
//! hardware directly, which is what makes the matching logic testable without
//! a PCI bus.

use kapi_abi::{BusType, DeviceId};

/// One device found on a PCI bus.
///
/// The identity is the PCI `(vendor, device)` pair, because that is what
/// `ds-registry` matches on; the bus coordinates are kept because two identical
/// cards must still be distinguishable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciDevice {
    /// PCI segment.
    pub segment: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    /// PCI class code, sub-class and programming interface, packed as
    /// `(class << 16) | (subclass << 8) | prog_if`.
    pub class_code: u32,
    /// The header's BARs, already read from the configuration space.
    pub bars: [PciBar; 6],
}

/// A 32-bit PCI base address register, decoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PciBar {
    /// Present, memory-mapped, with the base address and size.
    Memory { base: u64, size: u64 },
    /// Present, I/O ports, with the base and size.
    Io { base: u16, size: u16 },
    /// Not implemented, or a 64-bit upper half with no lower half.
    #[default]
    Absent,
}

impl PciBar {
    /// The memory base, or 0 for an I/O or absent BAR.
    pub const fn memory_base(&self) -> u64 {
        match *self {
            Self::Memory { base, .. } => base,
            _ => 0,
        }
    }

    /// Whether this BAR describes anything.
    pub const fn is_present(&self) -> bool {
        !matches!(self, Self::Absent)
    }
}

impl PciDevice {
    /// The packed requester id, `(segment << 16) | bdf`, as the IOMMU driver
    /// uses it, so one device needs no second identifier.
    pub const fn requester(&self) -> u32 {
        ((self.segment as u32) << 16)
            | ((self.bus as u32) << 8)
            | ((self.device as u32) << 3)
            | (self.function as u32)
    }

    /// The device identity `ds-registry` matches on.
    pub const fn id(&self) -> DeviceId {
        DeviceId::pci(self.vendor_id, self.device_id)
    }

    /// The broad class, `class_code >> 16`.
    pub const fn class(&self) -> u8 {
        (self.class_code >> 16) as u8
    }

    /// The sub-class, `(class_code >> 8) & 0xFF`.
    pub const fn subclass(&self) -> u8 {
        ((self.class_code >> 8) & 0xFF) as u8
    }

    /// `BDF` in the usual `bus:dev.func` spelling, for logs.
    pub fn address(&self) -> heapless::String<16> {
        let mut text = heapless::String::new();
        let _ = core::fmt::Write::write_fmt(
            &mut text,
            format_args!("{:02x}:{:02x}.{:x}", self.bus, self.device, self.function),
        );
        text
    }

    /// The bus this device sits on.
    pub const fn bus_type(&self) -> BusType {
        BusType::Pci
    }
}

/// A device that is not on PCI.
///
/// The audio stack also meets USB and platform (SoC) devices, and the registry
/// matches those too, so the model is not PCI-shaped even though the scanner
/// starts there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScannedDevice {
    /// A device on a PCI bus.
    Pci(PciDevice),
    /// A device reached over USB.
    Usb {
        vendor_id: u16,
        product_id: u16,
        bus: u8,
        port: u8,
    },
    /// A device described by the firmware rather than a bus.
    Platform {
        vendor_id: u16,
        device_id: u16,
        /// Firmware node path, e.g. `"/soc/audio@1f2"`.
        path: [u8; 48],
    },
}

impl ScannedDevice {
    /// The identity used for registry matching.
    pub const fn id(&self) -> DeviceId {
        match *self {
            Self::Pci(device) => device.id(),
            Self::Usb { vendor_id, product_id, .. } => DeviceId::usb(vendor_id, product_id),
            Self::Platform { vendor_id, device_id, .. } => DeviceId::pci(vendor_id, device_id),
        }
    }

    /// The bus this device was found on.
    pub const fn bus_type(&self) -> BusType {
        match self {
            Self::Pci(_) => BusType::Pci,
            Self::Usb { .. } => BusType::Usb,
            Self::Platform { .. } => BusType::Platform,
        }
    }

    /// The PCI view, when there is one.
    pub const fn as_pci(&self) -> Option<&PciDevice> {
        match self {
            Self::Pci(device) => Some(device),
            _ => None,
        }
    }

    /// A short description for logs: `pci 8086:2415` or `usb 1234:5678`.
    pub fn describe(&self) -> heapless::String<24> {
        let mut text = heapless::String::new();
        let (prefix, vendor, device) = match self {
            Self::Pci(d) => ("pci", d.vendor_id, d.device_id),
            Self::Usb { vendor_id, product_id, .. } => ("usb", *vendor_id, *product_id),
            Self::Platform { vendor_id, device_id, .. } => ("soc", *vendor_id, *device_id),
        };
        let _ = core::fmt::Write::write_fmt(
            &mut text,
            format_args!("{prefix} {vendor:04x}:{device:04x}"),
        );
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> PciDevice {
        PciDevice {
            segment: 0,
            bus: 0x2a,
            device: 5,
            function: 3,
            vendor_id: 0x8086,
            device_id: 0x2415,
            class_code: 0x0401_00,
            bars: [
                PciBar::Memory { base: 0xFEB0_0000, size: 0x1000 },
                PciBar::Io { base: 0x0060, size: 0x20 },
                PciBar::Absent,
                PciBar::Absent,
                PciBar::Absent,
                PciBar::Absent,
            ],
        }
    }

    #[test]
    fn requester_packs_segment_and_bdf() {
        let device = sample();
        // segment 0, bus 0x2a, device 5, function 3 -> 0x2a00 | (5 << 3) | 3.
        assert_eq!(device.requester(), 0x0000_2A2B);
        assert_eq!(device.requester() & 0xFFFF, 0x2A2B);
    }

    #[test]
    fn identity_matches_what_the_registry_expects() {
        let device = sample();
        assert_eq!(device.id(), DeviceId::pci(0x8086, 0x2415));
        assert_eq!(device.id().vendor(), 0x8086);
        assert_eq!(device.id().device(), 0x2415);
    }

    #[test]
    fn class_code_is_split_correctly() {
        let device = sample();
        assert_eq!(device.class(), 0x04, "multimedia");
        assert_eq!(device.subclass(), 0x01, "audio device");
    }

    #[test]
    fn bars_decode_and_ignore_absent_ones() {
        let device = sample();
        assert_eq!(device.bars[0].memory_base(), 0xFEB0_0000);
        assert!(device.bars[0].is_present());
        assert!(device.bars[1].is_present());
        assert!(!device.bars[2].is_present());
        assert_eq!(device.bars[5].memory_base(), 0);
    }

    #[test]
    fn non_pci_devices_carry_their_own_identity() {
        let usb = ScannedDevice::Usb { vendor_id: 0x1234, product_id: 0x5678, bus: 1, port: 2 };
        assert_eq!(usb.id(), DeviceId::usb(0x1234, 0x5678));
        assert_eq!(usb.bus_type(), BusType::Usb);
        assert!(usb.as_pci().is_none());
        assert!(usb.describe().starts_with("usb"));

        let soc = ScannedDevice::Platform {
            vendor_id: 0x8086,
            device_id: 0x2415,
            path: [0u8; 48],
        };
        assert_eq!(soc.bus_type(), BusType::Platform);
        assert!(soc.describe().starts_with("soc"));
    }

    #[test]
    fn address_is_the_usual_bdf_spelling() {
        assert_eq!(sample().address().as_str(), "2a:05.3");
    }
}
