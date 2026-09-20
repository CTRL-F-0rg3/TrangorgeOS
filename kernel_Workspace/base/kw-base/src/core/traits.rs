use super::error::KernelError;
use super::types::*;

pub trait BlockDevice {
    fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), KernelError>;
    fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), KernelError>;
    fn block_size(&self) -> usize;
    fn block_count(&self) -> u64;
}

pub trait CharDevice {
    fn read(&self, buf: &mut [u8]) -> Result<usize, KernelError>;
    fn write(&self, buf: &[u8]) -> Result<usize, KernelError>;
}

pub trait NetworkDevice {
    fn send(&self, frame: &[u8]) -> Result<(), KernelError>;
    fn recv(&self, buf: &mut [u8]) -> Result<usize, KernelError>;
    fn mac_address(&self) -> [u8; 6];
}

pub trait FileSystem {
    fn open(&self, path: &str, flags: u32) -> Result<Handle, KernelError>;
    fn close(&self, handle: Handle) -> Result<(), KernelError>;
    fn read(&self, handle: Handle, buf: &mut [u8], offset: u64) -> Result<usize, KernelError>;
    fn write(&self, handle: Handle, buf: &[u8], offset: u64) -> Result<usize, KernelError>;
    fn stat(&self, path: &str) -> Result<FileStat, KernelError>;
}

#[derive(Debug, Clone, Copy)]
pub struct FileStat {
    pub size: u64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub is_dir: bool,
}

pub trait Scheduler {
    fn schedule(&mut self) -> Option<TaskId>;
    fn enqueue(&mut self, task: TaskId);
    fn dequeue(&mut self, task: TaskId);
    fn yield_current(&mut self);
}

pub trait InterruptController {
    fn enable_irq(&self, irq: u32);
    fn disable_irq(&self, irq: u32);
    fn ack_irq(&self, irq: u32);
    fn end_of_interrupt(&self);
}