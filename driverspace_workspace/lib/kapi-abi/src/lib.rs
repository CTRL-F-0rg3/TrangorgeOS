#![no_std]

// 导出所有核心模块
pub mod primitives;
pub mod errors;
pub mod opcodes;
pub mod payloads;

// 为了方便使用，将最常用的类型 re-export 到根命名空间
pub use primitives::{Handle, CapId, PhysAddr, VirtAddr};
pub use errors::DsError;
pub use opcodes::Opcode;