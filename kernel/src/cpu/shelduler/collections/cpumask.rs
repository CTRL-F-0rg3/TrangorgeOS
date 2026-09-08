#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, Ordering};

use crate::cpu::scheduler::bitmap::AtomicBitmap;
pub use crate::cpu::scheduler::entities::task::CpuMask;
use crate::cpu::scheduler::entities::task::MAX_CPUS;

const WORDS: usize = MAX_CPUS / 64;

static POSSIBLE: AtomicBitmap<WORDS> = AtomicBitmap::new();
static PRESENT: AtomicBitmap<WORDS> = AtomicBitmap::new();
static ONLINE: AtomicBitmap<WORDS> = AtomicBitmap::new();
static ACTIVE: AtomicBitmap<WORDS> = AtomicBitmap::new();

const NO_TOPOLOGY: AtomicU32 = AtomicU32::new(u32::MAX);
static PACKAGE_ID: [AtomicU32; MAX_CPUS] = [NO_TOPOLOGY; MAX_CPUS];
static CORE_ID: [AtomicU32; MAX_CPUS] = [NO_TOPOLOGY; MAX_CPUS];

// ------------------------------------------------------------------
// Cykl życia CPU: possible -> present -> online -> active
// ------------------------------------------------------------------

pub fn mark_possible(cpu: u32) {
    if (cpu as usize) < MAX_CPUS {
        POSSIBLE.test_and_set(cpu as usize);
    }
}

pub fn mark_present(cpu: u32) {
    if (cpu as usize) < MAX_CPUS {
        PRESENT.test_and_set(cpu as usize);
    }
}

pub fn mark_online(cpu: u32) {
    if (cpu as usize) < MAX_CPUS {
        ONLINE.test_and_set(cpu as usize);
        ACTIVE.test_and_set(cpu as usize);
    }
}

pub fn mark_offline(cpu: u32) {
    if (cpu as usize) < MAX_CPUS {
        ACTIVE.test_and_clear(cpu as usize);
        ONLINE.test_and_clear(cpu as usize);
    }
}

pub fn mark_active(cpu: u32) {
    if (cpu as usize) < MAX_CPUS {
        ACTIVE.test_and_set(cpu as usize);
    }
}

pub fn mark_inactive(cpu: u32) {
    if (cpu as usize) < MAX_CPUS {
        ACTIVE.test_and_clear(cpu as usize);
    }
}

pub fn is_possible(cpu: u32) -> bool {
    (cpu as usize) < MAX_CPUS && POSSIBLE.test(cpu as usize)
}

pub fn is_present(cpu: u32) -> bool {
    (cpu as usize) < MAX_CPUS && PRESENT.test(cpu as usize)
}

pub fn is_online(cpu: u32) -> bool {
    (cpu as usize) < MAX_CPUS && ONLINE.test(cpu as usize)
}

pub fn is_active(cpu: u32) -> bool {
    (cpu as usize) < MAX_CPUS && ACTIVE.test(cpu as usize)
}

pub fn num_possible_cpus() -> u32 {
    POSSIBLE.weight()
}

pub fn num_present_cpus() -> u32 {
    PRESENT.weight()
}

pub fn num_online_cpus() -> u32 {
    ONLINE.weight()
}

pub fn num_active_cpus() -> u32 {
    ACTIVE.weight()
}

fn snapshot(bm: &AtomicBitmap<WORDS>) -> CpuMask {
    let mut out = CpuMask::empty();
    for cpu in 0..MAX_CPUS as u32 {
        if bm.test(cpu as usize) {
            out.set(cpu);
        }
    }
    out
}

pub fn possible_mask() -> CpuMask {
    snapshot(&POSSIBLE)
}

pub fn present_mask() -> CpuMask {
    snapshot(&PRESENT)
}

pub fn online_mask() -> CpuMask {
    snapshot(&ONLINE)
}

pub fn active_mask() -> CpuMask {
    snapshot(&ACTIVE)
}

pub fn for_each_online_cpu() -> impl Iterator<Item = u32> {
    online_mask().iter()
}

pub fn for_each_active_cpu() -> impl Iterator<Item = u32> {
    active_mask().iter()
}

pub fn first_online_cpu() -> Option<u32> {
    online_mask().first()
}

// ------------------------------------------------------------------
// Topologia (pakiet/gniazdo, rdzeń fizyczny) — do domen szeregowania
// świadomych SMT (hyperthreading).
// ------------------------------------------------------------------

pub fn set_topology(cpu: u32, package_id: u32, core_id: u32) {
    if (cpu as usize) < MAX_CPUS {
        PACKAGE_ID[cpu as usize].store(package_id, Ordering::Release);
        CORE_ID[cpu as usize].store(core_id, Ordering::Release);
    }
}

pub fn package_of(cpu: u32) -> Option<u32> {
    if (cpu as usize) >= MAX_CPUS {
        return None;
    }
    let id = PACKAGE_ID[cpu as usize].load(Ordering::Acquire);
    if id == u32::MAX {
        None
    } else {
        Some(id)
    }
}

pub fn core_of(cpu: u32) -> Option<u32> {
    if (cpu as usize) >= MAX_CPUS {
        return None;
    }
    let id = CORE_ID[cpu as usize].load(Ordering::Acquire);
    if id == u32::MAX {
        None
    } else {
        Some(id)
    }
}

/// Maska rdzeni logicznych dzielących ten sam rdzeń fizyczny (SMT).
/// Zawsze zawiera `cpu` samo w sobie, jeśli jest online.
pub fn sibling_mask(cpu: u32) -> CpuMask {
    let mut out = CpuMask::empty();
    let pkg = match package_of(cpu) {
        Some(p) => p,
        None => return out,
    };
    let core = match core_of(cpu) {
        Some(c) => c,
        None => return out,
    };
    for c in 0..MAX_CPUS as u32 {
        if is_online(c) && package_of(c) == Some(pkg) && core_of(c) == Some(core) {
            out.set(c);
        }
    }
    out
}

/// Maska wszystkich rdzeni logicznych w tym samym pakiecie/gnieździe.
pub fn package_mask(cpu: u32) -> CpuMask {
    let mut out = CpuMask::empty();
    let pkg = match package_of(cpu) {
        Some(p) => p,
        None => return out,
    };
    for c in 0..MAX_CPUS as u32 {
        if is_online(c) && package_of(c) == Some(pkg) {
            out.set(c);
        }
    }
    out
}

pub fn are_siblings(a: u32, b: u32) -> bool {
    a != b
        && package_of(a).is_some()
        && package_of(a) == package_of(b)
        && core_of(a) == core_of(b)
}

// ------------------------------------------------------------------
// Pomocnicze konstruktory masek
// ------------------------------------------------------------------

pub fn mask_from_iter<I: IntoIterator<Item = u32>>(cpus: I) -> CpuMask {
    let mut m = CpuMask::empty();
    for c in cpus {
        m.set(c);
    }
    m
}

pub fn restrict_to_online(mask: &CpuMask) -> CpuMask {
    mask.and(&online_mask())
}

pub fn restrict_to_active(mask: &CpuMask) -> CpuMask {
    mask.and(&active_mask())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn possible_present_online_active_lifecycle() {
        let cpu = 10;
        assert!(!is_possible(cpu));
        mark_possible(cpu);
        mark_present(cpu);
        assert!(is_possible(cpu));
        assert!(is_present(cpu));
        assert!(!is_online(cpu));

        mark_online(cpu);
        assert!(is_online(cpu));
        assert!(is_active(cpu));

        mark_inactive(cpu);
        assert!(is_online(cpu));
        assert!(!is_active(cpu));

        mark_offline(cpu);
        assert!(!is_online(cpu));
        assert!(!is_active(cpu));
    }

    #[test]
    fn masks_reflect_live_state_for_dedicated_cpu() {
        let cpu = 11;
        mark_possible(cpu);
        mark_online(cpu);
        assert!(possible_mask().is_set(cpu));
        assert!(online_mask().is_set(cpu));
        assert!(active_mask().is_set(cpu));
        mark_offline(cpu);
        assert!(!online_mask().is_set(cpu));
    }

    #[test]
    fn num_counters_track_weight_for_a_fresh_pair_of_cpus() {
        let a = 20;
        let b = 21;
        let before = num_online_cpus();
        mark_possible(a);
        mark_possible(b);
        mark_online(a);
        mark_online(b);
        assert_eq!(num_online_cpus(), before + 2);
        mark_offline(a);
        mark_offline(b);
        assert_eq!(num_online_cpus(), before);
    }

    #[test]
    fn for_each_online_cpu_matches_online_mask() {
        let cpu = 30;
        mark_online(cpu);
        assert!(for_each_online_cpu().any(|c| c == cpu));
        mark_offline(cpu);
        assert!(!for_each_online_cpu().any(|c| c == cpu));
    }

    #[test]
    fn first_online_cpu_reports_a_currently_online_one() {
        let cpu = 40;
        mark_online(cpu);
        let first = first_online_cpu();
        assert!(first.is_some());
        assert!(is_online(first.unwrap()));
        mark_offline(cpu);
    }

    #[test]
    fn topology_reports_none_before_being_set() {
        assert_eq!(package_of(200), None);
        assert_eq!(core_of(200), None);
    }

    #[test]
    fn sibling_mask_groups_same_core_same_package() {
        set_topology(50, 0, 0);
        set_topology(51, 0, 0);
        set_topology(52, 0, 1);
        mark_online(50);
        mark_online(51);
        mark_online(52);

        let siblings = sibling_mask(50);
        assert!(siblings.is_set(50));
        assert!(siblings.is_set(51));
        assert!(!siblings.is_set(52));
        assert!(are_siblings(50, 51));
        assert!(!are_siblings(50, 52));

        mark_offline(50);
        mark_offline(51);
        mark_offline(52);
    }

    #[test]
    fn package_mask_covers_whole_socket() {
        set_topology(60, 1, 0);
        set_topology(61, 1, 1);
        set_topology(62, 2, 0);
        mark_online(60);
        mark_online(61);
        mark_online(62);

        let pkg = package_mask(60);
        assert!(pkg.is_set(60));
        assert!(pkg.is_set(61));
        assert!(!pkg.is_set(62));

        mark_offline(60);
        mark_offline(61);
        mark_offline(62);
    }

    #[test]
    fn sibling_mask_is_empty_without_topology_info() {
        assert!(sibling_mask(70).is_empty());
        assert!(package_mask(70).is_empty());
    }

    #[test]
    fn mask_from_iter_builds_expected_set() {
        let m = mask_from_iter([1u32, 5, 9]);
        assert_eq!(m.count(), 3);
        assert!(m.is_set(1) && m.is_set(5) && m.is_set(9));
        assert!(!m.is_set(2));
    }

    #[test]
    fn restrict_to_online_intersects_with_live_online_mask() {
        let cpu = 80;
        mark_online(cpu);
        let requested = mask_from_iter([cpu, 81, 82]);
        let effective = restrict_to_online(&requested);
        assert!(effective.is_set(cpu));
        assert!(!effective.is_set(81));
        assert!(!effective.is_set(82));
        mark_offline(cpu);
    }

    #[test]
    fn restrict_to_active_excludes_inactive_but_online_cpu() {
        let cpu = 90;
        mark_online(cpu);
        mark_inactive(cpu);
        let requested = mask_from_iter([cpu]);
        assert!(restrict_to_active(&requested).is_empty());
        assert!(restrict_to_online(&requested).is_set(cpu));
        mark_offline(cpu);
    }
}