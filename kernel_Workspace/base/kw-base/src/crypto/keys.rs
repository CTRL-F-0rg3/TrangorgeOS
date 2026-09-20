use super::secure_mem::SecureBuffer;
use crate::core::{KResult, KernelError};

pub struct SymmetricKey {
    inner: SecureBuffer,
}

impl SymmetricKey {
    pub fn generate(len: usize) -> KResult<Self> {
        let mut buf = SecureBuffer::new(len).ok_or(KernelError::NoMemory)?;
        super::random::fill_random(buf.as_mut_slice())?;
        Ok(Self { inner: buf })
    }

    pub fn from_bytes(bytes: &[u8]) -> KResult<Self> {
        let mut buf = SecureBuffer::new(bytes.len()).ok_or(KernelError::NoMemory)?;
        buf.as_mut_slice().copy_from_slice(bytes);
        Ok(Self { inner: buf })
    }

    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }
}