use super::block::BlockDevice;
use super::lock::IrqGuard;

static mut DEVICES: [Option<&'static dyn BlockDevice>; 4] =
    [None, None, None, None];

pub fn register(dev: &'static dyn BlockDevice) -> bool {
    let _g = IrqGuard::lock();

    unsafe {
        for slot in DEVICES.iter_mut() {
            if slot.is_none() {
                *slot = Some(dev);
                return true;
            }
        }
    }

    false
}

pub fn get(index: usize) -> Option<&'static dyn BlockDevice> {
    let _g = IrqGuard::lock();

    unsafe { DEVICES.get(index).copied().flatten() }
}

pub fn first() -> Option<&'static dyn BlockDevice> {
    get(0)
}

pub fn count() -> usize {
    let _g = IrqGuard::lock();

    unsafe { DEVICES.iter().filter(|d| d.is_some()).count() }
}
