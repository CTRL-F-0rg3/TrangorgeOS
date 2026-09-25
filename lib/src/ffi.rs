//! Stable C ABI: the boundary every other language (C, Odin, Ada) calls.
//!
//! Consumers build [`CommMsg`] values (the protocol is public), but the
//! capability table, rings and the authorization gate stay opaque — they are
//! always manipulated through these functions, never re-implemented.

use crate::caps::{CapEntry, CapId, CapTable, ObjectType};
use crate::consts::{COMM_MSG_SIZE, COMM_VERSION, RING_CONTROL_SIZE};
use crate::filter::authorize;
use crate::rights::Rights;
use crate::ring::SpscRing;
use crate::wire::CommMsg;

/// Total byte size of a ring with `slots` message slots.
#[no_mangle]
pub extern "C" fn tgcomm_ring_size(slots: u32) -> usize {
    RING_CONTROL_SIZE + slots as usize * COMM_MSG_SIZE
}

/// Wire protocol version.
#[no_mangle]
pub extern "C" fn tgcomm_version() -> u32 {
    COMM_VERSION as u32
}

/// Size in bytes of one [`CommMsg`].
#[no_mangle]
pub extern "C" fn tgcomm_msg_size() -> u32 {
    COMM_MSG_SIZE as u32
}

/// Size in bytes of one [`CapTable`].
#[no_mangle]
pub extern "C" fn tgcomm_cap_table_size() -> usize {
    core::mem::size_of::<CapTable>()
}

/// Initialize a message in place (magic + version, all else zero).
///
/// # Safety
/// `msg` must point to a writable 64-byte `CommMsg`.
#[no_mangle]
pub unsafe extern "C" fn tgcomm_msg_init(msg: *mut CommMsg) {
    if let Some(m) = msg.as_mut() {
        *m = CommMsg::default();
    }
}

/// Non-zero when the message is structurally well-formed.
///
/// # Safety
/// `msg` must point to a readable 64-byte `CommMsg`.
#[no_mangle]
pub unsafe extern "C" fn tgcomm_msg_valid(msg: *const CommMsg) -> i32 {
    match msg.as_ref() {
        Some(m) => i32::from(m.well_formed()),
        None => 0,
    }
}

/// Authorize a message against a capability table.
/// Returns an `AuthorizeResult` discriminant (`0` == Forwarded).
///
/// # Safety
/// `table`/`msg` must be valid pointers owned by the caller.
#[no_mangle]
pub unsafe extern "C" fn tgcomm_authorize(
    table: *const CapTable,
    msg: *const CommMsg,
) -> i32 {
    match (table.as_ref(), msg.as_ref()) {
        (Some(t), Some(m)) => authorize(t, m) as i32,
        _ => 1, // Malformed
    }
}

/// Initialize a ring in place over `slots` slots.
///
/// # Safety
/// `ring` must point to `tgcomm_ring_size(slots)` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn tgcomm_ring_init(ring: *mut u8, slots: u32) {
    if !ring.is_null() {
        let _ = SpscRing::from_raw(ring, slots);
    }
}

/// Push `msg` into `ring`. Returns `0` on success, non-zero on failure (full).
///
/// # Safety
/// `ring` must be an initialized ring; `msg` a valid `CommMsg`.
#[no_mangle]
pub unsafe extern "C" fn tgcomm_ring_push(ring: *mut u8, msg: *const CommMsg) -> i32 {
    if ring.is_null() || msg.is_null() {
        return 1;
    }
    let r = SpscRing::attach(ring);
    if r.push(&*msg) {
        0
    } else {
        1
    }
}

/// Pop the next message from `ring` into `out`. Returns `0` on success.
///
/// # Safety
/// `ring` must be an initialized ring; `out` a writable `CommMsg`.
#[no_mangle]
pub unsafe extern "C" fn tgcomm_ring_pop(ring: *mut u8, out: *mut CommMsg) -> i32 {
    if ring.is_null() || out.is_null() {
        return 1;
    }
    let r = SpscRing::attach(ring);
    match r.pop() {
        Some(m) => {
            *out = m;
            0
        }
        None => 1,
    }
}

/// Number of pending messages in `ring`.
///
/// # Safety
/// `ring` must be an initialized ring.
#[no_mangle]
pub unsafe extern "C" fn tgcomm_ring_available(ring: *const u8) -> u32 {
    if ring.is_null() {
        return 0;
    }
    let r = SpscRing::attach(ring as *mut u8);
    r.available()
}

/// Initialize a capability table in place.
///
/// # Safety
/// `table` must point to `tgcomm_cap_table_size()` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn tgcomm_cap_table_init(table: *mut CapTable) {
    if let Some(t) = table.as_mut() {
        t.init();
    }
}

/// Insert a capability entry at handle `cap`.
///
/// # Safety
/// `table` must be an initialized capability table.
#[no_mangle]
pub unsafe extern "C" fn tgcomm_cap_insert(
    table: *mut CapTable,
    cap: u32,
    obj_id: u64,
    obj_type: u8,
    rights: u32,
) -> i32 {
    let Some(ty) = ObjectType::from_u8(obj_type) else {
        return 1;
    };
    match table.as_mut() {
        Some(t) => {
            if t.insert_at(CapId(cap), CapEntry::new(obj_id, ty, Rights(rights))) {
                0
            } else {
                1
            }
        }
        None => 1,
    }
}

/// Non-zero when `cap` holds at least `required_rights`.
///
/// # Safety
/// `table` must be an initialized capability table.
#[no_mangle]
pub unsafe extern "C" fn tgcomm_cap_check(
    table: *const CapTable,
    cap: u32,
    required_rights: u32,
) -> i32 {
    match table.as_ref() {
        Some(t) => i32::from(t.check_rights(CapId(cap), Rights(required_rights))),
        None => 0,
    }
}

/// Remove a capability entry. Returns non-zero when it existed.
///
/// # Safety
/// `table` must be an initialized capability table.
#[no_mangle]
pub unsafe extern "C" fn tgcomm_cap_remove(table: *mut CapTable, cap: u32) -> i32 {
    match table.as_mut() {
        Some(t) => i32::from(t.remove(CapId(cap)).is_some()),
        None => 0,
    }
}
