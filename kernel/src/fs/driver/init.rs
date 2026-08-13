use crate::fs::driver::driver::{REGISTRY, RamDisk};
use alloc::boxed::Box;

pub fn init() {
    let ramdisk = RamDisk::new(1024 * 1024, 512);
    REGISTRY.lock().register(Box::new(ramdisk));
}
