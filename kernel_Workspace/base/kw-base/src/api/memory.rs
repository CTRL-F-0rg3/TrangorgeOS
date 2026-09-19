use crate::core::{KResult, KernelError};
use gluecore::bridge::mm::vmm;

pub const PROT_READ: u32 = 1;
pub const PROT_WRITE: u32 = 2;
pub const PROT_EXEC: u32 = 4;

pub fn mmap(addr: u64, len: usize, prot: u32, _flags: u32) -> KResult<u64> {
    if len == 0 { return Err(KernelError::InvalidArg); }
    
    let mut vmm_flags = 0;
    if prot & PROT_READ != 0 { vmm_flags |= vmm::VMM_FLAG_READ; }
    if prot & PROT_WRITE != 0 { vmm_flags |= vmm::VMM_FLAG_WRITE; }
    if prot & PROT_EXEC != 0 { vmm_flags |= vmm::VMM_FLAG_EXEC; }

    let virt = vmm::alloc(len, vmm_flags).map_err(KernelError::from)?;
    Ok(virt.0)
}

pub fn munmap(addr: u64, len: usize) -> KResult<()> {
    if len == 0 { return Err(KernelError::InvalidArg); }
    vmm::free(kstd_base::VirtAddr(addr), len).map_err(KernelError::from)
}