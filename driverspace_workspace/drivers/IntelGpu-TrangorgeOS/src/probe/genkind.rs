//! Which Intel graphics generation a device is.
//!
//! # Why generation, not integrated-versus-discrete
//!
//! Intel's integrated and discrete parts do **not** need separate drivers.
//! Arc DG1 and DG2 are built on the same Xe cores as the integrated Iris Xe
//! of the same period, and Mesa's `iris` drives both from one code path. What
//! actually differs is the *generation*, because that is what changes the
//! register map, the command encoding and the memory model.
//!
//! So the split in this driver is:
//!
//! * [`GenKind`] - chooses the register map and the command format. This is the
//!   real compatibility boundary.
//! * `super::mm::MemoryModel` - chooses how a buffer is placed. A discrete
//!   card is not a separate driver; it is a generation with local memory.
//!
//! # Scope
//!
//! Gen8 and earlier are recognised but refused: the register maps that old
//! would need are a different driver, not an older version of this one.

use kapi_abi::DsError;

/// An Intel graphics generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GenKind {
    /// Broadwell, Gen8. Recognised, not driven.
    Gen8,
    /// Skylake and Kaby Lake, Gen9.
    Gen9,
    /// Coffee Lake, Gen9 on a changed GT block layout.
    Gen9_5,
    /// Comet Lake, the other Gen9 variant.
    // The digit-underscore spelling is deliberate: `Gen95Cml` would read as
    // Gen 9.5 CML, which is not what it is.
    #[allow(non_camel_case_types)]
    Gen9_5_Cml,
    /// Ice Lake, Gen11.
    Gen11,
    /// Tiger Lake, Gen12.
    Gen12,
    /// Alder Lake, Gen12 with the DG2 register additions.
    Gen12_5,
    /// Raptor Lake, Meteor Lake, Arrow Lake, Lunar Lake.
    Gen12_7,
    /// Battlemage, Gen13.
    Gen13,
    /// Newer than anything this driver knows.
    Gen14,
    /// Recognised as Intel, but older than this driver supports.
    Unsupported,
}

impl GenKind {
    /// Classify a 16-bit PCI device id.
    ///
    /// `None` when the id is not an Intel display device at all, which is the
    /// common case: most devices on a bus are not a GPU.
    pub const fn from_device_id(device_id: u16) -> Option<Self> {
        Some(match device_id {
            // Broadwell.
            0x1602 | 0x1606 | 0x160A | 0x160B | 0x160D | 0x160E
            | 0x1612 | 0x1616 | 0x161A | 0x161B | 0x161D | 0x161E
            | 0x1622 | 0x1626 | 0x162A | 0x162B | 0x162D | 0x162E => Self::Gen8,

            // Skylake, and Kaby Lake which shares its map exactly.
            0x1902 | 0x1906 | 0x190A | 0x190B | 0x190E
            | 0x1912 | 0x1913 | 0x1915 | 0x1916 | 0x1917
            | 0x191A | 0x191B | 0x191D | 0x191E | 0x1921
            | 0x1923 | 0x1926 | 0x1927 | 0x192A | 0x192B
            | 0x192D | 0x1932 | 0x193A | 0x193B | 0x193D
            | 0x5902 | 0x5906 | 0x5908 | 0x590A | 0x590B | 0x590E
            | 0x5912 | 0x5913 | 0x5915 | 0x5917 => Self::Gen9,

            // Coffee Lake: Gen9 numbering, different GT block layout.
            0x3E00..=0x3E4F | 0x3E50..=0x3E6F | 0x3E70..=0x3E9F => Self::Gen9_5,

            // Comet Lake.
            0x9B40..=0x9B4F | 0x9B50..=0x9B5B => Self::Gen9_5_Cml,

            // Ice Lake.
            0x8A50..=0x8A5D => Self::Gen11,

            // Tiger Lake.
            0x9A40..=0x9A5C | 0x9A60..=0x9A7D => Self::Gen12,

            // Alder Lake. The `dg2_*` ids are the discrete parts and share
            // this register map, which is the point: Arc needs no new driver.
            0x4680..=0x4693
            | 0x4905..=0x4909
            | 0x5690..=0x56B6 => Self::Gen12_5,

            // Raptor Lake, Meteor Lake, Arrow Lake, Lunar Lake.
            0xA780..=0xA783 | 0xA7A0..=0xA7AD | 0xA7B0..=0xA7BF
            | 0x642A..=0x642D | 0x6430 | 0x6437
            | 0x7D40 | 0x7D45 | 0x7D55 | 0x7D60 | 0x7DD5 => Self::Gen12_7,

            // Battlemage.
            0xE20B..=0xE223 => Self::Gen13,

            _ => return None,
        })
    }

    /// Whether this driver can drive the generation at all.
    pub const fn is_supported(self) -> bool {
        !matches!(self, Self::Unsupported | Self::Gen8)
    }

    /// The shortest name a human recognises, for logs and `GpuInfo`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gen8 => "Gen8",
            Self::Gen9 => "Gen9",
            Self::Gen9_5 => "Gen9.5",
            Self::Gen9_5_Cml => "Gen9.5 (CML)",
            Self::Gen11 => "Gen11",
            Self::Gen12 => "Gen12",
            Self::Gen12_5 => "Gen12.5",
            Self::Gen12_7 => "Gen12.7",
            Self::Gen13 => "Gen13",
            Self::Gen14 => "Gen14",
            Self::Unsupported => "unsupported",
        }
    }

    /// Refuse a generation this driver does not support.
    ///
    /// A driver that guessed here would write registers that mean something
    /// else, so this is the point where an old card is turned away.
    pub const fn require_supported(self) -> Result<Self, DsError> {
        if self.is_supported() {
            Ok(self)
        } else {
            Err(DsError::DeviceNotFound)
        }
    }
}
