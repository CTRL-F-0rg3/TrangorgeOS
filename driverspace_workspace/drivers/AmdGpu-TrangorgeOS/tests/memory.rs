//! Where a buffer goes, and what the device can honour.
//!
//! The rule under test: the *driver* decides placement, and a request the
//! device cannot satisfy is refused rather than quietly downgraded.

use amd_gpu_trangorgeos::{
    mm::{MemoryModel, Placement},
    probe,
};
use kapi_abi::payloads::gpu::GpuBufferFlags;

const AMD: u16 = 0x1002;

/// The model an RDNA 2 discrete card gets.
fn discrete() -> MemoryModel {
    probe::probe(AMD, 0x73BF, 0, 0, 0).expect("Navi 23").memory
}

/// The model an APU gets.
fn apu() -> MemoryModel {
    probe::probe(AMD, 0x164C, 0, 0, 0).expect("Cezanne").memory
}

// ── what each model can do ─────────────────────────────────────────────────────

#[test]
fn the_two_models_report_different_capacities() {
    let dgpu = discrete();
    let integrated = apu();

    assert!(dgpu.has_local_memory());
    assert!(!integrated.has_local_memory());
    assert!(dgpu.vram_size() > 0);
    assert_eq!(integrated.vram_size(), 0);
    // Both address system memory: a discrete card is not a card that cannot
    // see RAM, it is a card that also has memory of its own.
    assert!(dgpu.aperture() > 0);
    assert!(integrated.aperture() > 0);
}

#[test]
fn a_vega_card_reports_hbm_rather_than_generic_local_memory() {
    // Vega on HBM sits behind a different path, and a driver that treats it
    // as ordinary GDDR will place buffers it should not.
    let device = probe::probe(AMD, 0x6860, 0, 0, 0).expect("Vega 10");
    assert!(device.is_discrete());
    assert!(
        matches!(device.memory, MemoryModel::HbmAndShared { .. }),
        "got {:?}",
        device.memory
    );
}

// ── placement ──────────────────────────────────────────────────────────────────

#[test]
fn an_apu_cannot_honour_a_vram_request() {
    // The important case. Silently placing a VRAM request in system memory
    // would work, be correct, and be indistinguishable from a driver that
    // honoured the request.
    let flags = GpuBufferFlags::VRAM.bits();
    assert_eq!(apu().placement(flags), Placement::Impossible);
}

#[test]
fn a_discrete_card_honours_a_vram_request() {
    let flags = GpuBufferFlags::VRAM.bits();
    assert_eq!(discrete().placement(flags), Placement::Local);
}

#[test]
fn a_plain_buffer_is_shared_on_both() {
    // Vertex data, indices, uniforms: system memory on every part, because the
    // CPU filled it and the GPU only reads it.
    let flags = (GpuBufferFlags::VERTEX | GpuBufferFlags::INDEX | GpuBufferFlags::UNIFORM).bits();
    assert_eq!(apu().placement(flags), Placement::Shared);
    assert_eq!(discrete().placement(flags), Placement::Shared);
}

#[test]
fn a_render_target_lands_in_local_memory_on_a_discrete_card() {
    // What makes a discrete card worth having: the framebuffer the GPU writes
    // every frame does not cross the interconnect to get there.
    let flags = GpuBufferFlags::RENDER_TARGET.bits();
    assert_eq!(discrete().placement(flags), Placement::Local);
}

#[test]
fn a_render_target_lands_in_system_memory_on_an_apu() {
    // There is nowhere else for it to go, and that is fine: an APU's GPU
    // writes the same pages the compositor reads.
    let flags = GpuBufferFlags::RENDER_TARGET.bits();
    assert_eq!(apu().placement(flags), Placement::Shared);
}

#[test]
fn a_shared_render_target_stays_shared_even_on_a_discrete_card() {
    // The compositor reads the framebuffer, so it must be somewhere the CPU can
    // see. "Discrete" is not a reason to ignore that.
    let flags = (GpuBufferFlags::RENDER_TARGET | GpuBufferFlags::SHARED).bits();
    assert_eq!(discrete().placement(flags), Placement::Shared);
}

#[test]
fn an_explicit_vram_request_beats_the_render_target_rule() {
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

#[test]
fn placement_names_read_sensibly_in_a_log() {
    // These end up in an allocation failure message, so the wording is part
    // of the diagnostic.
    assert_eq!(Placement::Local.to_string(), "local");
    assert_eq!(Placement::Shared.to_string(), "shared");
    assert_eq!(Placement::Impossible.to_string(), "unavailable");
}
