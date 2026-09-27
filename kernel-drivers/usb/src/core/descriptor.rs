//! USB descriptors, as they come off the wire.
//!
//! Ported from `include/uapi/linux/usb/ch9.h` in the reference tree. Every offset
//! here is checked against the kernel's own structure definitions rather than
//! against a datasheet, because the whole point of the port is that the layouts
//! agree.
//!
//! # The bug this fixes
//!
//! The previous `core/descriptor.rs` read `bNumEndpoints` from **offset 8** of an
//! interface descriptor. Offset 8 is `iInterface` — the string index. `bNumEndpoints`
//! is offset **4**.
//!
//! Every HID and mass-storage interface in the tree has `bNumEndpoints` at or near
//! 4 and `iInterface` at or near 0, so the old parser reported a keyboard as
//! having zero endpoints and a disk as having one. A driver that then looked for
//! the interrupt endpoint it had been told did not exist simply never read the
//! keyboard, and no error was reported anywhere — the device was attached,
//! enumerated, and silent.
//!
//! # Why these are parsed into structs
//!
//! Because the wire format is little-endian and fixed-offset: the sizes are
//! declared in the first byte, and a field read from the wrong offset yields a
//! plausible number rather than a fault.

/// `struct usb_device_descriptor`, 18 bytes.
pub const DEVICE_DESC_LEN: usize = 18;
/// `struct usb_config_descriptor`, 9 bytes.
pub const CONFIG_DESC_LEN: usize = 9;
/// `struct usb_interface_descriptor`, 9 bytes.
pub const INTERFACE_DESC_LEN: usize = 9;
/// `struct usb_endpoint_descriptor`, 7 bytes without the audio extension.
pub const ENDPOINT_DESC_LEN: usize = 7;
/// The audio extension's extra `bRefresh` and `bSynchAddress`.
pub const ENDPOINT_AUDIO_DESC_LEN: usize = 9;

/// `bDescriptorType` for a device descriptor.
pub const DT_DEVICE: u8 = 1;
/// `bDescriptorType` for a configuration descriptor.
pub const DT_CONFIG: u8 = 2;
/// `bDescriptorType` for a string descriptor.
pub const DT_STRING: u8 = 3;
/// `bDescriptorType` for an interface descriptor.
pub const DT_INTERFACE: u8 = 4;
/// `bDescriptorType` for an endpoint descriptor.
pub const DT_ENDPOINT: u8 = 5;
/// `bDescriptorType` for a device qualifier.
pub const DT_DEVICE_QUALIFIER: u8 = 6;

/// `USB_DIR_OUT`: host-to-device traffic.
pub const DIR_OUT: u8 = 0x00;
/// `USB_DIR_IN`: device-to-host traffic.
pub const DIR_IN: u8 = 0x80;

/// `USB_ENDPOINT_XFER_CONTROL`.
pub const XFER_CONTROL: u8 = 0;
/// `USB_ENDPOINT_XFER_ISOC`.
pub const XFER_ISOC: u8 = 1;
/// `USB_ENDPOINT_XFER_BULK`.
pub const XFER_BULK: u8 = 2;
/// `USB_ENDPOINT_XFER_INT`.
pub const XFER_INT: u8 = 3;

/// The mask that extracts the transfer type from `bmAttributes`.
pub const XFER_TYPE_MASK: u8 = 0x03;
/// The mask for the `wMaxPacketSize` high-bandwidth multiplier bits.
///
/// Three bits, because the spec reserves the top of the field and the count is
/// one-based: 1 to 3 transactions per microframe.
pub const MAXPACKET_MULT_MASK: u8 = 0x07;
/// `wMaxPacketSize` bits 14:11: the high-bandwidth "additional transactions".
///
/// Bit 15 is reserved; the count is one-based and lives in bits 14:11, so the
/// field starts at bit **11**. Shifting by 12 instead puts the multiplier inside
/// the packet size: 512 bytes with two transactions is `0x0200 | (2 << 11)`, and
/// reading it as a 12-bit shift reports 1024 for an endpoint that moves 512.
pub const MAXPACKET_ADDITIONAL_SHIFT: u32 = 11;

/// The device's identity, from `struct usb_device_descriptor`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeviceDesc {
    /// `bLength`.
    pub length: u8,
    /// `bcdUSB`, the version the device speaks.
    pub bcd_usb: u16,
    /// `bDeviceClass`. Zero means the class is defined per interface.
    pub class: u8,
    /// `bDeviceSubClass`.
    pub subclass: u8,
    /// `bDeviceProtocol`.
    pub protocol: u8,
    /// `bMaxPacketSize0`: the endpoint-zero packet size.
    pub max_packet0: u8,
    /// `idVendor`.
    pub vendor: u16,
    /// `idProduct`.
    pub product: u16,
    /// `bcdDevice`: the device's own revision.
    pub bcd_device: u16,
    /// `iManufacturer`: a string index, zero meaning "none".
    pub manufacturer: u8,
    /// `iProduct`.
    pub product_name: u8,
    /// `iSerialNumber`.
    pub serial: u8,
    /// `bNumConfigurations`.
    pub num_configs: u8,
}

impl DeviceDesc {
    /// Parse from the wire.
    ///
    /// Returns `None` for a short buffer, a wrong `bDescriptorType`, or a
    /// `bLength` below the structure's size. That last check matters: a device
    /// reporting a shorter descriptor than the structure has is telling us it is
    /// an older revision, and reading past what it actually wrote means trusting
    /// whatever was in the buffer.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < DEVICE_DESC_LEN || buf[1] != DT_DEVICE {
            return None;
        }
        if (buf[0] as usize) < DEVICE_DESC_LEN {
            return None;
        }
        let le16 = |o: usize| u16::from_le_bytes([buf[o], buf[o + 1]]);
        Some(Self {
            length: buf[0],
            bcd_usb: le16(2),
            class: buf[4],
            subclass: buf[5],
            protocol: buf[6],
            max_packet0: buf[7],
            vendor: le16(8),
            product: le16(10),
            bcd_device: le16(12),
            manufacturer: buf[14],
            product_name: buf[15],
            serial: buf[16],
            num_configs: buf[17],
        })
    }

    /// The USB version as major and minor.
    ///
    /// `bcdUSB` is BCD: `0x0210` is version 2.10, not 210. Printing the raw field
    /// gives "528", which is the confusion this exists to stop.
    pub const fn usb_version(&self) -> (u8, u8) {
        ((self.bcd_usb >> 8) as u8, self.bcd_usb as u8)
    }

    /// Is the class defined per interface rather than for the whole device?
    ///
    /// True for hubs and most composite devices. A driver that keys off
    /// `bDeviceClass` alone misses all of them, which is how a composite keyboard
    /// with a mass-storage interface gets bound as a disk.
    #[inline]
    pub const fn class_is_per_interface(&self) -> bool {
        self.class == 0
    }
}

/// One configuration, from `struct usb_config_descriptor`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConfigDesc {
    /// `wTotalLength`: the whole configuration, interfaces included.
    pub total_length: u16,
    /// `bNumInterfaces`.
    pub num_interfaces: u8,
    /// `bConfigurationValue`: what `SET_CONFIGURATION` takes.
    pub value: u8,
    /// `iConfiguration`.
    pub string_index: u8,
    /// `bmAttributes`: bit 7 self-powered, bit 6 remote wakeup.
    pub attributes: u8,
    /// `bMaxPower`, in 2 mA units.
    pub max_power: u8,
}

impl ConfigDesc {
    /// Parse from the wire.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < CONFIG_DESC_LEN || buf[1] != DT_CONFIG {
            return None;
        }
        Some(Self {
            total_length: u16::from_le_bytes([buf[2], buf[3]]),
            num_interfaces: buf[4],
            value: buf[5],
            string_index: buf[6],
            attributes: buf[7],
            max_power: buf[8],
        })
    }

    /// Is the configuration self-powered?
    ///
    /// Bit 7 of `bmAttributes`. A driver that ignores this and assumes the bus can
    /// supply the current asks the host for more than it can give, and the device
    /// browns out the moment it draws real load.
    #[inline]
    pub const fn is_self_powered(&self) -> bool {
        self.attributes & 0x80 != 0
    }

    /// The declared current draw in milliamps.
    ///
    /// `bMaxPower` counts 2 mA units. A configuration with zero here declares no
    /// current at all, which sinks really do, and is not the same as unlimited.
    #[inline]
    pub const fn max_power_ma(&self) -> u16 {
        self.max_power as u16 * 2
    }
}

/// One interface, from `struct usb_interface_descriptor`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InterfaceDesc {
    /// `bInterfaceNumber`.
    pub number: u8,
    /// `bAlternateSetting`.
    pub alternate: u8,
    /// `bNumEndpoints` — **offset 4**, which is where this field actually lives.
    pub num_endpoints: u8,
    /// `bInterfaceClass`, offset 5.
    pub class: u8,
    /// `bInterfaceSubClass`, offset 6.
    pub subclass: u8,
    /// `bInterfaceProtocol`, offset 7.
    pub protocol: u8,
    /// `iInterface`, offset 8 — the string index. Reading this one as
    /// `bNumEndpoints` is precisely the mistake the old parser made.
    pub string_index: u8,
}

impl InterfaceDesc {
    /// Parse from the wire.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < INTERFACE_DESC_LEN || buf[1] != DT_INTERFACE {
            return None;
        }
        Some(Self {
            number: buf[2],
            alternate: buf[3],
            num_endpoints: buf[4],
            class: buf[5],
            subclass: buf[6],
            protocol: buf[7],
            string_index: buf[8],
        })
    }
}

/// One endpoint, from `struct usb_endpoint_descriptor`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EndpointDesc {
    /// `bEndpointAddress`: direction in bit 7, number in bits 3:0.
    pub address: u8,
    /// `bmAttributes`: transfer type in bits 1:0.
    pub attributes: u8,
    /// `wMaxPacketSize`, including the high-bandwidth multiplier bits.
    pub max_packet_raw: u16,
    /// `bInterval`, in milliseconds for interrupt endpoints.
    pub interval: u8,
}

impl EndpointDesc {
    /// Parse from the wire.
    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < ENDPOINT_DESC_LEN || buf[1] != DT_ENDPOINT {
            return None;
        }
        Some(Self {
            address: buf[2],
            attributes: buf[3],
            max_packet_raw: u16::from_le_bytes([buf[4], buf[5]]),
            interval: buf[6],
        })
    }

    /// The endpoint number, `address` bits 3:0.
    ///
    /// Not `address & 0xFF`: that is the whole byte, and using it as a number
    /// makes endpoint 0x81 — interrupt IN 1 — claim to be number 129. The loop
    /// that matches an interface's endpoints to its own numbering then never
    /// finds it.
    #[inline]
    pub const fn number(&self) -> u8 {
        self.address & 0x0F
    }

    /// The direction, as a [`DIR_IN`] or [`DIR_OUT`] mask.
    #[inline]
    pub const fn direction(&self) -> u8 {
        self.address & DIR_IN
    }

    /// Does the device send data to the host on this endpoint?
    #[inline]
    pub const fn is_in(&self) -> bool {
        self.address & DIR_IN != 0
    }

    /// The transfer type, as an `XFER_*` constant.
    #[inline]
    pub const fn xfer_type(&self) -> u8 {
        self.attributes & XFER_TYPE_MASK
    }

    /// Is this a control endpoint?
    #[inline]
    pub const fn is_control(&self) -> bool {
        self.xfer_type() == XFER_CONTROL
    }

    /// Is this a bulk endpoint?
    #[inline]
    pub const fn is_bulk(&self) -> bool {
        self.xfer_type() == XFER_BULK
    }

    /// Is this an interrupt endpoint?
    #[inline]
    pub const fn is_interrupt(&self) -> bool {
        self.xfer_type() == XFER_INT
    }

    /// Is this an isochronous endpoint?
    #[inline]
    pub const fn is_isoc(&self) -> bool {
        self.xfer_type() == XFER_ISOC
    }

    /// The packet size, with the high-bandwidth multiplier removed.
    ///
    /// For a high-speed bulk endpoint the top four bits of `wMaxPacketSize` hold
    /// the transaction count and bits 10:0 the per-transaction size. Masking off
    /// only the multiplier — rather than a fixed 11 bits — is what keeps a
    /// 1024-byte packet (0x0400) from coming back as zero.
    pub const fn max_packet(&self) -> u16 {
        self.max_packet_raw & !((MAXPACKET_MULT_MASK as u16) << MAXPACKET_ADDITIONAL_SHIFT)
    }

    /// Transactions per microframe: 1, 2 or 3.
    ///
    /// The field is one-based, so zero means "not set", which is the same as a
    /// single transaction.
    pub const fn transactions_per_microframe(&self) -> u16 {
        let n = (self.max_packet_raw >> MAXPACKET_ADDITIONAL_SHIFT) & MAXPACKET_MULT_MASK as u16;
        if n == 0 { 1 } else { n }
    }

    /// Bytes this endpoint moves in one microframe.
    pub const fn bytes_per_microframe(&self) -> u32 {
        self.max_packet() as u32 * self.transactions_per_microframe() as u32
    }
}

/// What one walk of a configuration descriptor found.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConfigScan {
    /// The configuration header itself.
    pub config: ConfigDesc,
    /// How many interfaces were seen.
    pub interface_count: u8,
    /// How many endpoints were seen.
    pub endpoint_count: u8,
    /// How many bytes of the configuration were consumed.
    pub bytes_used: u16,
}

/// Walk a configuration descriptor, counting interfaces and endpoints.
///
/// This is the safe replacement for the old `parse_config`, which advanced by the
/// first byte's value without checking that the descriptor actually fitted. A
/// truncated or malformed descriptor makes that loop run off the end of the
/// buffer, or step backwards if `bLength` is zero.
///
/// The walk stops at whichever comes first: `wTotalLength` bytes consumed, the
/// end of the buffer, or a descriptor whose `bLength` is below the two bytes
/// every descriptor must have. That last check is the important one — a zero
/// `bLength` with a `bDescriptorType` of zero would otherwise be an infinite
/// loop, since the walk would never advance.
pub fn scan_config(buf: &[u8]) -> Option<ConfigScan> {
    let config = ConfigDesc::parse(buf)?;
    let limit = (config.total_length as usize).min(buf.len());
    let mut scan = ConfigScan { config, ..Default::default() };

    let mut off = CONFIG_DESC_LEN;
    while off + 2 <= limit {
        let len = buf[off] as usize;
        let typ = buf[off + 1];

        // A descriptor must at least name its own length and type. Anything less
        // cannot be advanced past, so stopping is the only safe move.
        if len < 2 {
            break;
        }
        if off + len > limit {
            break;
        }

        match typ {
            DT_INTERFACE => {
                if let Some(i) = InterfaceDesc::parse(&buf[off..off + len]) {
                    let _ = i;
                    scan.interface_count += 1;
                }
            }
            DT_ENDPOINT => {
                if EndpointDesc::parse(&buf[off..off + len]).is_some() {
                    scan.endpoint_count += 1;
                }
            }
            _ => {}
        }
        off += len;
    }

    scan.bytes_used = off as u16;
    Some(scan)
}

/// Parse a device descriptor.
///
/// A thin alias for [`DeviceDesc::parse`], kept because the enumeration path
/// asks "did the device descriptor arrive" in exactly these terms.
pub fn parse_device(buf: &[u8]) -> Option<DeviceDesc> {
    DeviceDesc::parse(buf)
}

/// Parse a configuration descriptor into caller-owned arrays.
///
/// The enumeration path hands over fixed-size arrays rather than allocating, so
/// this returns what the old signature returned: the configuration value to
/// `SET_CONFIGURATION`, the total length, and how many interfaces and endpoints
/// were seen.
///
/// The output arrays are truncated to their own length. A device with more
/// endpoints than the driver has room for loses the tail rather than overrunning
/// — a real case, since a webcam declares video, audio and streaming endpoints
/// and the enumeration table here is eight deep. The *count* still reports what
/// the device declared, so a caller can tell "truncated" from "complete".
pub fn parse_config(
    buf: &[u8],
    out_ifaces: &mut [InterfaceDesc],
    out_eps: &mut [EndpointDesc],
) -> Option<(u8, u16, usize, usize)> {
    let scan = scan_config(buf)?;

    let mut ni = 0usize;
    let mut ne = 0usize;
    let mut off = CONFIG_DESC_LEN;
    let limit = (scan.config.total_length as usize).min(buf.len());

    while off + 2 <= limit {
        let len = buf[off] as usize;
        let typ = buf[off + 1];
        if len < 2 || off + len > limit {
            break;
        }
        match typ {
            DT_INTERFACE => {
                if let Some(i) = InterfaceDesc::parse(&buf[off..off + len]) {
                    if ni < out_ifaces.len() {
                        out_ifaces[ni] = i;
                    }
                    ni += 1;
                }
            }
            DT_ENDPOINT => {
                if let Some(e) = EndpointDesc::parse(&buf[off..off + len]) {
                    if ne < out_eps.len() {
                        out_eps[ne] = e;
                    }
                    ne += 1;
                }
            }
            _ => {}
        }
        off += len;
    }

    Some((scan.config.value, scan.config.total_length, ni, ne))
}

/// One interface with its endpoints, gathered from a configuration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InterfaceWithEndpoints {
    /// The interface header.
    pub interface: InterfaceDesc,
    /// The endpoints that follow it, up to [`MAX_ENDPOINTS`].
    pub endpoints: [EndpointDesc; MAX_ENDPOINTS],
    /// How many entries of `endpoints` are valid.
    pub endpoint_count: u8,
}

/// The most endpoints this driver will track for one interface.
///
/// USB 2.0 allows 15 per interface; the extra headroom is for the audio
/// endpoints a webcam declares alongside its video one. Fixed so the type needs
/// no allocator.
pub const MAX_ENDPOINTS: usize = 16;
/// The most interfaces this driver will track for one configuration.
pub const MAX_INTERFACES: usize = 16;

/// Find one interface and the endpoints belonging to it.
///
/// The association is positional, not incidental: an interface's endpoints are
/// the endpoint descriptors that follow it, up to `bNumEndpoints` of them, and
/// then the next interface descriptor begins. Getting this wrong — by claiming
/// every endpoint in the configuration for every interface — is how a composite
/// device ends up with four copies of each endpoint and a driver that hangs.
pub fn find_interface(buf: &[u8], target: u8) -> Option<InterfaceWithEndpoints> {
    let config = ConfigDesc::parse(buf)?;
    let limit = (config.total_length as usize).min(buf.len());
    let mut off = CONFIG_DESC_LEN;

    while off + 2 <= limit {
        let len = buf[off] as usize;
        let typ = buf[off + 1];
        if len < 2 || off + len > limit {
            break;
        }

        if typ == DT_INTERFACE {
            if let Some(iface) = InterfaceDesc::parse(&buf[off..off + len]) {
                if iface.number == target {
                    return collect_endpoints(buf, off + len, limit, iface);
                }
            }
        }
        off += len;
    }
    None
}

/// Collect the endpoints that follow an interface header.
///
/// `declared` is `bNumEndpoints` — a count, and a claim. A device that declares
/// three endpoints and then supplies one is not a reason to walk into the next
/// interface's descriptors looking for the other two, so the loop stops at the
/// next interface header and reports what it found.
fn collect_endpoints(
    buf: &[u8],
    start: usize,
    limit: usize,
    interface: InterfaceDesc,
) -> Option<InterfaceWithEndpoints> {
    let mut out = InterfaceWithEndpoints {
        interface,
        ..Default::default()
    };

    let mut off = start;
    let mut want = interface.num_endpoints as usize;
    while want > 0 && off + 2 <= limit {
        let len = buf[off] as usize;
        let typ = buf[off + 1];
        if len < 2 || off + len > limit {
            break;
        }
        if typ == DT_ENDPOINT {
            if let Some(ep) = EndpointDesc::parse(&buf[off..off + len]) {
                if (out.endpoint_count as usize) < MAX_ENDPOINTS {
                    out.endpoints[out.endpoint_count as usize] = ep;
                    out.endpoint_count += 1;
                }
                want -= 1;
            }
        } else if typ == DT_INTERFACE {
            break;
        }
        off += len;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real 18-byte device descriptor: a full-speed HID keyboard.
    ///
    /// Little-endian throughout, because that is the wire order. `0x045E` is
    /// Microsoft, class 0 so the class lives on the interface.
    fn keyboard_device_desc() -> [u8; DEVICE_DESC_LEN] {
        let mut d = [0u8; DEVICE_DESC_LEN];
        d[0] = DEVICE_DESC_LEN as u8;
        d[1] = DT_DEVICE;
        d[2..4].copy_from_slice(&0x0110u16.to_le_bytes()); // bcdUSB 1.10
        d[7] = 8; // bMaxPacketSize0
        d[8..10].copy_from_slice(&0x045Eu16.to_le_bytes());
        d[10..12].copy_from_slice(&0xC31Cu16.to_le_bytes());
        d[12..14].copy_from_slice(&0x0100u16.to_le_bytes());
        d[14] = 1; // iManufacturer
        d[15] = 2; // iProduct
        d[16] = 3; // iSerialNumber
        d[17] = 1; // bNumConfigurations
        d
    }

    /// A 9-byte interface descriptor with **one** endpoint.
    ///
    /// The layout is the point: `bNumEndpoints` is 1 at offset 4, `iInterface`
    /// is 0 at offset 8. A parser reading offset 8 as the endpoint count reports
    /// zero, and the driver then hunts for an interrupt endpoint it was told does
    /// not exist.
    fn hid_interface_desc() -> [u8; INTERFACE_DESC_LEN] {
        let mut i = [0u8; INTERFACE_DESC_LEN];
        i[0] = INTERFACE_DESC_LEN as u8;
        i[1] = DT_INTERFACE;
        i[4] = 1; // bNumEndpoints ← offset 4
        i[5] = 0x03; // bInterfaceClass: HID
        i[6] = 0x01; // boot interface subclass
        i[7] = 0x01; // keyboard
        i[8] = 0; // iInterface ← offset 8, a string index of "none"
        i
    }

    /// A 7-byte interrupt IN endpoint 1, the standard keyboard endpoint.
    fn keyboard_endpoint_desc() -> [u8; ENDPOINT_DESC_LEN] {
        let mut e = [0u8; ENDPOINT_DESC_LEN];
        e[0] = ENDPOINT_DESC_LEN as u8;
        e[1] = DT_ENDPOINT;
        e[2] = 0x81; // IN, endpoint 1
        e[3] = 0x03; // interrupt
        e[4..6].copy_from_slice(&8u16.to_le_bytes());
        e[6] = 10; // 10 ms
        e
    }

    fn config_with(extra: &[u8], total: u16) -> [u8; CONFIG_DESC_LEN + 32] {
        let mut c = [0u8; CONFIG_DESC_LEN + 32];
        c[0] = CONFIG_DESC_LEN as u8;
        c[1] = DT_CONFIG;
        c[2..4].copy_from_slice(&total.to_le_bytes());
        c[4] = 1; // bNumInterfaces
        c[5] = 1; // bConfigurationValue
        c[7] = 0x80; // self-powered
        c[8] = 50; // 100 mA
        let n = extra.len().min(c.len() - CONFIG_DESC_LEN);
        c[CONFIG_DESC_LEN..CONFIG_DESC_LEN + n].copy_from_slice(&extra[..n]);
        c
    }

    // ── the descriptor that was wrong ───────────────────────────────────────

    #[test]
    fn num_endpoints_comes_from_offset_four_not_eight() {
        let d = hid_interface_desc();
        let i = InterfaceDesc::parse(&d).expect("a valid interface descriptor");
        assert_eq!(i.num_endpoints, 1, "bNumEndpoints lives at offset 4");
        assert_eq!(i.string_index, 0, "offset 8 is iInterface, a string index");
        // The old parser read offset 8 and reported zero endpoints, so the driver
        // never found the keyboard's interrupt endpoint.
        assert_ne!(i.num_endpoints, d[8] as u8);
    }

    #[test]
    fn a_keyboard_interface_reports_its_one_endpoint() {
        let i = InterfaceDesc::parse(&hid_interface_desc()).expect("a valid descriptor");
        assert_eq!(i.class, 0x03, "HID class");
        assert_eq!(i.subclass, 0x01, "boot subclass");
        assert_eq!(i.protocol, 0x01, "keyboard");
    }

    // ── device descriptor ───────────────────────────────────────────────────

    #[test]
    fn a_device_descriptor_decodes_its_identity() {
        let d = DeviceDesc::parse(&keyboard_device_desc()).expect("a valid descriptor");
        assert_eq!(d.vendor, 0x045E);
        assert_eq!(d.product, 0xC31C);
        assert_eq!(d.max_packet0, 8);
        assert_eq!(d.num_configs, 1);
        assert_eq!(d.manufacturer, 1);
    }

    #[test]
    fn bcd_usb_is_decoded_as_bcd_not_as_an_integer() {
        // bcdUSB 0x0110 is version 1.10. The low byte is BCD, so its *value* is
        // 0x10 = 16, not 10 — reading it as a plain integer gives "USB 116", and
        // reading 0x0110 as an integer gives 272. Neither is a version.
        let d = DeviceDesc::parse(&keyboard_device_desc()).expect("a valid descriptor");
        assert_eq!(d.bcd_usb, 0x0110);
        assert_eq!(d.usb_version().0, 1, "major");
        assert_eq!(d.usb_version().1, 0x10, "minor, as a BCD value");
    }

    #[test]
    fn a_class_of_zero_means_per_interface() {
        let d = DeviceDesc::parse(&keyboard_device_desc()).expect("a valid descriptor");
        assert!(d.class_is_per_interface());
    }

    #[test]
    fn a_short_device_descriptor_is_refused() {
        assert_eq!(DeviceDesc::parse(&[0u8; 17]), None);
    }

    #[test]
    fn a_device_descriptor_with_the_wrong_type_is_refused() {
        let mut d = keyboard_device_desc();
        d[1] = DT_CONFIG;
        assert_eq!(DeviceDesc::parse(&d), None);
    }

    #[test]
    fn a_device_descriptor_claiming_too_small_a_length_is_refused() {
        // A device reporting bLength 8 has an older, smaller revision; reading
        // the full 18 bytes would trust whatever followed it in the buffer.
        let mut d = keyboard_device_desc();
        d[0] = 8;
        assert_eq!(DeviceDesc::parse(&d), None);
    }
    // ── endpoint descriptor ─────────────────────────────────────────────────

    #[test]
    fn an_endpoint_number_ignores_the_direction_bit() {
        // 0x81 is interrupt IN endpoint 1. Masking the whole byte gives 129, and
        // the loop matching endpoints to an interface's numbering then never finds
        // it — a keyboard that enumerates and never types.
        let e = EndpointDesc::parse(&keyboard_endpoint_desc()).expect("a valid endpoint");
        assert_eq!(e.number(), 1);
        assert_eq!(e.address, 0x81, "the raw address really is 0x81");
    }

    #[test]
    fn endpoint_direction_and_type_decode() {
        let e = EndpointDesc::parse(&keyboard_endpoint_desc()).expect("a valid endpoint");
        assert!(e.is_in());
        assert_eq!(e.direction(), DIR_IN);
        assert!(e.is_interrupt());
        assert!(!e.is_bulk());
        assert!(!e.is_control());
        assert!(!e.is_isoc());
        assert_eq!(e.max_packet(), 8);
        assert_eq!(e.interval, 10);
    }

    #[test]
    fn an_out_endpoint_is_not_in() {
        let mut d = keyboard_endpoint_desc();
        d[2] = 0x02; // OUT, endpoint 2
        let e = EndpointDesc::parse(&d).expect("a valid endpoint");
        assert!(!e.is_in());
        assert_eq!(e.number(), 2);
    }

    #[test]
    fn high_bandwidth_bulk_endpoints_report_their_full_size() {
        // Bits 14:11 of wMaxPacketSize hold the transaction count, bits 10:0 the
        // size. 512 bytes with two transactions is 0x0800 | (2 << 11). Getting
        // the shift wrong by one puts the multiplier inside the packet size and
        // reports 1024 for an endpoint that moves 512.
        let mut d = [0u8; ENDPOINT_DESC_LEN];
        d[0] = ENDPOINT_DESC_LEN as u8;
        d[1] = DT_ENDPOINT;
        d[2] = 0x81;
        d[3] = XFER_BULK;
        d[4..6].copy_from_slice(&(0x0200u16 | (2 << 11)).to_le_bytes());
        let e = EndpointDesc::parse(&d).expect("a valid endpoint");
        assert_eq!(e.max_packet(), 512, "the per-transaction size");
        assert_eq!(e.transactions_per_microframe(), 2);
        assert_eq!(e.bytes_per_microframe(), 1024, "which is what actually moves");
    }

    #[test]
    fn a_1024_byte_packet_survives_the_mask() {
        // The case a fixed 11-bit mask breaks: bit 10 is part of the size.
        let mut d = [0u8; ENDPOINT_DESC_LEN];
        d[0] = ENDPOINT_DESC_LEN as u8;
        d[1] = DT_ENDPOINT;
        d[2] = 0x81;
        d[3] = XFER_BULK;
        d[4..6].copy_from_slice(&0x0400u16.to_le_bytes());
        let e = EndpointDesc::parse(&d).expect("a valid endpoint");
        assert_eq!(e.max_packet(), 1024);
        assert_eq!(e.transactions_per_microframe(), 1);
    }

    #[test]
    fn a_single_transaction_endpoint_reports_one() {
        let e = EndpointDesc::parse(&keyboard_endpoint_desc()).expect("a valid endpoint");
        assert_eq!(e.transactions_per_microframe(), 1);
        assert_eq!(e.bytes_per_microframe(), 8);
    }

    #[test]
    fn a_bus_powered_config_is_not_self_powered() {
        let mut c = keyboard_config();
        c[7] = 0x00;
        assert!(!ConfigDesc::parse(&c).expect("a valid config").is_self_powered());
    }

    /// A configuration with one HID interface and one interrupt endpoint.
    fn keyboard_config() -> [u8; CONFIG_DESC_LEN + INTERFACE_DESC_LEN + ENDPOINT_DESC_LEN] {
        let total = (CONFIG_DESC_LEN + INTERFACE_DESC_LEN + ENDPOINT_DESC_LEN) as u16;
        let mut c = [0u8; CONFIG_DESC_LEN + INTERFACE_DESC_LEN + ENDPOINT_DESC_LEN];
        c[0] = CONFIG_DESC_LEN as u8;
        c[1] = DT_CONFIG;
        c[2..4].copy_from_slice(&total.to_le_bytes());
        c[4] = 1;
        c[5] = 1;
        c[7] = 0x80;
        c[8] = 50;
        c[CONFIG_DESC_LEN..CONFIG_DESC_LEN + INTERFACE_DESC_LEN]
            .copy_from_slice(&hid_interface_desc());
        c[CONFIG_DESC_LEN + INTERFACE_DESC_LEN..].copy_from_slice(&keyboard_endpoint_desc());
        c
    }

    #[test]
    fn scanning_a_configuration_counts_its_interfaces_and_endpoints() {
        let c = keyboard_config();
        let s = scan_config(&c).expect("a valid configuration");
        assert_eq!(s.interface_count, 1);
        assert_eq!(s.endpoint_count, 1);
        assert_eq!(s.bytes_used, c.len() as u16, "the whole config was walked");
    }

    #[test]
    fn the_walk_finds_the_interfaces_endpoints() {
        // The end-to-end consequence of the offset-4 fix: the driver now reaches
        // the endpoint it was previously told did not exist.
        let c = keyboard_config();
        let i = find_interface(&c, 0).expect("interface 0");
        assert_eq!(i.interface.num_endpoints, 1);
        assert_eq!(i.endpoint_count, 1);
        assert!(i.endpoints[0].is_interrupt());
        assert!(i.endpoints[0].is_in());
    }

    #[test]
    fn an_absent_interface_is_reported_as_absent() {
        let c = keyboard_config();
        assert!(find_interface(&c, 7).is_none());
    }

    #[test]
    fn a_zero_length_descriptor_stops_the_walk_instead_of_looping() {
        // bLength 0 with bDescriptorType 0 is the classic malformed descriptor.
        // A walk that advances by bLength without checking runs forever.
        let mut c = keyboard_config();
        c[CONFIG_DESC_LEN] = 0;
        c[CONFIG_DESC_LEN + 1] = 0;
        let s = scan_config(&c).expect("the config header still parses");
        assert_eq!(s.interface_count, 0, "the walk stopped rather than spinning");
    }

    #[test]
    fn a_descriptor_running_past_the_buffer_stops_the_walk() {
        let mut c = keyboard_config();
        c[CONFIG_DESC_LEN] = 0xF0; // claims far more than is left
        let s = scan_config(&c).expect("the config header still parses");
        assert_eq!(s.interface_count, 0);
    }

    #[test]
    fn a_walk_stops_at_wtotal_length() {
        // wTotalLength smaller than the buffer: the extra bytes are not ours.
        let mut truncated = keyboard_config();
        let header_and_iface = (CONFIG_DESC_LEN + INTERFACE_DESC_LEN) as u16;
        truncated[2..4].copy_from_slice(&header_and_iface.to_le_bytes());
        let s = scan_config(&truncated).expect("a valid header");
        assert_eq!(s.interface_count, 1);
        assert_eq!(s.endpoint_count, 0, "the endpoint is past wTotalLength");
    }

    #[test]
    fn endpoints_belong_to_their_own_interface() {
        // A composite device: interface 0 is HID with one endpoint, interface 1 is
        // mass storage with two. Claiming every endpoint for every interface gives
        // the HID driver three endpoints and the storage driver three, and the
        // storage driver then queues on an endpoint that is not its own.
        let mut body = [0u8; 40];
        // Interfaces are back to back; the HID one declares one endpoint, the
        // mass-storage one declares two.
        body[0] = INTERFACE_DESC_LEN as u8;
        body[1] = DT_INTERFACE;
        body[4] = 1;
        body[5] = 0x03; // HID
        let o = INTERFACE_DESC_LEN; // the HID interface's own endpoint
        body[o] = ENDPOINT_DESC_LEN as u8;
        body[o + 1] = DT_ENDPOINT;
        body[o + 2] = 0x81;
        body[o + 3] = XFER_INT;
        // interface 1 follows, and its two endpoints come after that
        let o1 = o + ENDPOINT_DESC_LEN;
        body[o1] = INTERFACE_DESC_LEN as u8;
        body[o1 + 1] = DT_INTERFACE;
        body[o1 + 2] = 1; // interface number 1
        body[o1 + 4] = 2;
        body[o1 + 5] = 0x08; // mass storage
        let o2 = o1 + INTERFACE_DESC_LEN;
        body[o2] = ENDPOINT_DESC_LEN as u8;
        body[o2 + 1] = DT_ENDPOINT;
        body[o2 + 2] = 0x81;
        body[o2 + 3] = XFER_BULK;
        let o3 = o2 + ENDPOINT_DESC_LEN;
        body[o3] = ENDPOINT_DESC_LEN as u8;
        body[o3 + 1] = DT_ENDPOINT;
        body[o3 + 2] = 0x02;
        body[o3 + 3] = XFER_BULK;

        // Interfaces are back to back: the HID one (9 bytes) plus its endpoint
        // (7), then the mass-storage one (9) plus its two (14).
        let total = (CONFIG_DESC_LEN + 2 * INTERFACE_DESC_LEN + 3 * ENDPOINT_DESC_LEN) as u16;
        let mut c = [0u8; CONFIG_DESC_LEN + 40];
        c[0] = CONFIG_DESC_LEN as u8;
        c[1] = DT_CONFIG;
        c[2..4].copy_from_slice(&total.to_le_bytes());
        c[4] = 2;
        c[5] = 1;
        c[CONFIG_DESC_LEN..CONFIG_DESC_LEN + body.len()].copy_from_slice(&body);

        let hid = find_interface(&c, 0).expect("interface 0");
        let storage = find_interface(&c, 1).expect("interface 1");
        assert_eq!(hid.endpoint_count, 1, "HID has exactly one endpoint");
        assert!(hid.endpoints[0].is_interrupt());
        assert_eq!(storage.endpoint_count, 2, "storage has its two");
        assert!(storage.endpoints[0].is_bulk());
        assert!(storage.endpoints[1].is_bulk());
    }

    #[test]
    fn a_device_that_overstates_its_endpoints_does_not_borrow_the_next_ones() {
        // Interface 0 claims two endpoints but supplies one, and interface 1
        // follows immediately. Walking past the gap to find the second would hand
        // interface 0 an endpoint belonging to a different function.
        let mut body = [0u8; 24];
        body[0] = INTERFACE_DESC_LEN as u8;
        body[1] = DT_INTERFACE;
        body[4] = 2; // claims two
        body[5] = 0x03;
        let o = INTERFACE_DESC_LEN;
        body[o] = ENDPOINT_DESC_LEN as u8;
        body[o + 1] = DT_ENDPOINT;
        body[o + 2] = 0x81;
        body[o + 3] = XFER_INT;
        let o2 = o + ENDPOINT_DESC_LEN;
        body[o2] = INTERFACE_DESC_LEN as u8;
        body[o2 + 1] = DT_INTERFACE;
        body[o2 + 2] = 1;
        body[o2 + 4] = 0;
        body[o2 + 5] = 0x08;

        let total = (CONFIG_DESC_LEN + 2 * INTERFACE_DESC_LEN + ENDPOINT_DESC_LEN) as u16;
        let mut c = [0u8; CONFIG_DESC_LEN + 24];
        c[0] = CONFIG_DESC_LEN as u8;
        c[1] = DT_CONFIG;
        c[2..4].copy_from_slice(&total.to_le_bytes());
        c[4] = 2;
        c[5] = 1;
        c[CONFIG_DESC_LEN..CONFIG_DESC_LEN + body.len()].copy_from_slice(&body);

        let hid = find_interface(&c, 0).expect("interface 0");
        assert_eq!(hid.endpoint_count, 1, "only the one that was actually there");
    }
}
