// kernel_Workspace/core/gluecore/src/bridge/caps/check.rs

use kstd_base::{Status, CapId};
use crate::ffi::c::extern_block;

pub fn has_cap(world_id: u32, cap: CapId) -> bool {
    unsafe { extern_block::caps_has(world_id, cap.0 as u8) == 1 }
}

pub fn request_cap(target_world: u32, cap: CapId) -> Result<(), Status> {
    let rc = unsafe { extern_block::caps_request(target_world, cap.0 as u8) };
    if rc == 0 {
        Ok(())
    } else {
        Err(Status::PermissionDenied)
    }
}

pub fn release_cap(world_id: u32, cap: CapId) -> Result<(), Status> {
    let rc = unsafe { extern_block::caps_release(world_id, cap.0 as u8) };
    if rc == 0 {
        Ok(())
    } else {
        Err(Status::InvalidArgument)
    }
}