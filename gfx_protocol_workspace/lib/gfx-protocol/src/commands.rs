#![no_std]

use super::surfaces::{SurfaceId, BufferId, SurfaceConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GfxCmd {
    CreateSurface = 0x01,
    DestroySurface = 0x02,
    ResizeSurface = 0x03,
    
    AllocBuffer = 0x10,
    AttachBuffer = 0x11,
    Commit = 0x12,
    
    SetPosition = 0x20,
    SetZOrder = 0x21,
    
    GpuSubmit = 0x80,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GfxIpcMsg {
    pub cmd: u32,
    pub status: i32,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
}