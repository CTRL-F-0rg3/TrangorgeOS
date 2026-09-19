use crate::arch::registers::CpuContext;

pub type TaskContext = CpuContext;

extern "C" {
    pub fn context_switch(old_ctx: *mut TaskContext, new_ctx: *mut TaskContext);
}

pub unsafe fn switch(old: &mut TaskContext, new: &TaskContext) {
    context_switch(old, new as *const _ as *mut _);
}