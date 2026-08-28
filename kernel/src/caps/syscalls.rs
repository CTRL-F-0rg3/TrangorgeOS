//! Syscalls capabilities: SYS_CAP_QUERY / REQUEST / RELEASE / AUDIT.

use super::types::Capability;
use super::store;
use super::check;
use super::grant;
use super::revoke;
use super::audit;

pub const SYS_CAP_QUERY: u64 = 0x1070;
pub const SYS_CAP_REQUEST: u64 = 0x1071;
pub const SYS_CAP_RELEASE: u64 = 0x1072;
pub const SYS_CAP_AUDIT: u64 = 0x1073;

fn cap_from_id(id: u8) -> Option<Capability> {
    Capability::iter_all().find(|c| c.id() == id)
}

/// Dispatch syscall capabilities
pub fn cap_syscall(num: u64, a0: u64, a1: u64, a2: u64) -> u64 {
    let wid = check::current_world_id_pub();

    match num {
        SYS_CAP_QUERY => {
            // a0 = cap_id; zwraca 1/0 dla current world
            match cap_from_id(a0 as u8) {
                Some(cap) => store::world_has_cap(wid, cap) as u64,
                None => 0,
            }
        }

        SYS_CAP_REQUEST => {
            // a0 = cap_id; wymaga Admin
            if !store::world_has_cap(wid, Capability::Admin) {
                return u64::MAX;
            }
            match cap_from_id(a0 as u8) {
                Some(cap) => match grant::grant_cap(wid, wid, cap) {
                    Ok(()) => 0,
                    Err(_) => u64::MAX,
                },
                None => u64::MAX,
            }
        }

        SYS_CAP_RELEASE => {
            // a0 = cap_id
            match cap_from_id(a0 as u8) {
                Some(cap) => match revoke::revoke_from_world(wid, cap) {
                    Ok(()) => 0,
                    Err(_) => u64::MAX,
                },
                None => u64::MAX,
            }
        }

        SYS_CAP_AUDIT => {
            // a0 = limit; zwraca deny_count (do telemetry)
            audit::deny_count() as u64
        }

        _ => u64::MAX,
    }
}