use crate::caps::{CapEntry, CapId, ObjectType, Process, Rights, Session};
use crate::gfx::{app, draw_demo_window};
use crate::ipc::{IpcMessage, PortId, Router};

#[test]
fn nested_capabilities_are_monotone() {
    let mut parent = Process::new(1, None);
    assert!(parent.grant(CapId(1), CapEntry::new(10, ObjectType::Device, Rights::CALL)));
    assert!(parent.grant(CapId(2), CapEntry::new(20, ObjectType::Memory, Rights::READ)));

    let mut session = Session::new();

    // A child may derive a subset of the parent's holdings (READ over memory).
    let child = session.spawn_derived(
        &parent,
        &[(CapId(2), CapEntry::new(20, ObjectType::Memory, Rights::READ))],
    );
    assert!(child.is_some());

    // A child may NOT derive rights the parent does not hold (WRITE over memory).
    let denied = session.spawn_derived(
        &parent,
        &[(CapId(2), CapEntry::new(20, ObjectType::Memory, Rights::WRITE))],
    );
    assert!(denied.is_none(), "child must not exceed parent's rights");
}

#[test]
fn ipc_router_delivers_messages() {
    let mut router = Router::new(3);

    router.deliver(IpcMessage {
        from: PortId(0),
        to: PortId(1),
        opcode: 7,
        a0: 42,
        ..Default::default()
    });

    let m = router.poll(PortId(1));
    assert!(m.is_some());
    assert_eq!(m.unwrap().a0, 42);
}

#[test]
fn demo_draws_window_and_terminal() {
    let (w, h) = (320u32, 200u32);
    let mut fb = vec![0u32; (w * h) as usize];
    draw_demo_window(&mut fb, w, h);

    // Background corner is the window-background color.
    assert_eq!(fb[0], app::BG);

    // The window's border, title bar, background and prompt text are present.
    assert!(fb.iter().any(|&p| p == app::BORDER), "border should be drawn");
    assert!(fb.iter().any(|&p| p == app::TITLE), "title bar should be drawn");
    assert!(fb.iter().any(|&p| p == app::WIN_BG), "window background should be drawn");
    assert!(fb.iter().any(|&p| p == app::PROMPT), "terminal prompt text should be drawn");
}
