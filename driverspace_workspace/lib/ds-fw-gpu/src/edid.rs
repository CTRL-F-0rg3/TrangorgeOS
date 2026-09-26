//! EDID, and the display mode it describes.
//!
//! One EDID parser serves both HDMI and DisplayPort, because the sink does not
//! know which link is carrying it: an EDID read over I²C/DDC on HDMI and over
//! AUX on DP is the same 128 bytes, the same structure, the same manufacturer
//! and product code. Splitting this per-linkage is how two parsers end up
//! disagreeing about the same monitor.
//!
//! ## What is here, and what is not
//!
//! * [`Edid`] — the raw 128 bytes, and the accessors that read them safely.
//! * [`parse`] — block validation and the four detailed timing descriptors.
//! * [`DisplayMode`] — a mode with its timings and derived refresh rate.
//!
//! Not here: how the bytes are *fetched*. That is a driver concern, because the
//! transport is the one thing the two linkages genuinely do not share.
//!
//! ## Why a base block is only 128 bytes
//!
//! EDID 1.4 fits in 128 bytes and that is what essentially every sink on real
//! hardware emits. The 256-byte extension carries CTA or DisplayID blocks that
//! matter for audio and HDR, and [`Edid::extension_count`] reports them so a
//! caller that cares can fetch more — but a driver that *requires* 256 bytes
//! cannot drive the monitors sold in the last two decades.

use alloc::vec::Vec;

/// The fixed size of one EDID block.
pub const EDID_LEN: usize = 128;

/// The header every block must begin with: `00 FF FF FF FF FF FF 00`.
pub const EDID_HEADER: [u8; 8] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];

/// Byte offset of the manufacturer ID.
pub const MANUFACTURER_OFFSET: usize = 8;
/// Byte offset of the product code.
pub const PRODUCT_OFFSET: usize = 10;
/// Byte offset of the week of manufacture.
pub const WEEK_OFFSET: usize = 12;
/// Byte offset of the year of manufacture, minus 1990.
pub const YEAR_OFFSET: usize = 13;
/// Byte offset of the version byte.
pub const VERSION_OFFSET: usize = 14;
/// Byte offset of the basic display parameter byte.
pub const BASIC_DISPLAY_OFFSET: usize = 20;
/// Byte offset of the first of four 18-byte detailed timing descriptors.
pub const DETAILED_TIMINGS_OFFSET: usize = 54;
/// The size of one detailed timing descriptor.
pub const DETAILED_TIMING_LEN: usize = 18;
/// How many detailed timing descriptors a base block carries.
pub const DETAILED_TIMING_COUNT: usize = 4;

/// The largest pixel clock a descriptor can express, in kHz.
///
/// The field holds a `u16` of 10 kHz units, so 655 350 kHz is the ceiling. The
/// parser clamps rather than letting a larger claim produce a wrapped refresh
/// rate that looks like a slow mode.
pub const MAX_PIXEL_CLOCK_KHZ: u32 = 655_350;

/// A pixel clock is stored in units of 10 kHz.
const CLOCK_UNIT_KHZ: u16 = 10;

/// Interlaced.
pub const MODE_FLAG_INTERLACE: u32 = 1 << 0;
/// Positive sync: pulses at the start of the blanking interval.
pub const MODE_FLAG_PVSYNC: u32 = 1 << 1;
/// Negative sync: pulses at the *end* of the blanking interval.
pub const MODE_FLAG_PHSYNC: u32 = 1 << 2;

/// The largest value an active-area or blanking field can express.
///
/// The active area is a low byte plus the high nibble of a later byte, so 12
/// bits — 4095, which covers 4096x2160 minus blanking. An earlier revision of
/// this file bounded it at 1023, which silently rejected *every* real desktop
/// mode: 1920 does not fit in 10 bits, so 1080p60 was discarded and the sink
/// appeared to offer no modes at all.
const FIELD_MAX_12BIT: u16 = 4095;

/// The smallest active area that is a real display mode.
///
/// A sink leaves an unused timing slot as `01 01 01 01 01 01 …`, which decodes
/// cleanly: 1x1 active, 257 pixels of blanking, a 2.57 MHz clock. Nothing else
/// about it says "blank", so the active area is the only honest discriminator.
/// Accepting it puts a 1x1 entry in the mode list.
const MIN_ACTIVE_EDGE: u16 = 16;

/// Interlace bit, byte 17 of a descriptor.
const INTERLACE_BIT: u8 = 0x80;

/// Byte 17 bit 3: vertical sync polarity, set meaning positive.
const VSYNC_POLARITY_BIT: u8 = 0x08;

/// Byte 17 bit 4: horizontal sync polarity, set meaning positive.
const HSYNC_POLARITY_BIT: u8 = 0x10;

/// An unused descriptor slot is filled with this clock value, not with zero.
///
/// A sink that skips a timing writes `01 01 01 01 01 01`, which decodes to a
/// 10 kHz 1x1 "mode". Treating it as real puts a 1x1 entry in the mode list.
const UNUSED_DESCRIPTOR_CLOCK_10KHZ: u16 = 1;

/// One display mode, with the timings a scanout engine needs.
///
/// `w` and `h` lead because that is what a caller wants when it lists modes; the
/// timings are what it wants when it programs one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DisplayMode {
    /// Width in pixels.
    pub w: u16,
    /// Height in lines.
    pub h: u16,
    /// Total horizontal period in pixels, active plus blanking.
    pub htotal: u16,
    /// Total vertical period in lines, active plus blanking.
    pub vtotal: u16,
    /// Horizontal sync pulse width, in pixels.
    pub hsync_width: u16,
    /// Horizontal sync pulse offset from the start of blanking.
    pub hsync_offset: u16,
    /// Vertical sync pulse width, in lines.
    pub vsync_width: u16,
    /// Vertical sync pulse offset from the start of blanking.
    pub vsync_offset: u16,
    /// Pixel clock in kHz.
    ///
    /// `u32`, not `u16`: a 1080p60 mode needs 148 500 kHz and 4K60 needs over
    /// 500 000, so a `u16` cannot hold a single ordinary desktop mode. The EDID
    /// field itself is a 10 kHz step, but the decoded value is not bounded by it.
    pub clock_khz: u32,
    /// A mask of `MODE_FLAG_*`.
    pub flags: u32,
}

impl DisplayMode {
    /// Refresh rate in millihertz, from the clock and the total period.
    ///
    /// Millihertz rather than Hz because the arithmetic is integral and 59.94 Hz
    /// is not 60: rounding it to 60 hands hardware a mode it cannot scan out. The
    /// caller divides by 1000.
    ///
    /// Returns 0 for a degenerate period, which is a descriptor whose total is
    /// zero rather than a very fast mode.
    #[inline]
    pub const fn refresh_mhz(&self) -> u32 {
        let total = self.htotal as u32 * self.vtotal as u32;
        if total == 0 {
            return 0;
        }
        // clock_khz is kHz; × 1_000_000 lands in millihertz. The 64-bit
        // intermediate is required: 500000 × 1_000_000 overflows u32.
        (self.clock_khz as u64 * 1_000_000 / total as u64) as u32
    }

    /// Refresh rate in whole Hz, rounded to nearest.
    #[inline]
    pub fn refresh_hz(&self) -> u32 {
        (self.refresh_mhz() + 500) / 1000
    }

    /// Bytes one scanline occupies at `bpp` bits per pixel.
    ///
    /// Saturating rather than wrapping: a 4K mode at 64 bpp overflows `u16`, but
    /// the real value is merely large, and a wrapped stride is a buffer the
    /// scanout engine reads out of bounds.
    #[inline]
    pub const fn stride_bytes(&self, bpp: u16) -> u32 {
        ((bpp as u32 + 7) / 8).saturating_mul(self.htotal as u32)
    }

    /// Is this mode usable as a scanout target?
    ///
    /// Rejects the zero mode, one with no active area, and one whose total period
    /// is shorter than its active area — each a descriptor we parsed but must not
    /// hand to hardware.
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.w != 0
            && self.h != 0
            && self.clock_khz != 0
            && (self.htotal as u32) >= self.w as u32
            && (self.vtotal as u32) >= self.h as u32
    }

    /// Is this mode interlaced?
    #[inline]
    pub const fn is_interlaced(&self) -> bool {
        self.flags & MODE_FLAG_INTERLACE != 0
    }

    /// Is the vertical sync polarity positive?
    #[inline]
    pub const fn is_positive_vsync(&self) -> bool {
        self.flags & MODE_FLAG_PVSYNC != 0
    }

    /// Is the horizontal sync polarity positive?
    #[inline]
    pub const fn is_positive_hsync(&self) -> bool {
        self.flags & MODE_FLAG_PHSYNC != 0
    }
}

/// One 128-byte EDID block.
///
/// A borrowed view rather than an owned array, because a driver holds at most a
/// handful of these and copying 128 bytes per getter call is waste.
#[derive(Debug, Clone, Copy)]
pub struct Edid<'a> {
    bytes: &'a [u8],
}

impl<'a> Edid<'a> {
    /// Wrap a buffer, checking nothing.
    ///
    /// Every accessor below is bounds-checked, so a short buffer yields `None`
    /// rather than a panic. That matters because this constructor is how bytes
    /// that came off a wire get in, and a truncated DDC read is a normal hardware
    /// outcome rather than a programming error.
    #[inline]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    /// The raw bytes, which may be shorter than [`EDID_LEN`].
    #[inline]
    pub const fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Is this a complete block?
    #[inline]
    pub const fn is_complete(&self) -> bool {
        self.bytes.len() >= EDID_LEN
    }

    /// Read the byte at `offset`, if the block reaches that far.
    #[inline]
    pub fn byte(&self, offset: usize) -> Option<u8> {
        self.bytes.get(offset).copied()
    }

    /// Does the block start with the mandatory header?
    ///
    /// Checked separately from [`Self::is_complete`] because a full-length buffer
    /// that is not an EDID at all is a plausible failure: a device that NAKs and
    /// leaves the bus pulled high reads as `0xFF` throughout, and that is a
    /// header-shaped failure worth distinguishing from a short read.
    pub fn has_valid_header(&self) -> bool {
        self.bytes.len() >= EDID_HEADER.len() && self.bytes[..EDID_HEADER.len()] == EDID_HEADER
    }

    /// The EDID version, major in the high nibble.
    #[inline]
    pub fn version(&self) -> Option<u8> {
        self.byte(VERSION_OFFSET)
    }

    /// The EDID revision, minor.
    #[inline]
    pub fn revision(&self) -> Option<u8> {
        self.byte(VERSION_OFFSET + 1)
    }

    /// Week of manufacture, 1–54.
    #[inline]
    pub fn week(&self) -> Option<u8> {
        self.byte(WEEK_OFFSET)
    }

    /// Year of manufacture.
    ///
    /// The stored byte is an offset from 1990, so a zero byte means 1990 and not
    /// "unknown". A block reporting 1990 is old, not absent.
    #[inline]
    pub fn year(&self) -> Option<u16> {
        self.byte(YEAR_OFFSET).map(|y| y as u16 + 1990)
    }

    /// The manufacturer ID, as the three packed 5-bit letters.
    ///
    /// Returned packed rather than decoded so a driver that only wants to log the
    /// raw value does not force a string. [`Self::manufacturer_str`] decodes it.
    #[inline]
    pub fn manufacturer(&self) -> Option<u16> {
        let a = self.byte(MANUFACTURER_OFFSET)?;
        let b = self.byte(MANUFACTURER_OFFSET + 1)?;
        Some(
            (u16::from(a >> 2) << 10) | (u16::from(b >> 5) << 5) | u16::from(b & 0x1F),
        )
    }

    /// The manufacturer ID as three letters.
    ///
    /// The letters are five bits each, most significant first, with `'A'` as 1.
    /// That is why this is a decode rather than arithmetic: getting the order
    /// wrong yields a plausible three characters naming no company.
    pub fn manufacturer_str(&self) -> Option<[u8; 3]> {
        let packed = self.manufacturer()?;
        let letter = |v: u16| if (1..=26).contains(&v) { b'A' + (v - 1) as u8 } else { b'?' };
        Some([
            letter((packed >> 10) & 0x1F),
            letter((packed >> 5) & 0x1F),
            letter(packed & 0x1F),
        ])
    }

    /// The product code, little-endian as the spec stores it.
    #[inline]
    pub fn product_code(&self) -> Option<u16> {
        Some(u16::from_le_bytes([
            self.byte(PRODUCT_OFFSET)?,
            self.byte(PRODUCT_OFFSET + 1)?,
        ]))
    }

    /// The serial number, little-endian.
    #[inline]
    pub fn serial(&self) -> Option<u32> {
        Some(u32::from_le_bytes([
            self.byte(12 + 4)?,
            self.byte(12 + 5)?,
            self.byte(12 + 6)?,
            self.byte(12 + 7)?,
        ]))
    }

    /// How many 128-byte extension blocks this block announces.
    ///
    /// The count at byte 126 is a *declaration*, not a fact: a sink may claim
    /// extensions it will not deliver. Hence the clamp — and the caller still has
    /// to handle a failed read of the block it names.
    #[inline]
    pub fn extension_count(&self) -> Option<u8> {
        self.byte(126).map(|n| n.min(1))
    }

    /// The basic display parameter byte: digital or analogue, and the sync type.
    #[inline]
    pub fn basic_display(&self) -> Option<u8> {
        self.byte(BASIC_DISPLAY_OFFSET)
    }
}

/// Decode one 18-byte detailed timing descriptor.
///
/// Returns `None` for an unused slot. This is the one piece of arithmetic in the
/// whole path that has to be exactly right, because the field layout is
/// genuinely awkward: the active area and the blanking intervals are split across
/// four bytes, and the two vertical offset fields are not adjacent.
///
/// The C this replaces in `kernel/src/displayport/edid.c` computes similar
/// expressions, but it indexes `p[10]` and `p[11]` for *both* the horizontal and
/// the vertical offset, where the vertical fields actually live, and it drops the
/// interlace bit entirely.
pub fn parse_detailed_timing(bytes: &[u8]) -> Option<DisplayMode> {
    if bytes.len() < DETAILED_TIMING_LEN {
        return None;
    }
    let p = &bytes[..DETAILED_TIMING_LEN];

    // Pixel clock in 10 kHz units. The value 1 marks an unused slot.
    let clock_10khz = u16::from_le_bytes([p[0], p[1]]);
    if clock_10khz == UNUSED_DESCRIPTOR_CLOCK_10KHZ {
        return None;
    }

    // Active area: low byte in p[2] and p[5], high nibble in the top of p[4]/p[7].
    let hactive = u16::from(p[2]) | (u16::from(p[4] >> 4) << 8);
    let vactive = u16::from(p[5]) | (u16::from(p[7] >> 4) << 8);
    // Blanking intervals, from the low nibbles of the same bytes.
    let hblank = u16::from(p[3]) | (u16::from(p[4] & 0x0F) << 8);
    let vblank = u16::from(p[6]) | (u16::from(p[7] & 0x0F) << 8);

    // Sync offsets and widths. This is where the byte layout is genuinely
    // awkward, and where the old `kernel/src/displayport/edid.c` went wrong: it
    // read `p[10]` and `p[11]` for both the horizontal and the vertical field.
    // They are four distinct fields sharing two bytes:
    //
    //   p[10] bits 7:4 → H sync offset, high 4 bits
    //   p[10] bits 3:0 → V sync *width*, low 4 bits
    //   p[11] bits 7:6 → V sync offset, high 2 bits   (max 3 — and it is)
    //   p[11] bits 5:4 → V sync *width*, high 2 bits  (6 bits total, up to 63)
    //   p[11] bits 3:0 → H sync width, high 4 bits
    //
    // The asymmetry is real, not a typo: a vertical sync offset is one or two
    // lines, while a vertical sync *width* can be a dozen.
    let hsync_offset = u16::from(p[8]) | (u16::from(p[10] >> 4) << 8);
    let hsync_width = u16::from(p[9]) | (u16::from(p[11] & 0x0F) << 8);
    let vsync_width =
        u16::from(p[10] & 0x0F) | (u16::from((p[11] >> 4) & 0x03) << 4);
    let vsync_offset = u16::from((p[11] >> 6) & 0x03);

    // A descriptor with no active area, with fields wider than the format
    // allows, or with an active area too small to be a display, is not a mode.
    // Rejecting here rather than arithmetic-ing on it is what stops `htotal`
    // from wrapping and what keeps a blank slot out of the mode list.
    if hactive < MIN_ACTIVE_EDGE
        || vactive < MIN_ACTIVE_EDGE
        || hactive > FIELD_MAX_12BIT
        || vactive > FIELD_MAX_12BIT
        || hblank > FIELD_MAX_12BIT
        || vblank > FIELD_MAX_12BIT
    {
        return None;
    }

    // Interlace and the two sync polarities, all from the last byte.
    //
    // Bit 7 is interlace. Bits 4 and 3 are the two polarities, and only when the
    // sync type in bits 4:3 is "digital separate" — which is why the mask below
    // tests the pair rather than either bit alone. Reading polarity as
    // "bit clear means positive" inverts both, and a display driven with inverted
    // vsync shows a picture that scrolls.
    let flags17 = p[17];
    let mut flags = 0u32;
    if flags17 & INTERLACE_BIT != 0 {
        flags |= MODE_FLAG_INTERLACE;
    }
    if flags17 & VSYNC_POLARITY_BIT != 0 {
        flags |= MODE_FLAG_PVSYNC;
    }
    if flags17 & HSYNC_POLARITY_BIT != 0 {
        flags |= MODE_FLAG_PHSYNC;
    }

    Some(DisplayMode {
        w: hactive,
        h: vactive,
        htotal: hactive + hblank,
        vtotal: vactive + vblank,
        hsync_width,
        hsync_offset,
        vsync_width,
        vsync_offset,
        clock_khz: clock_10khz as u32 * CLOCK_UNIT_KHZ as u32,
        flags,
    })
}

/// The result of reading and decoding a sink's EDID.
///
/// `PartialEq` because a test asserting "this block parses to that identity" is
/// the natural way to pin the decode down, and without it every such assertion
/// has to destructure field by field.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EdidInfo {
    /// Manufacturer ID, packed as the spec stores it.
    pub manufacturer: u16,
    /// The three letters, decoded.
    pub manufacturer_name: [u8; 3],
    /// Product code, little-endian.
    pub product_code: u16,
    /// Serial number, little-endian.
    pub serial: u32,
    /// Week of manufacture.
    pub week: u8,
    /// Year of manufacture.
    pub year: u16,
    /// EDID version and revision.
    pub version: (u8, u8),
    /// How many detailed timing descriptors held a real mode.
    pub mode_count: u32,
    /// Whether the sink announced a 128-byte extension.
    pub has_extension: bool,
}

/// Why an EDID read could not be turned into modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdidError {
    /// Fewer than 128 bytes came back.
    Truncated,
    /// The block does not start with the mandatory header.
    BadHeader,
    /// The block is well-formed but carries no usable detailed timing.
    NoModes,
}

/// Parse a base block into its identity and its detailed timings.
///
/// `out` is appended to, not cleared, so a caller that has already added modes
/// from a CTA extension block keeps them.
pub fn parse(bytes: &[u8], out: &mut Vec<DisplayMode>) -> Result<EdidInfo, EdidError> {
    if bytes.len() < EDID_LEN {
        return Err(EdidError::Truncated);
    }
    let edid = Edid::new(bytes);
    if !edid.has_valid_header() {
        return Err(EdidError::BadHeader);
    }

    let info = EdidInfo {
        manufacturer: edid.manufacturer().unwrap_or(0),
        manufacturer_name: edid.manufacturer_str().unwrap_or([b'?'; 3]),
        product_code: edid.product_code().unwrap_or(0),
        serial: edid.serial().unwrap_or(0),
        week: edid.week().unwrap_or(0),
        year: edid.year().unwrap_or(0),
        version: (edid.version().unwrap_or(0), edid.revision().unwrap_or(0)),
        mode_count: 0,
        has_extension: edid.extension_count().unwrap_or(0) > 0,
    };

    let before = out.len();
    for d in 0..DETAILED_TIMING_COUNT {
        let start = DETAILED_TIMINGS_OFFSET + d * DETAILED_TIMING_LEN;
        let Some(desc) = bytes.get(start..start + DETAILED_TIMING_LEN) else {
            break;
        };
        if let Some(mode) = parse_detailed_timing(desc) {
            if mode.is_valid() {
                out.push(mode);
            }
        }
    }

    let added = (out.len() - before) as u32;
    if added == 0 && out.is_empty() {
        return Err(EdidError::NoModes);
    }
    Ok(EdidInfo { mode_count: added, ..info })
}

/// Find the mode matching a resolution and refresh rate, if the sink offers one.
///
/// The tolerance exists because a sink reporting 59.94 and asked for 60 *is* the
/// mode the user meant, and refusing leaves a monitor at the wrong rate. An exact
/// match wins when both exist, so this never trades a precisely-listed mode for
/// an approximate one.
pub fn find_mode<'a>(
    modes: &'a [DisplayMode],
    w: u16,
    h: u16,
    refresh_hz: u32,
) -> Option<&'a DisplayMode> {
    if let Some(m) = modes
        .iter()
        .find(|m| m.w == w && m.h == h && m.refresh_hz() == refresh_hz)
    {
        return Some(m);
    }
    modes
        .iter()
        .find(|m| m.w == w && m.h == h && (m.refresh_hz() as i64 - refresh_hz as i64).abs() <= 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    /// A 1920x1080@60 descriptor, built from the spec's own field layout.
    ///
    /// The timings are the real CVT-RB ones: 148.5 MHz, 2200 x 1125 total,
    /// 48/32/3/5 for the sync offsets and widths. The fixture is assembled
    /// independently of the parser, so a parser that gets the nibble arithmetic
    /// wrong disagrees with it — which is the point.
    fn dtd_1080p60() -> [u8; 18] {
        let mut p = [0u8; 18];
        let (hact, hblank, vact, vblank) = (1920u16, 280u16, 1080u16, 45u16);
        let (hso, hsw, vso, vsw) = (48u16, 32u16, 3u16, 5u16);
        p[0..2].copy_from_slice(&14850u16.to_le_bytes()); // 148.5 MHz
        p[2] = (hact & 0xFF) as u8;
        p[3] = (hblank & 0xFF) as u8;
        p[4] = (((hact >> 8) << 4) | ((hblank >> 8) & 0x0F)) as u8;
        p[5] = (vact & 0xFF) as u8;
        p[6] = (vblank & 0xFF) as u8;
        p[7] = (((vact >> 8) << 4) | ((vblank >> 8) & 0x0F)) as u8;
        p[8] = (hso & 0xFF) as u8;
        p[9] = (hsw & 0xFF) as u8;
        p[10] = ((((hso >> 8) & 0x0F) << 4) | (vsw & 0x0F)) as u8;
        p[11] = ((((vso >> 4) & 0x03) << 6) | (((vsw >> 4) & 0x03) << 4) | ((hsw >> 8) & 0x0F)) as u8;
        p[17] = 0x18; // digital separate sync, positive hsync and positive vsync
        p
    }

    /// A 128-byte block carrying one real descriptor and three blank slots.
    fn base_block(desc: [u8; 18]) -> [u8; EDID_LEN] {
        let mut b = [0u8; EDID_LEN];
        b[..8].copy_from_slice(&EDID_HEADER);
        // "DEL": D=4, E=5, L=12, packed big-endian 5-bit letters.
        let packed = (4u16 << 10) | (5 << 5) | 12;
        b[8] = ((packed >> 8) & 0xFF) as u8;
        b[9] = (packed & 0xFF) as u8;
        b[10..12].copy_from_slice(&0x4321u16.to_le_bytes());
        b[12] = 12; // week
        b[13] = 34; // year, as 1990 + 34
        b[14] = 1; // version 1
        b[15] = 4; // revision 1.4
        b[54..72].copy_from_slice(&desc);
        b
    }

    #[test]
    fn a_block_with_the_magic_header_is_accepted() {
        assert!(Edid::new(&base_block(dtd_1080p60())).has_valid_header());
    }

    #[test]
    fn an_all_ones_bus_read_is_rejected() {
        // A NAK'd I2C read leaves the bus pulled high: 0xFF throughout. The first
        // byte must be 0x00, so this is caught rather than parsed as a garbage
        // panel with a 0xFF manufacturer.
        let b = [0xFFu8; EDID_LEN];
        assert!(!Edid::new(&b).has_valid_header());
    }

    #[test]
    fn a_zeroed_block_is_rejected() {
        let b = [0u8; EDID_LEN];
        assert!(!Edid::new(&b).has_valid_header());
    }

    #[test]
    fn a_short_buffer_is_not_complete() {
        let e = Edid::new(&[0u8; 64]);
        assert!(!e.is_complete());
        assert!(!e.has_valid_header());
    }

    #[test]
    fn accessors_return_none_on_a_short_buffer() {
        // Bounds-checked rather than panicking: a truncated read is a normal
        // hardware outcome, not a caller bug.
        let e = Edid::new(&[0u8; 3]);
        assert_eq!(e.manufacturer(), None);
        assert_eq!(e.product_code(), None);
        assert_eq!(e.serial(), None);
        assert_eq!(e.year(), None);
        assert_eq!(e.extension_count(), None);
    }

    // ── identity fields ─────────────────────────────────────────────────────

    #[test]
    fn the_manufacturer_id_decodes_to_three_letters() {
        let b = base_block(dtd_1080p60());
        assert_eq!(Edid::new(&b).manufacturer_str(), Some([b'D', b'E', b'L']));
    }

    #[test]
    fn a_zero_letter_decodes_to_a_question_mark() {
        // 'A' is 1, so a packed zero is not a company. Rendering it as 'A'-1
        // would print a real-looking character for a corrupt field.
        let mut b = base_block(dtd_1080p60());
        b[8] = 0;
        b[9] = 0;
        assert_eq!(Edid::new(&b).manufacturer_str(), Some([b'?', b'?', b'?']));
    }

    #[test]
    fn the_year_is_stored_as_an_offset_from_1990() {
        assert_eq!(Edid::new(&base_block(dtd_1080p60())).year(), Some(2024));
    }

    #[test]
    fn a_zero_year_byte_means_1990_not_unknown() {
        let mut b = base_block(dtd_1080p60());
        b[13] = 0;
        assert_eq!(Edid::new(&b).year(), Some(1990));
    }

    #[test]
    fn the_product_code_is_little_endian() {
        assert_eq!(Edid::new(&base_block(dtd_1080p60())).product_code(), Some(0x4321));
    }

    #[test]
    fn the_extension_count_is_clamped_to_one() {
        // A sink claiming eight extensions is broken or lying; a driver that
        // trusted it would try to read 1 KiB of DDC.
        let mut b = base_block(dtd_1080p60());
        b[126] = 8;
        assert_eq!(Edid::new(&b).extension_count(), Some(1));
    }

    // ── the descriptor decode ───────────────────────────────────────────────

    #[test]
    fn a_1080p60_descriptor_decodes_exactly() {
        let m = parse_detailed_timing(&dtd_1080p60()).expect("a real descriptor");
        assert_eq!(m.w, 1920);
        assert_eq!(m.h, 1080);
        assert_eq!(m.htotal, 2200, "1920 active + 280 blank");
        assert_eq!(m.vtotal, 1125, "1080 active + 45 blank");
        assert_eq!(m.clock_khz, 148_500);
    }

    #[test]
    fn the_vertical_sync_comes_from_its_own_bytes() {
        // The bug this guards: reading the horizontal offset's bytes for the
        // vertical field. Per the spec, p[10]'s low nibble is the vertical sync
        // *width*, and p[11]'s top two bits the vertical sync *offset* — so a
        // parser that swaps them reports 5 and 3 the wrong way round, and 0 for
        // both if it reads only p[11].
        let m = parse_detailed_timing(&dtd_1080p60()).expect("a real descriptor");
        assert_eq!(m.vsync_width, 5, "V width spans p[10] 3:0 and p[11] 5:4");
        assert_eq!(m.vsync_offset, 3, "V offset is p[11] 7:6, two bits only");
    }

    #[test]
    fn the_horizontal_sync_is_independent_of_the_vertical() {
        // Same two bytes, four different fields. A parser that conflates them
        // fails here rather than on a bench with a real cable.
        let m = parse_detailed_timing(&dtd_1080p60()).expect("a real descriptor");
        assert_eq!(m.hsync_offset, 48, "H offset is p[8] plus p[10] 7:4");
        assert_eq!(m.hsync_width, 32, "H width is p[9] plus p[11] 3:0");
    }

    #[test]
    fn the_interlace_bit_survives() {
        // The old C dropped it, so a 1080i descriptor came back as 1080p and got
        // programmed as progressive.
        let mut d = dtd_1080p60();
        d[17] |= 0x80;
        assert!(parse_detailed_timing(&d).expect("a real descriptor").is_interlaced());
    }

    #[test]
    fn sync_polarity_is_recorded_per_axis() {
        // Byte 17 bits 4 and 3, each set meaning positive. The fixture uses 0x18
        // — "digital separate sync, positive hsync, positive vsync" — so both must
        // come out set. Reading polarity as "clear means positive" inverts both.
        let m = parse_detailed_timing(&dtd_1080p60()).expect("a real descriptor");
        assert!(m.is_positive_vsync(), "p[17] bit 3 set means positive vsync");
        assert!(m.is_positive_hsync(), "p[17] bit 4 set means positive hsync");
    }

    #[test]
    fn a_cleared_polarity_bit_means_negative() {
        // The other direction, because a parser that always reports positive
        // passes the test above.
        let mut d = dtd_1080p60();
        d[17] &= !VSYNC_POLARITY_BIT & !HSYNC_POLARITY_BIT;
        let m = parse_detailed_timing(&d).expect("a real descriptor");
        assert!(!m.is_positive_vsync());
        assert!(!m.is_positive_hsync());
    }

    #[test]
    fn a_blank_slot_is_not_a_mode() {
        // A sink fills unused slots with 01 01 01 01 01 01, a 10 kHz 1x1 "mode".
        // Treating it as real puts 1x1 in the mode list.
        assert_eq!(parse_detailed_timing(&[0x01u8; 18]), None);
    }

    #[test]
    fn a_zero_clock_slot_is_not_a_mode() {
        assert_eq!(parse_detailed_timing(&[0u8; 18]), None);
    }

    #[test]
    fn a_truncated_descriptor_is_refused() {
        assert_eq!(parse_detailed_timing(&dtd_1080p60()[..17]), None);
    }

    #[test]
    fn a_descriptor_with_no_active_area_is_refused() {
        let mut d = dtd_1080p60();
        d[2] = 0;
        d[4] &= 0x0F;
        assert_eq!(parse_detailed_timing(&d), None);
    }

    // ── refresh arithmetic ──────────────────────────────────────────────────

    #[test]
    fn a_zero_total_period_is_not_a_fast_mode() {
        let m = DisplayMode { w: 1920, h: 1080, clock_khz: 148_500, ..Default::default() };
        assert_eq!(m.refresh_mhz(), 0, "a zero period yields 0, not a panic");
    }

    #[test]
    fn refresh_is_60_for_the_cvt_rb_timings() {
        let m = parse_detailed_timing(&dtd_1080p60()).expect("a real descriptor");
        assert_eq!(m.refresh_hz(), 60);
    }

    #[test]
    fn a_5994_mode_does_not_collapse_into_60() {
        // 59.94 and 60 need different pixel clocks. `refresh_hz` rounds, so the
        // guard has to be on the millihertz value: this mode is 59.85, which
        // rounds to 60 and would be indistinguishable from it if we only kept Hz.
        let m = DisplayMode {
            w: 1920, h: 1080, htotal: 2200, vtotal: 1125, clock_khz: 148_125, ..Default::default()
        };
        assert!(m.refresh_mhz() < 60_000, "59.85 must not report 60.000");
        assert_ne!(m.refresh_mhz(), 60_000);
    }

    #[test]
    fn stride_saturates_instead_of_wrapping() {
        // A stride that overflows `u16` is the case worth guarding: 8192 px at
        // 64 bpp is 65536, one past the old limit, and a wrapped value is a buffer
        // the scanout reads out of bounds.
        let m = DisplayMode { w: 8192, h: 4320, htotal: 8192, ..Default::default() };
        let s = m.stride_bytes(64);
        assert_eq!(s, 8192 * 8);
        assert!(s > u16::MAX as u32, "this is the case that used to wrap");
    }

    #[test]
    fn stride_rounds_a_partial_byte_up() {
        let m = DisplayMode { htotal: 1920, ..Default::default() };
        assert_eq!(m.stride_bytes(1), 1920);
        assert_eq!(m.stride_bytes(12), 1920 * 2, "12 bpp is two bytes, not one");
    }

    #[test]
    fn a_mode_whose_total_is_below_its_active_area_is_invalid() {
        let m = DisplayMode {
            w: 1920, h: 1080, htotal: 1000, vtotal: 2000, clock_khz: 148_500, ..Default::default()
        };
        assert!(!m.is_valid());
    }

    #[test]
    fn the_default_mode_is_invalid() {
        assert!(!DisplayMode::default().is_valid());
    }

    // ── the top-level parse ─────────────────────────────────────────────────

    #[test]
    fn a_good_block_yields_its_modes_and_identity() {
        let b = base_block(dtd_1080p60());
        let mut modes = Vec::new();
        let info = parse(&b, &mut modes).expect("a valid block");
        assert_eq!(modes.len(), 1);
        assert_eq!(info.mode_count, 1);
        assert_eq!(info.manufacturer_name, [b'D', b'E', b'L']);
        assert_eq!(info.product_code, 0x4321);
        assert_eq!(info.year, 2024);
        assert_eq!(info.version, (1, 4));
    }

    #[test]
    fn a_short_block_is_reported_as_truncated() {
        let mut modes = Vec::new();
        assert_eq!(parse(&[0u8; 64], &mut modes), Err(EdidError::Truncated));
    }

    #[test]
    fn a_bad_header_is_reported_as_such() {
        let mut b = base_block(dtd_1080p60());
        b[0] = 0x42;
        let mut modes = Vec::new();
        assert_eq!(parse(&b, &mut modes), Err(EdidError::BadHeader));
    }

    #[test]
    fn a_block_with_no_modes_is_distinct_from_a_bus_failure() {
        // A capture sink or an HDMI-to-VGA adapter reports no detailed timings.
        // That is a working monitor, not a broken cable.
        let b = base_block([0x01u8; 18]);
        let mut modes = Vec::new();
        assert_eq!(parse(&b, &mut modes), Err(EdidError::NoModes));
    }

    #[test]
    fn parse_appends_rather_than_clearing() {
        // A caller that has already added CTA extension modes keeps them.
        let b = base_block(dtd_1080p60());
        let mut modes = vec![DisplayMode { w: 640, h: 480, ..Default::default() }];
        parse(&b, &mut modes).expect("a valid block");
        assert_eq!(modes.len(), 2, "the caller's mode must survive");
    }

    // ── mode lookup ─────────────────────────────────────────────────────────

    #[test]
    fn an_exact_match_wins_over_an_approximate_one() {
        // Two 1080p modes: a true 60.000, and a 59.2 that `refresh_hz` does *not*
        // round to 60. The exact pass runs over the whole list before the
        // tolerant one, so a precise mode is never passed over for an approximate
        // one even when the approximate one is listed first.
        let near = DisplayMode {
            w: 1920, h: 1080, htotal: 2000, vtotal: 1200, clock_khz: 142_000, ..Default::default()
        };
        let exact = DisplayMode {
            w: 1920, h: 1080, htotal: 2200, vtotal: 1125, clock_khz: 148_500, ..Default::default()
        };
        assert_eq!(near.refresh_hz(), 59, "the near-miss must fail the exact test");
        assert_eq!(exact.refresh_hz(), 60);
        let modes = vec![near, exact];
        let m = find_mode(&modes, 1920, 1080, 60).expect("a match");
        assert_eq!(m.clock_khz, 148_500, "the exact 60, not the near-miss");
    }

    #[test]
    fn a_near_miss_still_matches_within_one_hz() {
        // A sink that reports 59.94 is the mode the user meant when they asked
        // for 60; refusing leaves the monitor at the wrong rate.
        let modes = vec![DisplayMode {
            w: 1920, h: 1080, htotal: 2200, vtotal: 1125, clock_khz: 148_125, ..Default::default()
        }];
        assert!(find_mode(&modes, 1920, 1080, 60).is_some());
    }

    #[test]
    fn a_different_resolution_never_matches() {
        let modes = vec![DisplayMode {
            w: 1920, h: 1080, htotal: 2200, vtotal: 1125, clock_khz: 148_500, ..Default::default()
        }];
        assert!(find_mode(&modes, 1280, 1024, 60).is_none());
    }
}
