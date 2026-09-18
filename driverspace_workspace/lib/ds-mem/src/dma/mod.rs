//! DMA and MMIO memory management for drivers.

use kapi_abi::primitives::{Handle, PhysAddr, VirtAddr};
use kapi_abi::errors::DsError;
use kapi_abi::opcodes::Opcode;
use kapi_abi::payloads::mem::{MmioMapPayload, DmaAllocPayload};
use kapi_syscall::sys_ipc_call;
use ds_log::ds_error;
use bitflags::bitflags;

bitflags! {
    /// Flags for DMA buffer allocation
    pub struct DmaFlags: u32 {
        const COHERENT   = 1 << 0; // Hardware coherent (no explicit cache flush needed)
        const HIGH_MEM   = 1 << 1; // Allow allocation above 4GB (if device supports 64-bit DMA)
        const CONTIGUOUS = 1 << 2; // Physically contiguous memory
    }
}

/// Represents a mapped MMIO region for a device.
#[derive(Debug, Clone, Copy)]
pub struct MmioRegion {
    pub phys_base: PhysAddr,
    pub virt_base: VirtAddr,
    pub size: u64,
    pub handle: Handle,
}

impl MmioRegion {
    /// Requests the ds-manager to map a physical MMIO region into the driver's virtual address space.
    pub fn map(manager_ep: Handle, phys_base: u64, size: u64) -> Result<Self, DsError> {
        let mut payload = MmioMapPayload {
            phys_addr: PhysAddr(phys_base),
            size,
            flags: 0,
            _pad: 0,
        };

        let mut reply_buf = [0u64; 2];

        let res = unsafe {
            sys_ipc_call(
                manager_ep.0,
                Opcode::MemMapMmio as u32,
                &mut payload as *mut _ as *mut u8,
                core::mem::size_of::<MmioMapPayload>() as u32,
                reply_buf.as_mut_ptr() as *mut u8,
                core::mem::size_of_val(&reply_buf) as u32,
            )
        };

        match res {
            Ok(_) => {
                let virt_addr = VirtAddr(reply_buf[0]);
                let handle = Handle(reply_buf[1] as u32);
                Ok(Self {
                    phys_base: PhysAddr(phys_base),
                    virt_base: virt_addr,
                    size,
                    handle,
                })
            }
            Err(e) => {
                ds_error!("Failed to map MMIO region: {:?}", e);
                Err(e)
            }
        }
    }

    /// Safely reads a 32-bit value from the MMIO region at the given offset.
    #[inline(always)]
    pub unsafe fn read32(&self, offset: u64) -> u32 {
        debug_assert!(offset + 4 <= self.size, "MMIO read out of bounds");
        let ptr = (self.virt_base.0 + offset) as *const volatile::Volatile<u32>;
        (*ptr).read()
    }

    /// Safely writes a 32-bit value to the MMIO region at the given offset.
    #[inline(always)]
    pub unsafe fn write32(&self, offset: u64, value: u32) {
        debug_assert!(offset + 4 <= self.size, "MMIO write out of bounds");
        let ptr = (self.virt_base.0 + offset) as *mut volatile::Volatile<u32>;
        (*ptr).write(value);
    }
}

/// Represents a DMA-capable memory buffer.
pub struct DmaBuffer {
    pub phys_addr: PhysAddr,
    pub virt_addr: VirtAddr,
    pub size: u64,
    pub handle: Handle,
}

impl DmaBuffer {
    /// Requests a DMA buffer from the ds-manager.
    pub fn allocate(manager_ep: Handle, size: u64, flags: DmaFlags) -> Result<Self, DsError> {
        let mut payload = DmaAllocPayload {
            size,
            flags: flags.bits(),
            _pad: 0,
        };

        let mut reply_buf = [0u64; 3];

        let res = unsafe {
            sys_ipc_call(
                manager_ep.0,
                Opcode::MemAllocDma as u32,
                &mut payload as *mut _ as *mut u8,
                core::mem::size_of::<DmaAllocPayload>() as u32,
                reply_buf.as_mut_ptr() as *mut u8,
                core::mem::size_of_val(&reply_buf) as u32,
            )
        };

        match res {
            Ok(_) => {
                Ok(Self {
                    phys_addr: PhysAddr(reply_buf[0]),
                    virt_addr: VirtAddr(reply_buf[1]),
                    size,
                    handle: Handle(reply_buf[2] as u32),
                })
            }
            Err(e) => {
                ds_error!("Failed to allocate DMA buffer: {:?}", e);
                Err(e)
            }
        }
    }
}

impl Drop for DmaBuffer {
    fn drop(&mut self) {
        // TODO: Notify ds-manager to free the DMA buffer via IPC
        ds_log::ds_trace!("Dropping DMA buffer at {:?}", self.phys_addr);
    }
}