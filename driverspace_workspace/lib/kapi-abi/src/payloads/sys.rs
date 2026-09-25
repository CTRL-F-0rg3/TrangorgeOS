//! System-level message payloads (logging and friends).

/// Drivers forward log messages through the manager (`DsCmd::SysLog`).
///
/// The struct only describes metadata; the `module` and `msg` bytes follow
/// the payload inside the same IPC buffer.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LogPayload {
    /// Log level (matches `ds-log`'s `LogLevel`)
    pub level: u8,

    /// Module name length in bytes
    pub module_len: u16,

    /// Message body length in bytes
    pub msg_len: u16,

    /// Reserved, for alignment
    pub _pad: u32,
}

/// A driver's request for one ACPI SDT (`DsCmd::SysAcpiTable`).
///
/// Drivers must not walk the RSDT themselves - that is the kernel's
/// firmware job. A driver only names the table it needs; the kernel locates
/// it, validates it and maps it into the caller's address space.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AcpiTableRequest {
    /// Four-character signature as a little-endian `u32`, e.g. `u32::from_le_bytes(*b"DMAR")`.
    pub signature: u32,
    /// Instance index when a signature occurs more than once; `0` is the first.
    pub instance: u32,
    pub _pad: u64,
}

impl AcpiTableRequest {
    /// Build a request from a four-character signature.
    #[inline]
    pub const fn new(signature: [u8; 4], instance: u32) -> Self {
        Self {
            signature: u32::from_le_bytes(signature),
            instance,
            _pad: 0,
        }
    }

    /// The four-character signature carried by this request.
    #[inline]
    pub const fn signature_bytes(self) -> [u8; 4] {
        self.signature.to_le_bytes()
    }
}

/// Register convention the kernel uses when replying to `SysAcpiTable`.
///
/// In the reply `DsMsg::arg0` is the table's virtual base in the caller's
/// address space and `DsMsg::arg1` its length in bytes. The table must stay
/// readable until the caller releases it.
pub mod acpi_table_reply {
    /// `DsMsg::arg0`: mapped virtual base.
    pub const VIRT_BASE: usize = 0;
    /// `DsMsg::arg1`: table length in bytes.
    pub const LENGTH: usize = 1;
}
