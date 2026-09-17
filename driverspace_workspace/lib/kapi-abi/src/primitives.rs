//! 基础内核类型定义。
//! 所有类型均使用固定宽度整数，确保在 Rust, C, Odin 中内存布局完全一致。

/// 内核对象句柄（如 IPC Channel, Memory Region, IRQ 等）。
/// 0 通常保留为 NULL_HANDLE。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Handle(pub u32);

impl Handle {
    pub const NULL: Self = Self(0);
    
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// 能力标识符（Capability ID）。
/// 用于基于能力的安全模型，标识对特定资源的访问权限。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CapId(pub u64);

/// 物理地址类型（裸机环境下的物理内存地址）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

/// 虚拟地址类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);