//! Audit trail: kto, kiedy, jaką capability sprawdził/dostał/stracił.

use super::types::Capability;
use spin::Mutex;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    CheckOk,
    CheckDeny,
    Grant,
    Revoke,
    Register,
    Unregister,
}

#[derive(Debug, Clone, Copy)]
pub struct AuditEvent {
    pub seq: u64,
    pub tick: u64,
    pub world_id: u32,
    pub target_world: u32,
    pub cap: Capability,
    pub kind: EventKind,
}

const AUDIT_CAP: usize = 2048;

struct AuditInner {
    buf: [Option<AuditEvent>; AUDIT_CAP],
    head: usize,
    seq: u64,
    total: u64,
}

static AUDIT: Mutex<AuditInner> = Mutex::new(AuditInner {
    buf: [None; AUDIT_CAP],
    head: 0,
    seq: 0,
    total: 0,
});

pub fn init_audit_log() -> Result<(), &'static str> {
    let mut a = AUDIT.lock();
    a.head = 0;
    a.seq = 0;
    a.total = 0;
    for i in 0..AUDIT_CAP {
        a.buf[i] = None;
    }
    Ok(())
}

fn now_tick() -> u64 {
    // Zegar arch-poziomu: TSC (x86_64) / CLINT mtime (RISC-V).
    crate::arch::now()
}

fn push(kind: EventKind, world: u32, target: u32, cap: Capability) {
    let mut a = AUDIT.lock();
    let idx = a.head as usize % AUDIT_CAP;
    let ev = AuditEvent {
        seq: a.seq,
        tick: now_tick(),
        world_id: world,
        target_world: target,
        cap,
        kind,
    };
    a.buf[idx] = Some(ev);
    a.head = (idx + 1) % AUDIT_CAP;
    a.seq += 1;
    a.total += 1;
}

pub fn log_check(world: u32, cap: Capability, ok: bool) {
    push(if ok { EventKind::CheckOk } else { EventKind::CheckDeny },
         world, world, cap);
}

pub fn log_grant(granter: u32, target: u32, cap: Capability, ok: bool) {
    if ok {
        push(EventKind::Grant, granter, target, cap);
    }
}

pub fn log_revoke(world: u32, cap: Capability, ok: bool) {
    if ok {
        push(EventKind::Revoke, world, world, cap);
    }
}

pub fn log_register(world: u32) {
    push(EventKind::Register, world, world, Capability::User);
}

pub fn log_unregister(world: u32) {
    push(EventKind::Unregister, world, world, Capability::User);
}

/// Łączna liczba zdarzeń
pub fn count() -> usize {
    AUDIT.lock().total as usize
}

/// Ostatnie N zdarzeń (od najnowszego)
pub fn recent(n: usize) -> Vec<AuditEvent> {
    let a = AUDIT.lock();
    let mut out = Vec::new();

    let mut idx = if a.head == 0 { AUDIT_CAP - 1 } else { a.head - 1 };

    for _ in 0..n {
        if let Some(ev) = a.buf[idx] {
            out.push(ev);
        }
        if idx == 0 { idx = AUDIT_CAP - 1; } else { idx -= 1; }
        if out.len() >= a.total as usize { break; }
    }

    out
}

/// Filtr po world
pub fn by_world(world: u32, limit: usize) -> Vec<AuditEvent> {
    recent(AUDIT_CAP).into_iter()
        .filter(|e| e.world_id == world || e.target_world == world)
        .take(limit)
        .collect()
}

/// Filtr po rodzaju
pub fn by_kind(kind: EventKind, limit: usize) -> Vec<AuditEvent> {
    recent(AUDIT_CAP).into_iter()
        .filter(|e| e.kind == kind)
        .take(limit)
        .collect()
}

/// Liczba odmów (do wykrywania ataków)
pub fn deny_count() -> usize {
    by_kind(EventKind::CheckDeny, AUDIT_CAP).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit() {
        init_audit_log().unwrap();

        log_check(1, Capability::User, true);
        log_check(1, Capability::Driver, false);
        log_grant(0, 1, Capability::User, true);

        assert_eq!(count(), 3);
        assert_eq!(deny_count(), 1);

        let r = recent(3);
        assert_eq!(r.len(), 3);
    }
}