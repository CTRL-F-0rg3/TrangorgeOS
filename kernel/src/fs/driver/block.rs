#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    Timeout,
    Io,
    Unsupported,
    InvalidBlock,
    InvalidLength,
    NoDevice,
}

pub trait BlockDevice {
    fn name(&self) -> &'static str;
    fn block_size(&self) -> usize;
    fn block_count(&self) -> u64;

    fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError>;
    fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError>;

    fn read_blocks(&self, start: u64, buf: &mut [u8]) -> Result<(), DriverError> {
        let bs = self.block_size();

        if buf.len() % bs != 0 {
            return Err(DriverError::InvalidLength);
        }

        let n = buf.len() / bs;

        for i in 0..n {
            let off = i * bs;
            self.read_block(start + i as u64, &mut buf[off..off + bs])?;
        }

        Ok(())
    }

    fn write_blocks(&self, start: u64, buf: &[u8]) -> Result<(), DriverError> {
        let bs = self.block_size();

        if buf.len() % bs != 0 {
            return Err(DriverError::InvalidLength);
        }

        let n = buf.len() / bs;

        for i in 0..n {
            let off = i * bs;
            self.write_block(start + i as u64, &buf[off..off + bs])?;
        }

        Ok(())
    }
}
