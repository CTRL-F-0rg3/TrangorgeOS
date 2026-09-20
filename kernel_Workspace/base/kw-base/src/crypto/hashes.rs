use crate::core::{KResult, KernelError};

extern "C" {
    fn k_sha256(data: *const u8, len: usize, out: *mut u8) -> bool;
    fn k_sha512(data: *const u8, len: usize, out: *mut u8) -> bool;
}

pub trait Hasher {
    const OUTPUT_LEN: usize;
    fn hash(data: &[u8], out: &mut [u8]) -> KResult<()>;
}

pub struct Sha256;
impl Hasher for Sha256 {
    const OUTPUT_LEN: usize = 32;
    fn hash(data: &[u8], out: &mut [u8]) -> KResult<()> {
        if out.len() < Self::OUTPUT_LEN { return Err(KernelError::InvalidArg); }
        if unsafe { k_sha256(data.as_ptr(), data.len(), out.as_mut_ptr()) } {
            Ok(())
        } else {
            Err(KernelError::IoError)
        }
    }
}

pub struct Sha512;
impl Hasher for Sha512 {
    const OUTPUT_LEN: usize = 64;
    fn hash(data: &[u8], out: &mut [u8]) -> KResult<()> {
        if out.len() < Self::OUTPUT_LEN { return Err(KernelError::InvalidArg); }
        if unsafe { k_sha512(data.as_ptr(), data.len(), out.as_mut_ptr()) } {
            Ok(())
        } else {
            Err(KernelError::IoError)
        }
    }
}