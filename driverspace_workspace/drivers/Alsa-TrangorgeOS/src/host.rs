//! The `Platform` implementation backed by `ds-manager`.
//!
//! Every method here is one `kapi_syscall::sys_ipc_call`. Nothing in this
//! module touches a physical address on its own: the manager owns resource
//! allocation and hands back a mapping, which is what keeps the driver-space
//! isolation meaningful (see `Workspace.md`, rule 5).

use crate::driver::Platform;
use kapi_abi::{DsCmd, DsError, DsMsg, Handle};
use kapi_syscall::sys_ipc_call;

/// Production platform: talks to `ds-manager` over IPC.
#[derive(Clone, Copy, Debug)]
pub struct SyscallPlatform {
    manager: Handle,
}

impl SyscallPlatform {
    #[inline]
    pub const fn new() -> Self {
        Self { manager: Handle::MANAGER }
    }

    /// Point the platform at a different manager endpoint.
    #[inline]
    pub const fn with_manager(manager: Handle) -> Self {
        Self { manager }
    }

    /// The endpoint requests are sent to.
    #[inline]
    pub const fn manager(&self) -> Handle {
        self.manager
    }

    /// Issue one IPC call and translate a non-zero status into a `DsError`.
    fn call(&self, cmd: DsCmd, arg0: u64, arg1: u64, arg2: u64) -> Result<DsMsg, DsError> {
        let reply = sys_ipc_call(cmd, arg0, arg1, arg2);
        if reply.is_ok() {
            Ok(reply)
        } else {
            Err(DsError::from_u32(reply.status as u32))
        }
    }
}

impl Default for SyscallPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for SyscallPlatform {
    fn map_mmio(&self, phys: u64, size: u64) -> Result<u64, DsError> {
        if size == 0 {
            return Err(DsError::InvalidMessage);
        }
        // The manager replies with the virtual base it chose.
        let reply = self.call(DsCmd::SysMapMmio, phys, size, 0)?;
        let virt = reply.arg0;
        if virt == 0 {
            return Err(DsError::DeviceFault);
        }
        Ok(virt)
    }

    fn unmap_mmio(&self, virt: u64, size: u64) -> Result<(), DsError> {
        self.call(DsCmd::SysUnmapMmio, virt, size, 0).map(|_| ())
    }

    fn alloc_dma(&self, size: u64) -> Result<(u64, u64), DsError> {
        if size == 0 {
            return Err(DsError::InvalidMessage);
        }
        // Reply carries the physical base in arg0 and the virtual base in arg1.
        let reply = self.call(DsCmd::SysAllocDma, size, 0, 0)?;
        let phys = reply.arg0;
        let virt = reply.arg1;
        if phys == 0 || virt == 0 {
            return Err(DsError::OutOfMemory);
        }
        Ok((phys, virt))
    }

    fn free_dma(&self, phys: u64, size: u64) -> Result<(), DsError> {
        self.call(DsCmd::SysFreeDma, phys, size, 0).map(|_| ())
    }

    fn read_pci(&self, requester: u32, offset: u32) -> Result<u32, DsError> {
        let reply = self.call(DsCmd::PciRead, requester as u64, offset as u64, 0)?;
        Ok(reply.arg0 as u32)
    }

    fn write_pci(&self, requester: u32, offset: u32, value: u32) -> Result<(), DsError> {
        self
            .call(DsCmd::PciWrite, requester as u64, offset as u64, value as u64)
            .map(|_| ())
    }
}
