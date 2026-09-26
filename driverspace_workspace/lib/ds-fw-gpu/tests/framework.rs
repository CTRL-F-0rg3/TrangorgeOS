//! Framework tests against a mock GPU.
//!
//! These exercise the rules the *framework* owns - capability gating, handle
//! lifetime, payload validation - rather than any vendor's hardware. Both
//! `AmdGpu-TrangorgeOS` and `IntelGpu-TrangorgeOS` inherit this coverage,
//! because the contract they implement is the one tested here.

use ds_fw_gpu::{
    codec, mock::MockGpu, GpuService, Reply, Request,
    service::required_capability,
};
use kapi_abi::payloads::gpu::{GpuBufferFlags, GpuShaderFlags, GpuVendor};
use kapi_abi::{CapId, DsCmd, DsError, DsMsg};

/// A client holding every GPU capability: the trusted compositor path.
const PRIVILEGED: CapId = CapId::GPU_ENUMERATE.union(CapId::GPU_DEVICE);

/// A message for `cmd` with no payload.
fn msg(cmd: DsCmd) -> DsMsg {
    DsMsg { cmd: cmd as u16, ..Default::default() }
}

/// A message for `cmd` whose scalar arguments are set.
fn msg_args(cmd: DsCmd, arg0: u64, arg1: u64) -> DsMsg {
    DsMsg { cmd: cmd as u16, arg0, arg1, ..Default::default() }
}

/// A service wrapping an AMD mock.
fn amd_service() -> GpuService {
    GpuService::new(Box::new(MockGpu::amd()))
}

/// A service wrapping an Intel mock.
fn intel_service() -> GpuService {
    GpuService::new(Box::new(MockGpu::intel()))
}

/// The payload for `GpuContextCreate` on device 0.
fn context_payload() -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&0u32.to_le_bytes()); // device
    p.extend_from_slice(&1u32.to_le_bytes()); // engines
    p.extend_from_slice(&0u64.to_le_bytes()); // shmem base
    p.extend_from_slice(&0u64.to_le_bytes()); // shmem size
    p
}

/// The payload for a `GpuSubmit` of 256 bytes from ring offset 0.
fn submit_payload(context: u32) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&context.to_le_bytes());
    p.extend_from_slice(&0u32.to_le_bytes()); // offset
    p.extend_from_slice(&256u32.to_le_bytes()); // length
    p.extend_from_slice(&0u32.to_le_bytes()); // ring
    p.extend_from_slice(&1u32.to_le_bytes()); // engines
    p.extend_from_slice(&0u32.to_le_bytes()); // pad
    p
}

/// Create a context and return its handle, asserting it worked.
fn make_context(service: &mut GpuService) -> u32 {
    let mut reply = Vec::new();
    let request = Request::new(msg(DsCmd::GpuContextCreate), &context_payload());
    let out = service.dispatch(&request, PRIVILEGED, &mut reply);
    assert!(out.is_ok(), "create failed with status {}", out.status);
    out.arg0 as u32
}

// ── the capability split ───────────────────────────────────────────────────────

#[test]
fn the_service_advertises_both_gpu_capabilities() {
    let service = amd_service();
    assert!(service.caps().has(CapId::GPU_ENUMERATE));
    assert!(service.caps().has(CapId::GPU_DEVICE));
    assert_eq!(service.vendor(), GpuVendor::Amd);
}

#[test]
fn reading_needs_only_enumerate_and_writing_needs_device() {
    // The whole point of the split: a client that may look must not drive.
    assert_eq!(required_capability(DsCmd::GpuEnumerate), Some(CapId::GPU_ENUMERATE));
    assert_eq!(required_capability(DsCmd::GpuInfo), Some(CapId::GPU_ENUMERATE));
    assert_eq!(required_capability(DsCmd::GpuFormatStride), Some(CapId::GPU_ENUMERATE));

    for cmd in [
        DsCmd::GpuContextCreate,
        DsCmd::GpuContextDestroy,
        DsCmd::GpuBufferAlloc,
        DsCmd::GpuBufferFree,
        DsCmd::GpuSubmit,
        DsCmd::GpuFenceQuery,
        DsCmd::GpuShaderCompile,
        DsCmd::GpuShaderFree,
        DsCmd::GpuReset,
    ] {
        assert_eq!(required_capability(cmd), Some(CapId::GPU_DEVICE), "{cmd:?}");
    }
}

#[test]
fn an_opcode_from_another_device_class_is_rejected() {
    // A block read sent to a GPU driver is not GPU work, whatever caps it holds.
    let mut service = amd_service();
    let mut reply = Vec::new();
    let out = service.dispatch(&Request::new(msg(DsCmd::BlkRead), &[]), CapId::KERNEL_ALL, &mut reply);
    assert_eq!(out.status, DsError::InvalidMessage as i32);
}

#[test]
fn a_client_with_no_capabilities_is_refused() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let out = service.dispatch(&Request::new(msg(DsCmd::GpuEnumerate), &[]), CapId::NONE, &mut reply);
    assert_eq!(out.status, DsError::PermissionDenied as i32);
}

#[test]
fn a_client_that_may_look_may_not_drive() {
    // `GPU_ENUMERATE` alone must not be enough to create a context.
    let mut service = amd_service();
    let mut reply = Vec::new();
    let request = Request::new(msg(DsCmd::GpuContextCreate), &context_payload());
    let out = service.dispatch(&request, CapId::GPU_ENUMERATE, &mut reply);
    assert_eq!(out.status, DsError::PermissionDenied as i32);
    assert_eq!(service.live_contexts(), 0, "the driver was never asked");
}

// ── discovery ───────────────────────────────────────────────────────────────────

#[test]
fn enumeration_reports_a_count_and_describes_the_device() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let out = service.dispatch(&Request::new(msg(DsCmd::GpuEnumerate), &[]), CapId::GPU_ENUMERATE, &mut reply);
    assert!(out.is_ok());
    assert_eq!(out.arg0, 1, "one device reported");
    let info = codec::decode_device(&reply).expect("a decodable GpuInfo");
    assert_eq!(info.vendor, GpuVendor::Amd);
    assert_eq!(info.vendor_id, 0x1002);
    assert_eq!(info.index, 0);
}

#[test]
fn an_intel_driver_reports_intel() {
    let mut service = intel_service();
    let mut reply = Vec::new();
    let out = service.dispatch(&Request::new(msg(DsCmd::GpuEnumerate), &[]), CapId::GPU_ENUMERATE, &mut reply);
    assert!(out.is_ok());
    let info = codec::decode_device(&reply).expect("a decodable GpuInfo");
    // The two drivers are separate, and the framework keeps them apart.
    assert_eq!(info.vendor, GpuVendor::Intel);
    assert_eq!(info.vendor_id, 0x8086);
    assert_ne!(info.vendor, GpuVendor::Amd);
}

#[test]
fn walking_past_the_last_device_is_not_an_error() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let request = Request::new(msg_args(DsCmd::GpuEnumerate, 5, 0), &[]);
    let out = service.dispatch(&request, CapId::GPU_ENUMERATE, &mut reply);
    // The count still comes back, so a client can stop walking.
    assert!(out.is_ok());
    assert_eq!(out.arg0, 1);
    assert!(reply.is_empty());
}

#[test]
fn reading_a_device_that_does_not_exist_is_an_error() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let request = Request::new(msg_args(DsCmd::GpuInfo, 3, 0), &[]);
    let out = service.dispatch(&request, CapId::GPU_ENUMERATE, &mut reply);
    assert_eq!(out.status, DsError::DeviceNotFound as i32);
}

#[test]
fn the_stride_answer_comes_from_the_driver_not_the_framework() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    // Width 1920, XRGB8888.
    let request = Request::new(msg_args(DsCmd::GpuFormatStride, 1920, 0), &[]);
    let out = service.dispatch(&request, CapId::GPU_ENUMERATE, &mut reply);
    assert!(out.is_ok());
    assert_eq!(out.arg0, 1920 * 4);
}

// ── context lifetime ───────────────────────────────────────────────────────────

#[test]
fn the_context_lifecycle_works_end_to_end() {
    let mut service = amd_service();
    let mut reply = Vec::new();

    let request = Request::new(msg(DsCmd::GpuContextCreate), &context_payload());
    let out = service.dispatch(&request, PRIVILEGED, &mut reply);
    assert!(out.is_ok(), "create failed with status {}", out.status);
    let context = out.arg0 as u32;
    assert_ne!(context, u32::MAX, "a real handle was issued, not the sentinel");
    assert_eq!(service.live_contexts(), 1);

    // The handle also appears in the reply payload, so a client that only
    // reads payloads still sees the same value the header promised.
    let info = codec::decode_context_info(&reply).expect("a decodable GpuContextInfo");
    assert_eq!(info.handle, context);
    assert_eq!(info.ring_size, 1 << 20);
    assert!(info.ring_tail_usable < info.ring_size, "the tail is guarded");

    reply.clear();
    let destroy = Request::new(msg_args(DsCmd::GpuContextDestroy, context as u64, 0), &[]);
    let out = service.dispatch(&destroy, PRIVILEGED, &mut reply);
    assert!(out.is_ok());
    assert_eq!(service.live_contexts(), 0);
}

#[test]
fn a_destroyed_handle_is_stale_afterwards() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let context = make_context(&mut service);

    let destroy = Request::new(msg_args(DsCmd::GpuContextDestroy, context as u64, 0), &[]);
    assert!(service.dispatch(&destroy, PRIVILEGED, &mut reply).is_ok());

    // The same handle again: it is stale and must not resolve.
    let again = Request::new(msg_args(DsCmd::GpuContextDestroy, context as u64, 0), &[]);
    let out = service.dispatch(&again, PRIVILEGED, &mut reply);
    assert_eq!(out.status, DsError::InvalidHandle as i32);
}

#[test]
fn a_truncated_context_payload_is_refused_before_the_driver() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    // Far too short to be a context request.
    let request = Request::new(msg(DsCmd::GpuContextCreate), &[1, 2, 3]);
    let out = service.dispatch(&request, PRIVILEGED, &mut reply);
    assert_eq!(out.status, DsError::InvalidMessage as i32);
    assert_eq!(service.live_contexts(), 0);
}

// ── buffers ────────────────────────────────────────────────────────────────────

#[test]
fn a_buffer_is_allocated_and_freed_within_a_context() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let context = make_context(&mut service);

    let mut payload = Vec::new();
    payload.extend_from_slice(&4096u64.to_le_bytes());
    let flags = GpuBufferFlags::RENDER_TARGET | GpuBufferFlags::VRAM;
    payload.extend_from_slice(&flags.bits().to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes());

    let alloc = Request::new(msg_args(DsCmd::GpuBufferAlloc, context as u64, 0), &payload);
    let out = service.dispatch(&alloc, PRIVILEGED, &mut reply);
    assert!(out.is_ok(), "alloc failed with status {}", out.status);

    let buffer = codec::decode_buffer(&reply).expect("a decodable GpuBuffer");
    assert_eq!(buffer.size, 4096);
    assert_ne!(buffer.phys, 0, "the driver placed it in memory");
    assert_eq!(out.arg0, buffer.handle as u64, "header and payload agree");

    reply.clear();
    let free = Request::new(msg_args(DsCmd::GpuBufferFree, context as u64, buffer.handle as u64), &[]);
    assert!(service.dispatch(&free, PRIVILEGED, &mut reply).is_ok());
}

#[test]
fn a_buffer_request_for_a_dead_context_is_refused() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    // No context has ever been created, so handle 1 is not live.
    let mut payload = Vec::new();
    payload.extend_from_slice(&4096u64.to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes());
    let alloc = Request::new(msg_args(DsCmd::GpuBufferAlloc, 1, 0), &payload);
    let out = service.dispatch(&alloc, PRIVILEGED, &mut reply);
    assert_eq!(out.status, DsError::InvalidHandle as i32);
}

// ── shaders ────────────────────────────────────────────────────────────────────

#[test]
fn a_shader_compiles_with_its_source_attached() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let context = make_context(&mut service);

    let source = b"void main() {}";
    let mut payload = Vec::new();
    payload.extend_from_slice(&context.to_le_bytes());
    payload.extend_from_slice(&GpuShaderFlags::VERTEX.bits().to_le_bytes());
    payload.extend_from_slice(&(source.len() as u32).to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes());
    payload.extend_from_slice(source);

    let compile = Request::new(msg_args(DsCmd::GpuShaderCompile, context as u64, 0), &payload);
    let out = service.dispatch(&compile, PRIVILEGED, &mut reply);
    assert!(out.is_ok(), "compile failed with status {}", out.status);
    let shader = codec::decode_shader(&reply).expect("a decodable GpuShader");
    assert_eq!(shader.hash, 0xDEAD_BEEF, "the driver hashed the program");
}

#[test]
fn a_shader_claiming_a_source_that_is_not_there_is_refused() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let context = make_context(&mut service);

    // Claims 1 MB of source; five bytes follow. This must be an error rather
    // than an allocation sized by a number off the wire.
    let mut payload = Vec::new();
    payload.extend_from_slice(&context.to_le_bytes());
    payload.extend_from_slice(&GpuShaderFlags::VERTEX.bits().to_le_bytes());
    payload.extend_from_slice(&1_000_000u32.to_le_bytes());
    payload.extend_from_slice(&0u32.to_le_bytes());
    payload.extend_from_slice(b"short");

    let compile = Request::new(msg_args(DsCmd::GpuShaderCompile, context as u64, 0), &payload);
    let out = service.dispatch(&compile, PRIVILEGED, &mut reply);
    assert_eq!(out.status, DsError::InvalidMessage as i32);
}

// ── submission and fences ──────────────────────────────────────────────────────

#[test]
fn a_submission_returns_a_fence_that_advances() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let context = make_context(&mut service);
    let payload = submit_payload(context);
    let submit = Request::new(msg_args(DsCmd::GpuSubmit, context as u64, 0), &payload);

    let f1 = service.dispatch(&submit, PRIVILEGED, &mut reply).arg0;
    assert!(f1 > 0, "a real fence, not the sentinel");

    let f2 = service.dispatch(&submit, PRIVILEGED, &mut reply).arg0;
    // Monotonicity is the only property a client needs to know that a batch
    // completed; a driver that reused a value would break every wait.
    assert!(f2 > f1, "fence {f2} should exceed {f1}");
}

#[test]
fn a_submission_for_a_dead_context_is_refused() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let context = make_context(&mut service);
    let payload = submit_payload(context);

    let destroy = Request::new(msg_args(DsCmd::GpuContextDestroy, context as u64, 0), &[]);
    service.dispatch(&destroy, PRIVILEGED, &mut reply);

    let submit = Request::new(msg_args(DsCmd::GpuSubmit, context as u64, 0), &payload);
    let out = service.dispatch(&submit, PRIVILEGED, &mut reply);
    assert_eq!(out.status, DsError::InvalidHandle as i32);
}

#[test]
fn a_truncated_submission_is_refused() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let context = make_context(&mut service);
    let submit = Request::new(msg_args(DsCmd::GpuSubmit, context as u64, 0), &[0u8; 4]);
    let out = service.dispatch(&submit, PRIVILEGED, &mut reply);
    assert_eq!(out.status, DsError::InvalidMessage as i32);
}

#[test]
fn a_driver_refusal_reaches_the_client_unchanged() {
    // The framework must not report its own success when the driver refused.
    let mut service = amd_service();
    let mut reply = Vec::new();
    let context = make_context(&mut service);
    let payload = submit_payload(context);
    service.device_mut().fail_submit = true;

    let submit = Request::new(msg_args(DsCmd::GpuSubmit, context as u64, 0), &payload);
    let out = service.dispatch(&submit, PRIVILEGED, &mut reply);
    assert_eq!(out.status, DsError::DeviceBusy as i32);
}

// ── recovery ───────────────────────────────────────────────────────────────────

#[test]
fn a_forced_reset_invalidates_every_context() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    make_context(&mut service);
    assert_eq!(service.live_contexts(), 1);

    let reset = Request::new(msg_args(DsCmd::GpuReset, 0, 1), &[]);
    let out = service.dispatch(&reset, PRIVILEGED, &mut reply);
    assert!(out.is_ok());
    // The ring is gone, so the context must be too: a live handle would let a
    // client submit into memory that no longer means anything.
    assert_eq!(service.live_contexts(), 0);
}

#[test]
fn resetting_a_device_that_does_not_exist_is_an_error() {
    let mut service = amd_service();
    let mut reply = Vec::new();
    let reset = Request::new(msg_args(DsCmd::GpuReset, 9, 1), &[]);
    let out = service.dispatch(&reset, PRIVILEGED, &mut reply);
    assert_eq!(out.status, DsError::DeviceNotFound as i32);
}

#[test]
fn a_reply_echoes_the_request_identity() {
    // The caller matches a reply to its request by id, so this must survive.
    let mut service = amd_service();
    let mut reply = Vec::new();
    let request = msg(DsCmd::GpuEnumerate);
    let out: Reply = service.dispatch(&Request::new(request, &[]), CapId::GPU_ENUMERATE, &mut reply);
    let sent = out.into_message(&request);
    assert_eq!(sent.id, request.id);
    assert_eq!(sent.cmd, request.cmd);
    assert_eq!(sent.status, 0);
}
