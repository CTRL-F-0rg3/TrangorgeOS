#![no_std]

// Re-export foundations so higher layers depend on a single crate.
pub use kstd_base;
pub use kstd_alloc;
pub use kstd_core;
pub use kstd_data;
pub use kstd_io;

use kstd_base::{PhysAddr, VirtAddr, Size, WorldId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PartitionKind {
    KernelCore    = 0,
    DriverSpace   = 1,
    UserSpace     = 3,
    LinuxCompat   = 4,
    WindowsCompat = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AbiFlavor {
    NativeTrangorge,
    PosixLinux,
    Win32Nt,
    DriverSpaceRpc,
}

#[derive(Debug, Clone, Copy)]
pub struct SandboxRules {
    pub virt_start: VirtAddr,
    pub virt_end: VirtAddr,
    pub phys_base: PhysAddr,
    pub mem_size: Size,
    pub max_ipc_ports: u32,
    pub max_open_handles: u32,
    pub allowed_irq_mask: u64,
    pub can_access_hw_direct: bool,
    pub can_map_arbitrary_phys: bool,
}

impl SandboxRules {
    #[inline]
    pub const fn contains_virt(&self, addr: VirtAddr) -> bool {
        addr.0 >= self.virt_start.0 && addr.0 < self.virt_end.0
    }
}

// Represents an isolated execution environment.
// Binds a World/Process to its memory sandbox, capabilities, and ABI translation hooks.
pub struct OsWorkspace {
    pub id: WorldId,
    pub kind: PartitionKind,
    pub abi: AbiFlavor,
    pub asid: u16, 
    pub cr3_phys: u64,
    pub rules: SandboxRules,
    pub syscall_entry: u64,
    pub abi_translate_hook: u64,
}

impl OsWorkspace {
    #[inline]
    pub const fn is_native(&self) -> bool {
        matches!(self.abi, AbiFlavor::NativeTrangorge)
    }

    #[inline]
    pub const fn is_compat(&self) -> bool {
        matches!(self.abi, AbiFlavor::PosixLinux | AbiFlavor::Win32Nt)
    }

    #[inline]
    pub const fn is_driver(&self) -> bool {
        matches!(self.kind, PartitionKind::DriverSpace)
    }
    
    #[inline]
    pub fn check_memory_access(&self, addr: VirtAddr, size: usize) -> bool {
        let end = addr.0.saturating_add(size as u64);
        end <= self.rules.virt_end.0 && addr.0 >= self.rules.virt_start.0
    }
}