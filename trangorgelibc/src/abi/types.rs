
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SystemInfo {
    pub abi_major: u32,
    pub abi_minor: u32,
    pub total_memory: u64,
    pub free_memory: u64,
    pub cpu_count: u32,
    pub uptime_ticks: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handle(pub u64);

impl Handle {
    pub const INVALID: Handle = Handle(u64::MAX);

    pub fn is_valid(self) -> bool {
        self != Self::INVALID
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Stat {
    pub size: u64,
    pub is_dir: bool,
}