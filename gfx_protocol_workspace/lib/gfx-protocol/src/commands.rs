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

impl GfxCmd {
    /// Decode a command number, or `None` for one that is not a Gfx command.
    ///
    /// The wire carries a plain `u32`, so every value decodes to *something*;
    /// returning `None` for an unassigned one is what lets a server answer
    /// "unknown command" rather than falling through into a valid arm.
    #[inline]
    pub const fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x01 => Some(Self::CreateSurface),
            0x02 => Some(Self::DestroySurface),
            0x03 => Some(Self::ResizeSurface),
            0x10 => Some(Self::AllocBuffer),
            0x11 => Some(Self::AttachBuffer),
            0x12 => Some(Self::Commit),
            0x20 => Some(Self::SetPosition),
            0x21 => Some(Self::SetZOrder),
            0x80 => Some(Self::GpuSubmit),
            _ => None,
        }
    }
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