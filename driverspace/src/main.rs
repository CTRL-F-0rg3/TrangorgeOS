#![no_std]
#![no_main]

use driverspacelib as ds;
use ds::driver::{Driver, DeviceInfo};

mod drivers;

static mut STORAGE: drivers::storage::StorageDrv = drivers::storage::StorageDrv::new();
static mut REGISTERED: bool = false;

#[link_section = ".text.ds_entry"]
#[no_mangle]
pub extern "C" fn ds_entry(params_va: u64) {
    unsafe {
        ds::init_once(params_va);

        if !REGISTERED {
            ds::register(&mut STORAGE);
            REGISTERED = true;
        }
    }

    ds::tick();
    ds::yield_to_kernel();
}

fn call(cmd: ds::abi::DsCmd, arg0: u64, arg1: u64, arg2: u64) -> Result<ds::abi::DsMsg, ds::abi::DsError> {
    let mut resp = ds::request(cmd, arg0, arg1, arg2)?;
    let status = resp.status;
    if status != 0 {
        return Err(ds::abi::DsError::from(status));
    }
    Ok(resp)
}

fn _init_drivers() -> Result<(), ds::abi::DsError> {
    let count = call(ds::abi::DsCmd::GetDeviceCount, 0, 0, 0)?.arg0;
    for i in 0..count {
        let info = call(ds::abi::DsCmd::GetDeviceInfo, i, 0, 0)?;
        let dev_info = DeviceInfo {
            device_id: info.arg0,
            vendor_id: info.arg1,
            class_code: info.arg2,
        };
        drivers::init_driver(dev_info)?;
    }
    Ok(())
}


#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}