pub const DS_INIT_PARAMS_VA: u64 = 0x4000_0000;
pub const DS_K2D_VA: u64 = 0x4000_1000;
pub const DS_D2K_VA: u64 = 0x4000_2000;
pub const DS_SCRATCH_VA: u64 = 0x4000_3000;
pub const DS_SCRATCH_SIZE: usize = 4096;
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DsInitParams {
    pub magic: u64,
    pub version: u32,
    pub pad: u32,
    pub k2d_va: u64,
    pub d2k_va: u64,
    pub ring_cap: u64,
    pub ds_va_base: u64,
    pub ds_va_size: u64,
}