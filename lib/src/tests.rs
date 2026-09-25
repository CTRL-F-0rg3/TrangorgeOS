//! Unit tests proving the core security property:
//! **only authorized requests are forwarded to the kernel handler.**
//!
//! These are the executable counterpart of the SPARK proof in `lib-ada`.
//! They run under `cargo test` (where `std` is available).

use crate::*;

/// A driver→kernel request for a video op (requires CALL over a Device).
fn video_request(cap: CapId) -> CommMsg {
    let mut m = CommMsg::new(
        MsgKind::Request,
        Layer::Driverspace,
        Layer::Kernel,
        opcode(OpClass::Video, 1),
    );
    m.cap = cap.0;
    m
}

fn table_with_device_call() -> (CapTable, CapId) {
    let mut t = CapTable::new();
    let cap = t
        .insert(CapEntry::new(42, ObjectType::Device, Rights::CALL))
        .unwrap();
    (t, cap)
}

#[test]
fn authorized_request_is_forwarded() {
    let (t, cap) = table_with_device_call();
    let m = video_request(cap);
    assert_eq!(authorize(&t, &m), AuthorizeResult::Forwarded);
}

#[test]
fn missing_capability_is_denied() {
    let (t, _) = table_with_device_call();
    let m = video_request(CapId(999));
    assert_eq!(authorize(&t, &m), AuthorizeResult::NoCapability);
}

#[test]
fn insufficient_rights_is_denied() {
    let mut t = CapTable::new();
    // READ rights over a device are not enough to CALL it.
    let cap = t
        .insert(CapEntry::new(42, ObjectType::Device, Rights::READ))
        .unwrap();
    let m = video_request(cap);
    assert_eq!(authorize(&t, &m), AuthorizeResult::RightsInsufficient);
}

#[test]
fn wrong_object_type_is_denied() {
    let mut t = CapTable::new();
    // CALL rights but over the wrong object kind (Memory, not Device).
    let cap = t
        .insert(CapEntry::new(42, ObjectType::Memory, Rights::CALL))
        .unwrap();
    let m = video_request(cap);
    assert_eq!(authorize(&t, &m), AuthorizeResult::TypeMismatch);
}

#[test]
fn forbidden_route_is_denied() {
    let (t, cap) = table_with_device_call();
    // Userspace may NOT talk to Driverspace directly.
    let mut m = video_request(cap);
    m.layer = Layer::Userspace as u8;
    m.target = Layer::Driverspace as u8;
    assert_eq!(authorize(&t, &m), AuthorizeResult::RouteDenied);
}

#[test]
fn unknown_opcode_is_denied() {
    let (t, cap) = table_with_device_call();
    let mut m = video_request(cap);
    m.opcode = 0xFF_FF; // unknown class -> never forwarded
    assert_eq!(authorize(&t, &m), AuthorizeResult::UnknownOp);
}

#[test]
fn malformed_magic_is_denied() {
    let (t, cap) = table_with_device_call();
    let mut m = video_request(cap);
    m.magic = 0xDEAD_BEEF;
    assert_eq!(authorize(&t, &m), AuthorizeResult::Malformed);
}

#[test]
fn revoked_capability_is_denied() {
    let (mut t, cap) = table_with_device_call();
    assert_eq!(authorize(&t, &video_request(cap)), AuthorizeResult::Forwarded);
    // Revoking flips the same request to a hard deny.
    assert!(t.remove(cap).is_some());
    assert_eq!(authorize(&t, &video_request(cap)), AuthorizeResult::NoCapability);
}

/// A handler that records how many times it was actually invoked.
struct CountingHandler {
    calls: u32,
}

impl KernelHandler for CountingHandler {
    fn handle(&mut self, _msg: &CommMsg) -> i32 {
        self.calls += 1;
        0
    }
}

#[test]
fn gate_forwards_only_authorized() {
    let (mut table, cap) = table_with_device_call();
    // Also add a SYS capability so a sys request can be authorized.
    table
        .insert(CapEntry::new(1, ObjectType::Device, Rights::CALL))
        .unwrap();

    let mut gate = Gate::new(table, CountingHandler { calls: 0 });

    // 1. Authorized -> handler runs exactly once.
    let mut ok = video_request(cap);
    assert_eq!(gate.dispatch(&mut ok), AuthorizeResult::Forwarded);
    assert_eq!(gate.handler().calls, 1);

    // 2. Denied routes never reach the handler.
    let mut bad_route = video_request(cap);
    bad_route.layer = Layer::Userspace as u8;
    bad_route.target = Layer::Driverspace as u8;
    assert_eq!(gate.dispatch(&mut bad_route), AuthorizeResult::RouteDenied);

    let mut bad_cap = video_request(CapId(777));
    assert_eq!(gate.dispatch(&mut bad_cap), AuthorizeResult::NoCapability);

    let mut bad_magic = video_request(cap);
    bad_magic.magic = 0;
    assert_eq!(gate.dispatch(&mut bad_magic), AuthorizeResult::Malformed);

    // The handler must still have been called only once overall.
    assert_eq!(gate.handler().calls, 1);
}

#[test]
fn ring_roundtrip_preserves_messages() {
    // A small ring (4 slots) round-trips the messages we push.
    let mut storage = [0u8; RING_CONTROL_SIZE + 4 * COMM_MSG_SIZE];
    let ring = unsafe { SpscRing::from_raw(storage.as_mut_ptr(), 4) };

    let (_, cap) = table_with_device_call();
    let m = video_request(cap);

    assert!(ring.push(&m));
    assert_eq!(ring.available(), 1);

    let got = ring.pop().expect("message should be present");
    assert_eq!(got, m);
    assert_eq!(ring.available(), 0);
    assert!(ring.pop().is_none());
}

#[test]
fn manager_grants_and_revokes() {
    let mut mgr = Manager::new();
    let cap = mgr
        .grant(99, ObjectType::Device, Rights::CALL)
        .expect("grant");
    let m = video_request(cap);
    assert_eq!(mgr.authorize(&m), AuthorizeResult::Forwarded);

    assert!(mgr.revoke(cap).is_some());
    assert_eq!(mgr.authorize(&video_request(cap)), AuthorizeResult::NoCapability);
}
