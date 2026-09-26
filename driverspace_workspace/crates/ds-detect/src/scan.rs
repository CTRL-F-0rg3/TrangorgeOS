//! The PCI scanner: walk the buses and read the configuration space.
//!
//! This is the only part of the layer that talks to hardware. It is behind a
//! trait so the rest of `ds-detect` - and the planner in particular - can be
//! tested against a recorded bus instead of a live one.
//!
//! ## What it reads, and why only that
//!
//! The scanner reads the vendor and device ids, the class code and the BARs.
//! It deliberately does **not** probe vendor registers: a driver that needs
//! more than that should do it itself after being granted the mapping. Probing
//! unknown registers at scan time is how a detection layer turns a harmless
//! machine into a crashing one.

use heapless::Vec;

use crate::hw::{PciBar, PciDevice, ScannedDevice};

/// How many devices one scan may report.
///
/// Generous for a modern machine, and a hard bound so the planner's fixed-size
/// tables cannot be overrun by a malformed bus.
pub const MAX_DEVICES: usize = 16;

/// Reads PCI configuration space.
pub trait PciBus {
    /// Read one 32-bit configuration register.
    ///
    /// `offset` is the dword offset (register number times four), as in the
    /// x86 CONFIG_ADDRESS port protocol.
    fn read32(&self, bus: u8, device: u8, function: u8, offset: u8) -> Option<u32>;
}

/// PCI configuration-space register offsets used by the scanner.
mod reg {
    pub const VENDOR_ID: u8 = 0x00;
    pub const CLASS_REV: u8 = 0x08;
    pub const HEADER_TYPE: u8 = 0x0E;
    pub const BAR0: u8 = 0x10;
}

/// Scan every bus behind `bus` and report the devices found.
///
/// A device counts as present when the vendor id is neither `0xFFFF` (absent)
/// nor `0x0000` (a floating bus with nothing decoded).
pub fn scan_pci<B: PciBus>(bus: &B, bus_numbers: &[u8]) -> Vec<ScannedDevice, MAX_DEVICES> {
    let mut found: Vec<ScannedDevice, MAX_DEVICES> = Vec::new();

    for &bus_number in bus_numbers {
        for device in 0..32u8 {
            // Function 0 is always probed; the rest only when the header says
            // this is a multi-function device.
            let Some(header) = bus.read32(bus_number, device, 0, reg::HEADER_TYPE) else {
                continue;
            };
            let multifunction = header & 0x80 != 0;
            let functions = if multifunction { 8u8 } else { 1u8 };

            for function in 0..functions {
                let Some(vendor_device) =
                    bus.read32(bus_number, device, function, reg::VENDOR_ID)
                else {
                    continue;
                };
                let vendor_id = (vendor_device & 0xFFFF) as u16;
                let device_id = (vendor_device >> 16) as u16;
                if vendor_id == 0xFFFF || vendor_id == 0x0000 {
                    continue;
                }
                let Some(class_rev) =
                    bus.read32(bus_number, device, function, reg::CLASS_REV)
                else {
                    continue;
                };
                let found_device = read_device(
                    bus,
                    bus_number,
                    device,
                    function,
                    vendor_id,
                    device_id,
                    class_rev,
                );
                if found.push(ScannedDevice::Pci(found_device)).is_err() {
                    // The image's table is full; stop rather than truncate
                    // silently, and let the planner report the difference.
                    return found;
                }
            }
        }
    }
    found
}

/// Read one function's full descriptor.
fn read_device<B: PciBus>(
    bus: &B,
    bus_number: u8,
    device: u8,
    function: u8,
    vendor_id: u16,
    device_id: u16,
    class_rev: u32,
) -> PciDevice {
    // Register 0x08 holds the revision in bits 7:0, the sub-class in 15:8 and
    // the class in 23:16. The framework wants a single packed value, so shift
    // the two fields up and leave the revision out entirely.
    let class_code = class_rev & 0xFFFF_FF00;

    let mut bars = [PciBar::Absent; 6];
    for index in 0..bars.len() {
        let offset = reg::BAR0 + (index as u8) * 4;
        if let Some(raw) = bus.read32(bus_number, device, function, offset) {
            bars[index] = decode_bar(raw);
        }
    }

    PciDevice {
        segment: 0,
        bus: bus_number,
        device,
        function,
        vendor_id,
        device_id,
        class_code,
        bars,
    }
}

/// Decode one BAR.
///
/// The low four bits are type and interrupt-bit flags, not address, so they
/// are masked off before the base is used.
fn decode_bar(raw: u32) -> PciBar {
    // Bit 0: 0 = memory, 1 = I/O. Bit 1: 0 = 32-bit, 1 = 64-bit.
    if raw & 1 == 0 {
        let base = (raw & 0xFFFF_FFF0) as u64;
        if base == 0 {
            return PciBar::Absent;
        }
        PciBar::Memory { base, size: 0 }
    } else {
        let base = ((raw >> 2) & 0xFFFF) as u16;
        if base == 0 {
            return PciBar::Absent;
        }
        PciBar::Io { base, size: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // The crate is `no_std`, so the test harness needs an explicit `extern
    // crate std;` - without it `BTreeMap` below has nowhere to come from.
    extern crate std;
    use std::collections::BTreeMap;

    /// A configuration space backed by a map, so a test can describe a bus.
    struct FakeBus {
        registers: BTreeMap<(u8, u8, u8, u8), u32>,
    }

    impl FakeBus {
        fn new() -> Self {
            Self { registers: BTreeMap::new() }
        }

        /// Describe a function: `(bus, device, function) -> (vendor, device, class)`.
        ///
        /// `class` is the packed framework value, e.g. `0x0401` for class 04
        /// sub-class 01; it is written into register 0x08 the way the hardware
        /// stores it, with the sub-class in bits 15:8 and the class in 23:16.
        fn add_function(
            &mut self,
            bus: u8,
            device: u8,
            function: u8,
            vendor: u16,
            device_id: u16,
            class: u32,
        ) {
            self.registers.insert(
                (bus, device, function, reg::VENDOR_ID),
                vendor as u32 | ((device_id as u32) << 16),
            );
            // Register 0x08: revision in 7:0, sub-class in 15:8, class in
            // 23:16. `class` arrives as the packed framework value `0xCCSS`,
            // so the class byte moves to 23:16 and the sub-class byte to 15:8.
            self.registers.insert(
                (bus, device, function, reg::CLASS_REV),
                ((class >> 8) & 0xFF) << 16 | (class & 0xFF) << 8,
            );
            self.registers.insert((bus, device, function, reg::HEADER_TYPE), 0);
        }

        /// Set a BAR value.
        fn set_bar(&mut self, bus: u8, device: u8, function: u8, index: u8, value: u32) {
            self.registers.insert((bus, device, function, reg::BAR0 + index * 4), value);
        }

        /// Mark a slot as multi-function.
        fn set_multifunction(&mut self, bus: u8, device: u8) {
            self.registers.insert((bus, device, 0, reg::HEADER_TYPE), 0x80);
        }
    }

    impl PciBus for FakeBus {
        fn read32(&self, bus: u8, device: u8, function: u8, offset: u8) -> Option<u32> {
            self.registers.get(&(bus, device, function, offset)).copied()
        }
    }

    #[test]
    fn a_simple_device_is_found() {
        let mut bus = FakeBus::new();
        bus.add_function(0, 0x1f, 0, 0x8086, 0x2415, 0x0401);
        // A real memory BAR has its low four bits clear (or, with UACPI, the
        // prefetch bit set); 0xFEB0_0000 on its own would decode as an I/O
        // port, because bit 0 selects the BAR type.
        bus.set_bar(0, 0x1f, 0, 0, 0xFEB0_0004);

        let found = scan_pci(&bus, &[0]);
        assert_eq!(found.len(), 1);
        let Some(ScannedDevice::Pci(device)) = found.first() else {
            panic!("expected a PCI device");
        };
        assert_eq!(device.vendor_id, 0x8086);
        assert_eq!(device.device_id, 0x2415);
        assert_eq!(device.class(), 0x04);
        assert_eq!(device.subclass(), 0x01);
        assert_eq!(device.bars[0].memory_base(), 0xFEB0_0000);
    }

    #[test]
    fn absent_slots_are_skipped() {
        let mut bus = FakeBus::new();
        // Vendor 0xFFFF is an absent device.
        bus.registers.insert((0, 0x00, 0, reg::VENDOR_ID), 0xFFFF_FFFF);
        bus.add_function(0, 0x1f, 0, 0x8086, 0x2415, 0x0401);

        let found = scan_pci(&bus, &[0]);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn a_multifunction_slot_yields_every_function() {
        let mut bus = FakeBus::new();
        // `add_function` writes a zero header type, so the multi-function flag
        // has to be set *after* the functions are described.
        bus.add_function(0, 0x1f, 0, 0x8086, 0x2415, 0x0401);
        bus.add_function(0, 0x1f, 1, 0x8086, 0x2425, 0x0401);
        bus.set_multifunction(0, 0x1f);

        let found = scan_pci(&bus, &[0]);
        assert_eq!(found.len(), 2, "both functions of a multi-function slot");
    }

    #[test]
    fn a_single_function_device_probes_only_function_zero() {
        let mut bus = FakeBus::new();
        bus.add_function(0, 0x1f, 0, 0x8086, 0x2415, 0x0401);
        // Described but not flagged multi-function, so it does not exist.
        bus.add_function(0, 0x1f, 1, 0x8086, 0x2425, 0x0401);

        let found = scan_pci(&bus, &[0]);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn several_buses_are_all_scanned() {
        let mut bus = FakeBus::new();
        bus.add_function(0, 0x1f, 0, 0x8086, 0x2415, 0x0401);
        bus.add_function(1, 0x00, 0, 0x1234, 0x1111, 0x0300);

        let found = scan_pci(&bus, &[0, 1]);
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn bar_decoding_separates_memory_from_io() {
        // Bit 0 clear: memory. Bit 0 set: I/O ports, address in bits 15:2.
        assert!(matches!(decode_bar(0xFEB0_0000), PciBar::Memory { .. }));
        match decode_bar(0x0000_0181) {
            PciBar::Io { base, .. } => assert_eq!(base, 0x0060),
            other => panic!("expected an I/O BAR, got {other:?}"),
        }
        // A zero BAR is not implemented.
        assert_eq!(decode_bar(0), PciBar::Absent);
    }

    #[test]
    fn a_memory_bar_with_the_prefetch_bit_still_decodes_as_memory() {
        // Bit 2 is the UACPI prefetch indicator, not part of the address, and
        // bit 0 must stay clear or the BAR reads as I/O instead.
        match decode_bar(0xFEB0_0004) {
            PciBar::Memory { base, .. } => assert_eq!(base, 0xFEB0_0000),
            other => panic!("expected a memory BAR, got {other:?}"),
        }
    }
}
