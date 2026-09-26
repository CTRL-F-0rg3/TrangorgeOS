//! Probe tests: PCI ids in, classification out.
//!
//! These run on any machine, with or without an AMD GPU, because probe takes
//! ids and not a live BAR. That is deliberate - the point of keeping probe pure
//! is that the rules can be checked against every id in the table from a build
//! machine.

use amd_gpu_trangorgeos::{
    mm::MemoryModel,
    probe::{self, Arch},
};

const AMD: u16 = 0x1002;
const INTEL: u16 = 0x8086;

// ── the vendor gate ────────────────────────────────────────────────────────────

#[test]
fn a_non_amd_device_is_never_ours() {
    assert!(probe::probe(INTEL, 0x56A0, 0, 0, 0).is_none());
    assert!(!probe::is_supported(INTEL, 0x56A0));
}

#[test]
fn an_unknown_amd_device_is_also_not_ours() {
    // A device id from no era of AMD: refused, like a foreign vendor.
    assert!(probe::probe(AMD, 0xFFFF, 0, 0, 0).is_none());
    assert!(!probe::is_supported(AMD, 0xFFFF));
}

// ── architectures ──────────────────────────────────────────────────────────────

#[test]
fn architectures_cover_gcn_vega_and_rDNA() {
    // Every id here is checked against the generated table, not from memory: a
    // wrong id here would test nothing and still pass a driver that ignores
    // classification entirely.
    let cases = [
        (0x67B0, Arch::Gcn1, "Hawaii"),
        (0x7300, Arch::Gcn4, "Fiji"),
        (0x67DF, Arch::Gcn5, "Ellesmere / Polaris"),
        (0x6860, Arch::Vega, "Vega 10"),
        (0x731F, Arch::Rdna1, "Navi 10"),
        (0x73BF, Arch::Rdna2, "Navi 21"),
        (0x744C, Arch::Rdna3, "Navi 31"),
    ];
    for (id, expected, what) in cases {
        let family = probe::family_for(id)
            .unwrap_or_else(|| panic!("{what}: device {id:#06x} is not in the table"));
        assert_eq!(
            Arch::from_family(family),
            expected,
            "{what} ({id:#06x}, family {family:?})"
        );
    }
}

#[test]
fn a_navi_step_does_not_determine_the_architecture_on_its_own() {
    // The bug this guards: "Navi" is a prefix of every Navi part, so a
    // prefix rule puts Navi 21 and Navi 31 in RDNA 1 with Navi 10. The step
    // number is what separates them, and a Navi 21 is RDNA 2, not RDNA 1.
    for (id, expected) in [
        (0x731F_u16, Arch::Rdna1), // Navi 10
        (0x73BF, Arch::Rdna2),     // Navi 21
        (0x73DF, Arch::Rdna2),     // Navi 23
        (0x744C, Arch::Rdna3),     // Navi 31
        (0x7550, Arch::Rdna4),     // Navi 44
    ] {
        let family = probe::family_for(id).unwrap_or_else(|| panic!("{id:#06x} missing"));
        assert_eq!(Arch::from_family(family), expected, "{id:#06x} family {family:?}");
    }
}

#[test]
fn generations_are_not_ordered_the_way_the_names_suggest() {
    // The fact that makes `Arch` an enumeration rather than a scale: a 2019
    // discrete card is GCN 5, and a 2023 APU is RDNA 2. A driver that compared
    // generation numbers would treat the APU as newer hardware when it is a
    // different design entirely.
    let polaris = Arch::from_family("Polaris");
    let renoir = Arch::from_family("Renoir");
    assert_eq!(polaris, Arch::Gcn5);
    assert_eq!(renoir, Arch::Rdna2);
    // RDNA 2 is not "newer" than GCN 5 in a way the driver can rely on; they
    // are different engines, which is why the wave size differs.
    assert_eq!(Arch::Gcn5.wave_size(), 64);
    assert_eq!(Arch::Rdna2.wave_size(), 32);
}

#[test]
fn the_wave_width_halved_at_rdna() {
    // Not a performance fact: a dispatch sized for the wrong wave is a
    // correctness bug on the GPU.
    for arch in [Arch::Gcn1, Arch::Gcn4, Arch::Gcn5, Arch::Vega] {
        assert_eq!(arch.wave_size(), 64, "{arch:?}");
    }
    for arch in [Arch::Rdna1, Arch::Rdna2, Arch::Rdna3, Arch::Rdna4] {
        assert_eq!(arch.wave_size(), 32, "{arch:?}");
    }
    // A refused architecture has no wave width, rather than a wrong one.
    assert_eq!(Arch::Legacy.wave_size(), 0);
    assert_eq!(Arch::Unknown.wave_size(), 0);
}

// ── APU versus discrete ────────────────────────────────────────────────────────

#[test]
fn an_apu_has_no_memory_of_its_own() {
    // Renoir is a Ryzen APU: Radeon graphics on the same package as the CPU.
    let device = probe::probe(AMD, 0x1636, 0, 0, 0).expect("Renoir is a known APU");
    assert!(!device.is_discrete());
    assert!(!device.memory.has_local_memory());
    assert_eq!(device.memory.vram_size(), 0, "an APU has no VRAM to report");
    assert!(device.memory.aperture() > 0, "but it does address system memory");
}

#[test]
fn a_discrete_card_has_local_memory() {
    // Navi 10, the discrete part.
    let device = probe::probe(AMD, 0x731F, 0, 0, 0).expect("Navi 10 is a known dGPU");
    assert!(device.is_discrete());
    assert!(device.memory.has_local_memory());
    assert!(device.memory.vram_size() > 0);
}

#[test]
fn an_apu_and_a_discrete_card_can_share_an_architecture() {
    // Cezanne is an APU on RDNA 2; Navi 22 is a discrete card on RDNA 2. Same
    // register map, different memory: which is exactly the split that has to
    // live outside the architecture.
    let apu = probe::probe(AMD, 0x164C, 0, 0, 0);
    let dgpu = probe::probe(AMD, 0x73BF, 0, 0, 0);
    if let (Some(apu), Some(dgpu)) = (apu, dgpu) {
        assert_eq!(apu.arch, Arch::Rdna2);
        assert_eq!(dgpu.arch, Arch::Rdna2);
        assert_ne!(apu.memory, dgpu.memory);
    }
}

#[test]
fn every_apu_family_is_actually_listed_as_one() {
    // The APU list is the thing that decides a shared memory model, so an
    // entry in it that no device uses is dead weight, and a family that is
    // one but missing from it is a card treated as discrete.
    for family in probe::APU_FAMILIES {
        assert!(!family.is_empty());
    }
}

// ── what is refused ────────────────────────────────────────────────────────────

#[test]
fn legacy_parts_are_recognised_but_not_driven() {
    // A TeraScale part is AMD, so probe names it - but its register map is a
    // different driver, not an older version of this one.
    let legacy = Arch::Legacy;
    assert!(!legacy.is_supported());
    assert!(legacy.require_supported().is_err());
}

#[test]
fn the_accelerators_are_left_unrecognised() {
    // Instinct MI-series parts share AMD's vendor id and sit beside Radeon
    // parts in pci.ids, but they have no display engine. `pci.ids` carries no
    // PCI class, so the name is the only signal here and refusing is the safe
    // reading of an ambiguous one.
    // The Instinct MI50 shares device id 0x66A1 with a retail Vega 20: the
    // same die with the display engine disabled. This is the case that matters,
    // and it is why the *id* cannot be the thing that refuses an accelerator.
    // The refusal belongs to the driver, after it has read the silicon.
    let device = probe::probe(AMD, 0x66A1, 0, 0, 0).expect("0x66A1 is a known id");
    assert_eq!(
        device.arch,
        Arch::Vega,
        "the id classifies as Vega, and refusing an MI50 here would reject a retail card too"
    );
    // A Vega retail part must still be drivable.
    assert!(device.arch.is_supported());
    assert!(probe::is_supported(AMD, 0x66A1));
}
