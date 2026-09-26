//! Which AMD graphics architecture a device is.
//!
//! # Why a family, not a product name
//!
//! AMD's product names span architectures. "Navi" is Navi 10, 14, 21 and 22 -
//! one register map - while "Vega" is a discrete HBM part and an APU that
//! differ in how they reach memory. So the driver branches on the
//! architecture, and the name is only for a human.
//!
//! # Why this differs from Intel
//!
//! Intel's generations are linear: Gen12 is newer than Gen11 everywhere. AMD's
//! are not. Polaris is GCN 4, *newer* than Fiji which is also GCN 4, and the
//! APU line jumps generations at its own pace - an APU shipped in 2023 can be
//! RDNA 2 while a discrete card from 2019 is GCN 5. A driver that ordered
//! generations by number would get this backwards, which is why
//! [`Arch`] is an explicit enumeration rather than a scale.
//!
//! # Scope
//!
//! The generations listed are the ones this driver drives. Older parts are
//! recognised as [`Arch::Legacy`] and refused: a TeraScale register map is a
//! different driver, not an older version of this one.

use kapi_abi::DsError;

/// An AMD graphics architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Arch {
    /// Pre-GCN: R300 through R700, and the TeraScale parts. Recognised, refused.
    Legacy,
    /// GCN 1.x, 2011-2013.
    Gcn1,
    /// GCN 4, 2013-2016.
    Gcn4,
    /// GCN 5, "Polaris", 2016-2019.
    Gcn5,
    /// Vega, 2017. AMD's first display part with hardware memory.
    Vega,
    /// RDNA 1, "Navi 10/14", 2019-2020.
    Rdna1,
    /// RDNA 2, 2020-2023.
    Rdna2,
    /// RDNA 3, 2022-2025.
    Rdna3,
    /// RDNA 4, 2025 onwards.
    Rdna4,
    /// A part this driver has no entry for.
    Unknown,
}

impl Arch {
    /// Classify a family name, as `pci.ids` spells it.
    ///
    /// Takes the *family*, not the device id: the id table maps ids to
    /// families, and this maps families to architectures.
    pub const fn from_family(family: &str) -> Self {
        // The match is on length first, because the names overlap:
        // "Navi" and "Navi2" are different architectures and a prefix match
        // would put Navi 23 in RDNA 1.
        match family.as_bytes() {
            b"Tahiti" | b"Pitcairn" | b"Verde" | b"Oland" | b"Hainan" | b"Hawaii"
            | b"Bonaire" | b"Kaveri" | b"Kabini" | b"Mullins" | b"Topaz" | b"Kaori" => Self::Gcn1,

            b"Tonga" | b"Fiji" | b"Carrizo" | b"Stoney" | b"Barts" => Self::Gcn4,

            b"Ellesmere" | b"Baffin" | b"Lexa" | b"Emea" => Self::Gcn5,

            b"Vega" | b"Vega10" | b"Vega12" | b"Vega20" | b"Vega3" | b"Aruba"
            | b"Lesotho" | b"Sienna" => Self::Vega,

            // Navi carries its step, because "Navi" alone does not say which
            // RDNA generation it is. The step is the generation, and getting it
            // wrong means a dispatch sized for a 32-wide wave on a 64-wide one.
            b"Navi10" | b"Navi12" | b"Navi14" => Self::Rdna1,
            b"Navi21" | b"Navi22" | b"Navi23" | b"Navi24" | b"Renoir"
            | b"Cezanne" | b"Lucienne" | b"Raphael" | b"Barcelo" | b"Dali"
            | b"Mendocino" | b"YellowCarp" | b"Sian" | b"Aquaria" => Self::Rdna2,
            b"Navi31" | b"Navi32" | b"Navi33" | b"Phoenix"
            | b"Phoenix1" | b"Phoenix2" | b"Rembrandt" | b"Hawk" | b"HawkPoint1"
            | b"HawkPoint2" | b"Strix" | b"Krackan" | b"Krackan2" | b"VanGogh"
            | b"Gorgon" => Self::Rdna3,
            b"Navi44" | b"Navi48" | b"Strixhalo" => Self::Rdna4,

            // "Polaris" is the marketing name for the GCN 5 parts, and
            // "Ellesmere"/"Baffin" are the same silicon. It sits in the GCN 5
            // arm above, not here.
            b"Polaris" => Self::Gcn5,

            // A Ryzen APU that predates Navi: Picasso and Raven are RDNA 2
            // under different names, which is the case that makes "newer
            // number" a bad way to order AMD's parts.
            b"Picasso/Raven" | b"Raven" => Self::Rdna2,

            _ => Self::Unknown,
        }
    }

    /// Whether this driver can drive the architecture.
    pub const fn is_supported(self) -> bool {
        matches!(
            self,
            Self::Gcn1 | Self::Gcn4 | Self::Gcn5 | Self::Vega
                | Self::Rdna1 | Self::Rdna2 | Self::Rdna3 | Self::Rdna4
        )
    }

    /// A short name for logs and `GpuInfo`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Legacy => "Legacy",
            Self::Gcn1 => "GCN 1.x",
            Self::Gcn4 => "GCN 4",
            Self::Gcn5 => "GCN 5",
            Self::Vega => "Vega",
            Self::Rdna1 => "RDNA 1",
            Self::Rdna2 => "RDNA 2",
            Self::Rdna3 => "RDNA 3",
            Self::Rdna4 => "RDNA 4",
            Self::Unknown => "unknown",
        }
    }

    /// Refuse an architecture this driver does not support.
    pub const fn require_supported(self) -> Result<Self, DsError> {
        if self.is_supported() {
            Ok(self)
        } else {
            Err(DsError::DeviceNotFound)
        }
    }

    /// Whether the architecture's compute units are a different size.
    ///
    /// Not a capability the framework needs; it is the fact that decides how
    /// wide a dispatch is, and getting it wrong is a correctness problem on the
    /// GPU rather than a slow path.
    pub const fn wave_size(self) -> u32 {
        match self {
            // GCN through Vega used 64-wide waves; RDNA halved them.
            Self::Gcn1 | Self::Gcn4 | Self::Gcn5 | Self::Vega => 64,
            Self::Rdna1 | Self::Rdna2 | Self::Rdna3 | Self::Rdna4 => 32,
            _ => 0,
        }
    }
}
