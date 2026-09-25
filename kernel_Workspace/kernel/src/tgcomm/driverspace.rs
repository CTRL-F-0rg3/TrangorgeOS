//! Driver space <-> kernel communication over `tg_comm` shared memory.

use crate::mm::{phys, space};
use tg_comm::{CapEntry, CapId, CapTable, Layer, ObjectType, Rights};

use super::bridge::LayerBridge;
use super::dispatch;

const DIRECT_BASE: u64 = 0xFFFF888000000000;

/// Driver-space virtual addresses for the shared rings.
pub const DS_K2D_VA: u64 = 0x4000_1000;
pub const DS_D2K_VA: u64 = 0x4000_2000;
pub const DS_RING_SLOTS: u32 = 256;

/// Capability handles handed to the driver-space manager.
pub const DS_CAP_DEVICE: u32 = 1; // CALL over a Device
pub const DS_CAP_MEMORY: u32 = 2; // CALL|MAP|WRITE|MANAGE over Memory

fn kv(phys: u64) -> *mut u8 {
    (DIRECT_BASE + phys) as *mut u8
}

static mut BRIDGE: Option<LayerBridge> = None;

fn device_cap() -> CapEntry {
    CapEntry::new(0, ObjectType::Device, Rights::CALL)
}

fn memory_cap() -> CapEntry {
    CapEntry::new(
        0,
        ObjectType::Memory,
        Rights::CALL.union(Rights::MAP).union(Rights::WRITE).union(Rights::MANAGE),
    )
}

/// Allocate and map the `tg_comm` rings, bootstrap the capability table and
/// attach the kernel side of the bridge. Reuses the legacy driver-space
/// address space (`init::prepare`) so `service::handle` keeps working.
pub fn prepare() -> bool {
    if crate::driverspaceinit::init::init::prepare().is_err() {
        return false;
    }

    let bytes = tg_comm::RING_CONTROL_SIZE + DS_RING_SLOTS as usize * tg_comm::COMM_MSG_SIZE;
    let pages = (bytes + 4095) / 4096;

    let k2d_phys = match phys::alloc_frames(pages) {
        Some(p) => p,
        None => return false,
    };
    let d2k_phys = match phys::alloc_frames(pages) {
        Some(p) => p,
        None => return false,
    };

    let prot = space::PROT_READ | space::PROT_WRITE | space::PROT_USER;

    if !crate::driverspaceinit::init::init::map_into_ds(DS_K2D_VA, k2d_phys, pages * 4096, prot) {
        return false;
    }
    if !crate::driverspaceinit::init::init::map_into_ds(DS_D2K_VA, d2k_phys, pages * 4096, prot) {
        return false;
    }

    let mut table = CapTable::new();
    table.insert_at(CapId(DS_CAP_DEVICE), device_cap());
    table.insert_at(CapId(DS_CAP_MEMORY), memory_cap());

    let bridge = unsafe {
        LayerBridge::new(
            Layer::Driverspace,
            kv(k2d_phys), DS_RING_SLOTS, // tx: kernel -> driverspace
            kv(d2k_phys), DS_RING_SLOTS, // rx: driverspace -> kernel
            table,
        )
    };

    unsafe { BRIDGE = Some(bridge); }
    true
}

/// Drain and dispatch authorized driver-space requests.
pub fn poll() {
    unsafe {
        if let Some(b) = BRIDGE.as_ref() {
            b.poll(dispatch::handle);
        }
    }
}

/// Post an asynchronous event to the driver space.
pub fn post_event(opcode: u32, a0: u64, a1: u64) {
    unsafe {
        if let Some(b) = BRIDGE.as_ref() {
            b.post_event(opcode, a0, a1);
        }
    }
}
