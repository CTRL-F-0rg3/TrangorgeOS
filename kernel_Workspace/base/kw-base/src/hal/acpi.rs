use crate::core::{KResult, KernelError};

extern "C" {
    fn k_acpi_get_rsdp() -> *const u8;
    fn k_acpi_find_table(sig: *const u8, idx: u32) -> *const u8;
}

pub fn get_rsdp() -> Option<*const u8> {
    let ptr = unsafe { k_acpi_get_rsdp() };
    if ptr.is_null() { None } else { Some(ptr) }
}

pub fn find_table(sig: &[u8; 4], index: u32) -> Option<*const u8> {
    let ptr = unsafe { k_acpi_find_table(sig.as_ptr(), index) };
    if ptr.is_null() { None } else { Some(ptr) }
}