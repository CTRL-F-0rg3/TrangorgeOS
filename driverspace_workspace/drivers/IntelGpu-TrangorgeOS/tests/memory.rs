//! Where a buffer goes, and what the device can honour.
//!
//! The rule under test: the *driver* decides placement, and a request the
//! device cannot satisfy is refused rather than quietly downgraded.

use intel_gpu_trangorgeos::{
    mm::{MemoryModel, Placement},
    probe,
};
use kapi_abi::payloads::gpu::GpuBufferFlags;

const INTEL: u16 = 0x8086;

/// The model a discrete Arc card gets.
fn discrete() -> MemoryModel {
    probe::memory_model(0x56A0).expect("Arc DG2 has a memory model")
}

/// The model an integrated part gets.
fn integrated() -> MemoryModel {
    probe::memory_model(0x4680).expect("Alder Lake has a memory model")
}

// ── what each model can do ─────────────────────────────────────────────────────

#[test]
fn the_two_models_report_different_capacities() {
    let dgpu = discrete();
    let igpu = integrated();

    assert!(dgpu.has_local_memory());
    assert!(!igpu.has_local_memory());
    assert!(dgpu.vram_size() > 0);
    assert_eq!(igpu.vram_size(), 0);
    // Both reach system memory: a discrete card is not a card that cannot
    // see RAM, it is a card that also has memory of its own.
    assert!(dgpu.gtt_size() > 0);
    assert!(igpu.gtt_size() > 0);
}

// ── placement ──────────────────────────────────────────────────────────────────

#[test]
fn an_integrated_part_cannot_honour_a_vram_request() {
    // The important case. Silently placing a VRAM request in system memory
    // would work, be correct, and be slower - and the client would have no way
    // to find out. Refusing tells it the device has no such memory.
    let flags = GpuBufferFlags::VRAM.bits();
    assert_eq!(integrated().placement(flags), Placement::Impossible);
}

#[test]
fn a_discrete_part_honours_a_vram_request() {
    let flags = GpuBufferFlags::VRAM.bits();
    assert_eq!(discrete().placement(flags), Placement::Local);
}

#[test]
fn a_plain_buffer_is_shared_on_both() {
    // Vertex data, indices, uniforms: system memory on every part, because the
    // CPU filled it and the GPU only reads it.
    let flags = (GpuBufferFlags::VERTEX | GpuBufferFlags::INDEX | GpuBufferFlags::UNIFORM).bits();
    assert_eq!(integrated().placement(flags), Placement::Shared);
    assert_eq!(discrete().placement(flags), Placement::Shared);
}

#[test]
fn a_render_target_lands_in_local_memory_on_a_discrete_part() {
    // What makes a discrete card worth having: the framebuffer the GPU writes
    // every frame does not cross PCIe on its way to VRAM.
    let flags = GpuBufferFlags::RENDER_TARGET.bits();
    assert_eq!(discrete().placement(flags), Placement::Local);
}

#[test]
fn a_render_target_lands_in_system_memory_on_an_integrated_part() {
    // There is nowhere else for it to go.
    let flags = GpuBufferFlags::RENDER_TARGET.bits();
    assert_eq!(integrated().placement(flags), Placement::Shared);
}

#[test]
fn a_shared_render_target_stays_shared_even_on_a_discrete_part() {
    // The compositor reads the framebuffer, so it must be somewhere the CPU
    // can see. "Discrete" is not a reason to ignore that.
    let flags = (GpuBufferFlags::RENDER_TARGET | GpuBufferFlags::SHARED).bits();
    assert_eq!(discrete().placement(flags), Placement::Shared);
}

#[test]
fn an_explicit_vram_request_beats_the_render_target_rule() {
    // Both point at local memory, but the client asked, so the answer is
    // unambiguous either way.
    let flags = (GpuBufferFlags::RENDER_TARGET | GpuBufferFlags::VRAM).bits();
    assert_eq!(discrete().placement(flags), Placement::Local);
}

#[test]
fn placement_is_pure() {
    // Same inputs, same answer, every time: the rule is consulted from the
    // allocation path and must not depend on when it runs.
    let flags = GpuBufferFlags::RENDER_TARGET.bits();
    let model = discrete();
    let first = model.placement(flags);
    for _ in 0..16 {
        assert_eq!(model.placement(flags), first);
    }
}
