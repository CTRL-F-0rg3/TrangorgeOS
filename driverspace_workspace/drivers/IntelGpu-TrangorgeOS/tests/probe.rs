//! Probe tests: PCI ids in, classification out.
//!
//! These run on any machine, with or without an Intel GPU, because probe
//! takes ids and not a live BAR. That is deliberate - the point of keeping
//! probe pure is that the rules can be checked against every id in the tree
//! from a build machine.

use intel_gpu_trangorgeos::probe::{self, GenKind};

const INTEL: u16 = 0x8086;
const AMD: u16 = 0x1002;

// ── the vendor gate ────────────────────────────────────────────────────────────

#[test]
fn a_non_intel_device_is_never_ours() {
    // The common case on any bus. An AMD card must come back as "not mine" so
    // `ds-detect` can hand it to the AMD driver instead.
    assert!(probe::probe(AMD, 0x1234, 0, 0, 0).is_none());
    assert!(!probe::is_supported(AMD, 0x1234));
}

#[test]
fn an_unknown_intel_device_is_also_not_ours() {
    // Deliberately indistinguishable from a non-Intel device: both mean the
    // same thing to the loader.
    assert!(probe::probe(INTEL, 0xFFFF, 0, 0, 0).is_none());
    assert!(!probe::is_supported(INTEL, 0xFFFF));
}

// ── the iGPU / dGPU split ──────────────────────────────────────────────────────

#[test]
fn arc_dg2_is_classified_discrete() {
    // A770 is 0x56A0. This is the case the whole split exists for.
    let device = probe::probe(INTEL, 0x56A0, 0, 0, 0).expect("A770 is an Intel display device");
    assert!(device.is_discrete(), "an Arc card has local memory");
    assert!(device.memory.has_local_memory());
    assert!(device.memory.vram_size() > 0, "and it has a real size");
}

#[test]
fn arc_dg1_is_classified_discrete() {
    // The Xe MAX board, 0x4905.
    let device = probe::probe(INTEL, 0x4905, 0, 0, 0).expect("DG1 is an Intel display device");
    assert!(device.is_discrete());
}

#[test]
fn alder_lake_iGPU_is_classified_integrated() {
    // 0x4680 is Alder Lake integrated. It is the *same generation* as an Arc
    // card, which is the whole reason the memory model is a separate axis.
    let device = probe::probe(INTEL, 0x4680, 0, 0, 0).expect("ADL-S is an Intel display device");
    assert!(!device.is_discrete(), "Alder Lake iGPU has no local memory");
    assert_eq!(device.memory.vram_size(), 0);
    assert!(device.memory.gtt_size() > 0, "but it does have a GTT");
}

#[test]
fn the_same_generation_serves_both_an_iGPU_and_a_dGPU() {
    // The fact that makes one driver correct rather than a compromise: ADL
    // integrated and Arc DG2 need the same register map.
    let igpu = probe::probe(INTEL, 0x4680, 0, 0, 0).unwrap();
    let dgpu = probe::probe(INTEL, 0x56A0, 0, 0, 0).unwrap();
    assert_eq!(
        igpu.generation, dgpu.generation,
        "same generation, so the same register map"
    );
    assert_ne!(
        igpu.memory, dgpu.memory,
        "but a different memory model, so one driver is still not one code path"
    );
}

#[test]
fn tiger_lake_and_ice_lake_are_integrated() {
    for id in [0x9A40_u16, 0x8A50] {
        let device = probe::probe(INTEL, id, 0, 0, 0).expect("a known Intel iGPU");
        assert!(!device.is_discrete(), "{id:#06x} should be integrated");
    }
}

// ── generations ────────────────────────────────────────────────────────────────

#[test]
fn generations_cover_the_whole_modern_range() {
    let cases = [
        (0x8A50, GenKind::Gen11),   // Ice Lake
        (0x9A40, GenKind::Gen12),   // Tiger Lake
        (0x4680, GenKind::Gen12_5), // Alder Lake iGPU
        (0x56A0, GenKind::Gen12_5), // Arc DG2 - the same generation
        (0xA780, GenKind::Gen12_7), // Raptor Lake
        (0x7D55, GenKind::Gen12_7), // Meteor Lake
        (0xE20B, GenKind::Gen13),   // Battlemage
    ];
    for (id, expected) in cases {
        assert_eq!(GenKind::from_device_id(id), Some(expected), "device {id:#06x}");
    }
}

#[test]
fn old_parts_are_recognised_but_refused() {
    // Broadwell is Intel, so probe finds it - but this driver does not drive
    // it, and pretending otherwise would mean writing registers that mean
    // something else.
    let device = probe::probe(INTEL, 0x1602, 0, 0, 0).expect("Broadwell is an Intel display device");
    assert_eq!(device.generation, GenKind::Gen8);
    assert!(!device.generation.is_supported());
    assert!(device.generation.require_supported().is_err());
    // And the loader is told not to bother.
    assert!(!probe::is_supported(INTEL, 0x1602));
}

#[test]
fn supported_generations_pass_the_gate() {
    for id in [0x8A50_u16, 0x9A40, 0x4680, 0x56A0, 0xE20B] {
        assert!(probe::is_supported(INTEL, id), "device {id:#06x} should load");
    }
}

#[test]
fn generation_names_are_stable_strings() {
    // These go into `GpuInfo` and into logs; they are part of what a user sees.
    assert_eq!(GenKind::Gen12_5.as_str(), "Gen12.5");
    assert_eq!(GenKind::Gen9_5_Cml.as_str(), "Gen9.5 (CML)");
}

// ── names ──────────────────────────────────────────────────────────────────────

#[test]
fn arc_cards_get_their_market_names() {
    assert!(probe::name_for(0x56A0).contains("A770"), "got {:?}", probe::name_for(0x56A0));
    assert!(probe::name_for(0x56A6).contains("A310"), "got {:?}", probe::name_for(0x56A6));
}

#[test]
fn probe_records_where_it_found_the_device() {
    let device = probe::probe(INTEL, 0x56A0, 0x01, 0x04, 0x00).unwrap();
    assert_eq!(device.bus, 1);
    assert_eq!(device.slot, 4);
    assert_eq!(device.function, 0);
    assert_eq!(device.vendor_id, INTEL);
}

#[test]
fn a_probed_device_always_has_a_memory_model() {
    // If this ever fails, a caller is about to place a buffer without knowing
    // whether the device can.
    for id in [0x8A50_u16, 0x4680, 0x56A0, 0xE20B] {
        let model = probe::memory_model(id);
        assert!(model.is_some(), "device {id:#06x} has no memory model");
        assert!(model.unwrap().gtt_size() > 0, "and no GTT");
    }
}
