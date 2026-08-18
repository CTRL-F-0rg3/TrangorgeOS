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