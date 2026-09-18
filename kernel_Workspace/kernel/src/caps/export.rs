use super::types::Capability;
use super::store;
use super::check;
use super::grant;
use super::revoke;

fn cap_from_id(id: u8) -> Option<Capability> {
    Capability::iter_all().find(|c| c.id() == id)
}

#[no_mangle]
pub extern "C" fn caps_self_bits() -> u32 {
    let wid = check::current_world_id_pub();
    store::get_world_caps(wid).map(|c| c.bits()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn caps_world_bits(world_id: u32) -> u32 {
    store::get_world_caps(world_id).map(|c| c.bits()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn caps_has(world_id: u32, cap_id: u8) -> i32 {
    match cap_from_id(cap_id) {
        Some(cap) => store::world_has_cap(world_id, cap) as i32,
        None => 0,
    }
}


#[no_mangle]
pub extern "C" fn caps_name(cap_id: u8, buf: *mut u8, len: u32) -> i32 {
    let cap = match cap_from_id(cap_id) {
        Some(c) => c,
        None => return -1,
    };

    let name = cap.name();
    let n = name.len().min(len as usize);

    unsafe {
        core::ptr::copy_nonoverlapping(name.as_ptr(), buf, n);
    }

    n as i32
}

#[no_mangle]
pub extern "C" fn caps_request(target: u32, cap_id: u8) -> i32 {
    let cap = match cap_from_id(cap_id) {
        Some(c) => c,
        None => return -1,
    };

    match grant::grant_cap(check::kernel_world_id(), target, cap) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn caps_release(world_id: u32, cap_id: u8) -> i32 {
    let cap = match cap_from_id(cap_id) {
        Some(c) => c,
        None => return -1,
    };

    match revoke::revoke_from_world(world_id, cap) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn caps_world_count() -> u32 {
    store::world_count() as u32
}

#[no_mangle]
pub extern "C" fn caps_audit_count() -> u64 {
    super::audit::count() as u64
}