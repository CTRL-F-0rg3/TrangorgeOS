// Linux Capability bits mapping to TrangorgeOS native capabilities
pub const CAP_CHOWN: u32 = 0;
pub const CAP_DAC_OVERRIDE: u32 = 1;
pub const CAP_FOWNER: u32 = 3;
pub const CAP_KILL: u32 = 5;
pub const CAP_NET_BIND_SERVICE: u32 = 10;
pub const CAP_NET_RAW: u32 = 13;
pub const CAP_SYS_MODULE: u32 = 16;
pub const CAP_SYS_RAWIO: u32 = 17;
pub const CAP_SYS_ADMIN: u32 = 21;
pub const CAP_SYS_BOOT: u32 = 22;

// Check if a Linux capability requires a specific TrangorgeOS native cap
#[inline]
pub fn requires_native_cap(linux_cap: u32) -> Option<kstd_base::CapId> {
    match linux_cap {
        CAP_SYS_ADMIN | CAP_SYS_MODULE => Some(kstd_base::CapId(0x01)), // Native Root/Kernel
        CAP_NET_RAW | CAP_NET_BIND_SERVICE => Some(kstd_base::CapId(0x10)), // Native Net
        CAP_SYS_RAWIO => Some(kstd_base::CapId(0x20)), // Native HW/IO
        _ => None, // Mapped via standard user sandbox rules
    }
}