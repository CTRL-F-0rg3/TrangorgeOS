use crate::core::{KResult, KernelError};

extern "C" {
    fn k_get_random(buf: *mut u8, len: usize) -> bool;
    fn k_get_hw_random(buf: *mut u8, len: usize) -> bool;
}

pub fn fill_random(buf: &mut [u8]) -> KResult<()> {
    if unsafe { k_get_random(buf.as_mut_ptr(), buf.len()) } {
        Ok(())
    } else {
        Err(KernelError::IoError)
    }
}

pub fn fill_hw_random(buf: &mut [u8]) -> KResult<()> {
    if unsafe { k_get_hw_random(buf.as_mut_ptr(), buf.len()) } {
        Ok(())
    } else {
        Err(KernelError::NotSupported)
    }
}

pub fn next_u32() -> KResult<u32> {
    let mut val = 0u32;
    fill_random(unsafe { core::slice::from_raw_parts_mut(&mut val as *mut u32 as *mut u8, 4) })?;
    Ok(val)
}