#[no_std]
// kernel_Workspace/core/gluecore/src/bridge/mm/phys.rs

use kstd_base::errors::CoreError;
use kstd_base::types::PhysAddr;

// Importujemy "stare" jądro jako bibliotekę silnika
use kernel::mm::phys as kernel_phys;

/// Bezpieczny wrapper nad fizycznym alokatorem ramek z jądra.
pub mod pmm {
    use super::*;

    /// Bezpieczna alokacja jednej ramki fizycznej (4KB).
    /// Tłumaczy `Option<u64>` z jądra na `Result<PhysAddr, CoreError>`.
    pub fn alloc_frame() -> Result<PhysAddr, CoreError> {
        kernel_phys::alloc_frame()
            .map(PhysAddr)
            .ok_or(CoreError::OutOfMemory)
    }

    /// Zwolnienie ramki.
    pub fn free_frame(addr: PhysAddr) -> Result<(), CoreError> {
        if kernel_phys::free_frame(addr.0) {
            Ok(())
        } else {
            Err(CoreError::InvalidAddress)
        }
    }
}