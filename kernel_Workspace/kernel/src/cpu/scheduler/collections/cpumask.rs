#![allow(dead_code)]
#![allow(clippy::missing_safety_doc)]

//! Per-CPU membership masks (online/active/possible) and CPU topology
//! bookkeeping for the scheduler.
//!
//! `CpuMask` itself is defined in `entities::task`; this module only keeps
//! the global mask registry and topology lookup helpers on top of it.

pub use crate::cpu::scheduler::entities::task::{CpuMask, MAX_CPUS};

use core::sync::atomic::{AtomicU64, Ordering};

const WORDS: usize = MAX_CPUS / 64;

#[repr(C)]
pub struct AtomicCpuMask {
    bits: [AtomicU64; WORDS],
}

impl AtomicCpuMask {
    #[inline]
    pub const fn new() -> Self {
        const INIT: AtomicU64 = AtomicU64::new(0);
        Self { bits: [INIT; WORDS] }
    }

    #[inline]
    pub fn set(&self, cpu: u32) {
        let idx = cpu as usize;
        if idx < MAX_CPUS {
            self.bits[idx / 64].fetch_or(1u64 << (idx % 64), Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn clear(&self, cpu: u32) {
        let idx = cpu as usize;
        if idx < MAX_CPUS {
            self.bits[idx / 64].fetch_and(!(1u64 << (idx % 64)), Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn test(&self, cpu: u32) -> bool {
        let idx = cpu as usize;
        idx < MAX_CPUS && (self.bits[idx / 64].load(Ordering::Acquire) & (1u64 << (idx % 64))) != 0
    }

    #[inline]
    pub fn snapshot(&self) -> CpuMask {
        let mut m = CpuMask::empty();
        for cpu in 0..MAX_CPUS as u32 {
            if self.test(cpu) {
                m.set(cpu);
            }
        }
        m
    }
}

static POSSIBLE: AtomicCpuMask = AtomicCpuMask::new();
static PRESENT: AtomicCpuMask = AtomicCpuMask::new();
static ONLINE: AtomicCpuMask = AtomicCpuMask::new();
static ACTIVE: AtomicCpuMask = AtomicCpuMask::new();

const NO_TOPO: u32 = u32::MAX;
static PACKAGE_ID: [core::sync::atomic::AtomicU32; MAX_CPUS] =
    [const { core::sync::atomic::AtomicU32::new(NO_TOPO) }; MAX_CPUS];
static CORE_ID: [core::sync::atomic::AtomicU32; MAX_CPUS] =
    [const { core::sync::atomic::AtomicU32::new(NO_TOPO) }; MAX_CPUS];

#[inline]
pub fn mark_possible(cpu: u32) {
    POSSIBLE.set(cpu);
}

#[inline]
pub fn mark_present(cpu: u32) {
    PRESENT.set(cpu);
}

#[inline]
pub fn mark_online(cpu: u32) {
    ONLINE.set(cpu);
    ACTIVE.set(cpu);
}

#[inline]
pub fn mark_offline(cpu: u32) {
    ACTIVE.clear(cpu);
    ONLINE.clear(cpu);
}

#[inline]
pub fn is_online(cpu: u32) -> bool {
    ONLINE.test(cpu)
}

#[inline]
pub fn online_mask() -> CpuMask {
    ONLINE.snapshot()
}

#[inline]
pub fn active_mask() -> CpuMask {
    ACTIVE.snapshot()
}

#[inline]
pub fn set_topology(cpu: u32, pkg: u32, core: u32) {
    if (cpu as usize) < MAX_CPUS {
        PACKAGE_ID[cpu as usize].store(pkg, Ordering::Release);
        CORE_ID[cpu as usize].store(core, Ordering::Release);
    }
}

#[inline]
pub fn sibling_mask(cpu: u32) -> CpuMask {
    let mut out = CpuMask::empty();
    let pkg = PACKAGE_ID[cpu as usize].load(Ordering::Acquire);
    let core = CORE_ID[cpu as usize].load(Ordering::Acquire);
    if pkg == NO_TOPO || core == NO_TOPO {
        return out;
    }

    for c in online_mask().iter() {
        let c_idx = c as usize;
        if PACKAGE_ID[c_idx].load(Ordering::Acquire) == pkg && CORE_ID[c_idx].load(Ordering::Acquire) == core {
            out.set(c);
        }
    }
    out
}

#[inline]
pub fn package_mask(cpu: u32) -> CpuMask {
    let mut out = CpuMask::empty();
    let pkg = PACKAGE_ID[cpu as usize].load(Ordering::Acquire);
    if pkg == NO_TOPO {
        return out;
    }

    for c in online_mask().iter() {
        let c_idx = c as usize;
        if PACKAGE_ID[c_idx].load(Ordering::Acquire) == pkg {
            out.set(c);
        }
    }
    out
}