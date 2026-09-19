use crate::core::{KResult, KernelError};

extern "C" {
    fn k_aes256_enc(key: *const u8, iv: *const u8, data: *mut u8, len: usize) -> bool;
    fn k_aes256_dec(key: *const u8, iv: *const u8, data: *mut u8, len: usize) -> bool;
}

pub trait BlockCipher {
    const KEY_LEN: usize;
    const IV_LEN: usize;
    const BLOCK_LEN: usize;

    fn encrypt(key: &[u8], iv: &[u8], data: &mut [u8]) -> KResult<()>;
    fn decrypt(key: &[u8], iv: &[u8], data: &mut [u8]) -> KResult<()>;
}

pub struct Aes256Cbc;
impl BlockCipher for Aes256Cbc {
    const KEY_LEN: usize = 32;
    const IV_LEN: usize = 16;
    const BLOCK_LEN: usize = 16;

    fn encrypt(key: &[u8], iv: &[u8], data: &mut [u8]) -> KResult<()> {
        if key.len() != Self::KEY_LEN || iv.len() != Self::IV_LEN {
            return Err(KernelError::InvalidArg);
        }
        if data.len() % Self::BLOCK_LEN != 0 {
            return Err(KernelError::InvalidArg);
        }
        if unsafe { k_aes256_enc(key.as_ptr(), iv.as_ptr(), data.as_mut_ptr(), data.len()) } {
            Ok(())
        } else {
            Err(KernelError::IoError)
        }
    }

    fn decrypt(key: &[u8], iv: &[u8], data: &mut [u8]) -> KResult<()> {
        if key.len() != Self::KEY_LEN || iv.len() != Self::IV_LEN {
            return Err(KernelError::InvalidArg);
        }
        if data.len() % Self::BLOCK_LEN != 0 {
            return Err(KernelError::InvalidArg);
        }
        if unsafe { k_aes256_dec(key.as_ptr(), iv.as_ptr(), data.as_mut_ptr(), data.len()) } {
            Ok(())
        } else {
            Err(KernelError::IoError)
        }
    }
}