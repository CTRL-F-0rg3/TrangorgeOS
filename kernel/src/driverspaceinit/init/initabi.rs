pub const DS_SWITCH_VA: u64 = 0x4000_4000;
pub const DS_STACK_VA: u64 = 0x4FFF_0000;
pub const DS_STACK_SIZE: u64 = 16 * 1024;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DsSwitch {
    pub kernel_cr3: u64,
    pub ds_cr3: u64,
    pub kernel_resume: u64,
    pub kernel_stack: u64,
}

// Service classes and ops for the driverspace ABI. Values mirror
// driverspacelib/src/abi.rs so the kernel-side dispatch uses the same numbers.
pub const SVC_SYS: u32 = 0;
pub const SVC_VIDEO: u32 = 1;
pub const SVC_AUDIO: u32 = 2;
pub const SVC_INPUT: u32 = 3;
pub const SVC_BLOCK: u32 = 4;
pub const SVC_NET: u32 = 5;

// VIDEO ops
pub const VID_FB_INFO: u32 = 1;
pub const VID_FB_TAKEOVER: u32 = 2;
pub const VID_FB_RELEASE: u32 = 3;

// INPUT ops
pub const IN_KEY_POLL: u32 = 1;

// AUDIO ops
pub const AUD_PLAY: u32 = 1;
pub const AUD_STOP: u32 = 2;
pub const AUD_JACK: u32 = 3;
pub const AUD_AMP: u32 = 4;

// BLOCK ops
pub const BLK_COUNT: u32 = 1;
pub const BLK_READ: u32 = 2;
pub const BLK_WRITE: u32 = 3;

// Virtual addresses handed to the driverspace (mirrors libs/dsabi.h).
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