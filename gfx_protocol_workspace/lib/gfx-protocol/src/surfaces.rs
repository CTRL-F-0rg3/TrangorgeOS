#![no_std]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct SurfaceId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct BufferId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PixelFormat {
    XRGB8888 = 0,
    ARGB8888 = 1,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SurfaceConfig {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub flags: u32,
}

pub const SURFACE_FLAG_VISIBLE: u32 = 1 << 0;
pub const SURFACE_FLAG_FULLSCREEN: u32 = 1 << 1;