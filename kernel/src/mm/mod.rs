pub mod api;
pub mod ffi;
pub mod phys;
pub mod space;
pub mod virt;

pub fn init(params: &ffi::MmBootParams) -> bool {
    unsafe { ffi::mm_init(params as *const ffi::MmBootParams) }
}

pub fn ready() -> bool {
    unsafe { ffi::mm_ready() }
}

pub fn dump() {
    unsafe { ffi::mm_dump() }
}