//! User space (ring 3) <-> kernel communication over `tg_comm` shared memory.
//!
//! Mirrors the driver-space bridge, but for `Layer::Userspace` and with the
//! rings mapped into a ring-3 address space (PROT_USER).

use crate::mm::{phys, space};
use tg_comm::{CapEntry, CapId, CapTable, Layer, ObjectType, Rights};

use super::bridge::LayerBridge;
use super::dispatch;

const DIRECT_BASE: u64 = 0xFFFF888000000000;

/// User-space virtual addresses for the shared rings.
pub const US_K2U_VA: u64 = 0x7000_0000;
pub const US_U2K_VA: u64 = 0x7000_1000;
pub const US_RING_SLOTS: u32 = 256;

/// Capability handles handed to the userspace init process.
pub const US_CAP_DEVICE: u32 = 1; // CALL over a Device
pub const US_CAP_MEMORY: u32 = 2; // CALL|READ|WRITE over Memory

fn kv(phys: u64) -> *mut u8 {
    (DIRECT_BASE + phys) as *mut u8
}

struct UserBridge {
    bridge: LayerBridge,
    k2u_phys: u64,
    u2k_phys: u64,
}

static mut BRIDGE: Option<UserBridge> = None;

/// Allocate the shared rings and bootstrap the userspace capability table.
pub fn prepare() -> bool {
    let bytes = tg_comm::RING_CONTROL_SIZE + US_RING_SLOTS as usize * tg_comm::COMM_MSG_SIZE;
    let pages = (bytes + 4095) / 4096;

    let k2u_phys = match phys::alloc_frames(pages) {
        Some(p) => p,
        None => return false,
    };
    let u2k_phys = match phys::alloc_frames(pages) {
        Some(p) => p,
        None => return false,
    };

    let mut table = CapTable::new();
    table.insert_at(CapId(US_CAP_DEVICE), CapEntry::new(0, ObjectType::Device, Rights::CALL));
    table.insert_at(
        CapId(US_CAP_MEMORY),
        CapEntry::new(
            0,
            ObjectType::Memory,
            Rights::CALL.union(Rights::READ).union(Rights::WRITE),
        ),
    );

    let bridge = unsafe {
        LayerBridge::new(
            Layer::Userspace,
            kv(k2u_phys), US_RING_SLOTS, // tx: kernel -> userspace
            kv(u2k_phys), US_RING_SLOTS, // rx: userspace -> kernel
            table,
        )
    };

    unsafe {
        BRIDGE = Some(UserBridge {
            bridge,
            k2u_phys,
            u2k_phys,
        });
    }
    true
}

/// Map the shared rings into a ring-3 address space. Called at process spawn
/// time with the target address space. Returns `false` on mapping failure.
pub fn map_rings_into(aspace: &space::AddressSpace) -> bool {
    let (k2u_phys, u2k_phys, pages) = unsafe {
        match BRIDGE.as_ref() {
            Some(b) => {
                let bytes = tg_comm::RING_CONTROL_SIZE
                    + US_RING_SLOTS as usize * tg_comm::COMM_MSG_SIZE;
                (b.k2u_phys, b.u2k_phys, (bytes + 4095) / 4096)
            }
            None => return false,
        }
    };

    let prot = space::PROT_READ | space::PROT_WRITE | space::PROT_USER;

    aspace.map_phys(US_K2U_VA, k2u_phys, pages * 4096, prot)
        && aspace.map_phys(US_U2K_VA, u2k_phys, pages * 4096, prot)
}

/// Drain and dispatch authorized userspace requests.
pub fn poll() {
    unsafe {
        if let Some(b) = BRIDGE.as_ref() {
            b.bridge.poll(dispatch::handle);
        }
    }
}

/// Kernel-side polling hook callable from the ring-3 hypercall path.
#[no_mangle]
pub extern "C" fn k_shm_poll() {
    poll();
}
