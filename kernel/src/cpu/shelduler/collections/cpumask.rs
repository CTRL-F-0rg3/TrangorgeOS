//! CPU Mask (cpumask) implementation for the TrangorgeOS scheduler.
//!
//! Provides a robust, high-performance bitset for representing sets of CPUs.
//! Supports compile-time construction, O(K) iteration (where K is the number of set bits),
//! atomic operations, and topology-aware queries.

use core::fmt;
use core::iter::FusedIterator;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use core::sync::atomic::{AtomicU64, Ordering};

/// Maximum number of CPUs supported by the scheduler.
/// Must be a multiple of 64 for optimal `CpuMask` word alignment.
pub const MAX_CPUS: usize = 256;
pub const CPUMASK_WORDS: usize = MAX_CPUS / 64;

/// A bitmask representing a set of CPUs.
///
/// Optimized for fast bitwise operations and iteration.
/// Stored as an array of 64-bit words for cache efficiency.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CpuMask {
    bits: [u64; CPUMASK_WORDS],
}

impl CpuMask {
    /// Creates an empty CPU mask (no CPUs set).
    #[inline]
    pub const fn empty() -> Self {
        Self { bits: [0; CPUMASK_WORDS] }
    }

    /// Creates a CPU mask with all possible CPUs set.
    #[inline]
    pub const fn full() -> Self {
        Self { bits: [u64::MAX; CPUMASK_WORDS] }
    }

    /// Creates a CPU mask with a single CPU set.
    /// Silently ignores CPUs >= MAX_CPUS.
    #[inline]
    pub const fn single(cpu: u32) -> Self {
        let mut bits = [0u64; CPUMASK_WORDS];
        if (cpu as usize) < MAX_CPUS {
            bits[cpu as usize / 64] = 1u64 << (cpu as usize % 64);
        }
        Self { bits }
    }

    /// Creates a CPU mask with the first `n` CPUs set (0 to n-1).
    #[inline]
    pub const fn first_n(n: u32) -> Self {
        let mut bits = [0u64; CPUMASK_WORDS];
        let mut i = 0;
        while i < CPUMASK_WORDS {
            let word_start = i * 64;
            let word_end = word_start + 64;
            if (n as usize) <= word_start {
                bits[i] = 0;
            } else if (n as usize) >= word_end {
                bits[i] = u64::MAX;
            } else {
                let bits_to_set = (n as usize) - word_start;
                bits[i] = (1u64 << bits_to_set) - 1;
            }
            i += 1;
        }
        Self { bits }
    }

    /// Sets the specified CPU in the mask.
    #[inline]
    pub fn set(&mut self, cpu: u32) {
        let cpu = cpu as usize;
        if cpu < MAX_CPUS {
            self.bits[cpu / 64] |= 1u64 << (cpu % 64);
        }
    }

    /// Clears the specified CPU from the mask.
    #[inline]
    pub fn clear(&mut self, cpu: u32) {
        let cpu = cpu as usize;
        if cpu < MAX_CPUS {
            self.bits[cpu / 64] &= !(1u64 << (cpu % 64));
        }
    }

    /// Toggles the specified CPU in the mask.
    #[inline]
    pub fn toggle(&mut self, cpu: u32) {
        let cpu = cpu as usize;
        if cpu < MAX_CPUS {
            self.bits[cpu / 64] ^= 1u64 << (cpu % 64);
        }
    }

    /// Returns `true` if the specified CPU is set in the mask.
    #[inline]
    pub fn is_set(&self, cpu: u32) -> bool {
        let cpu = cpu as usize;
        cpu < MAX_CPUS && (self.bits[cpu / 64] & (1u64 << (cpu % 64))) != 0
    }

    /// Returns `true` if the mask contains no CPUs.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits.iter().all(|&w| w == 0)
    }

    /// Returns `true` if the mask contains all possible CPUs.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.bits.iter().all(|&w| w == u64::MAX)
    }

    /// Returns the number of CPUs set in the mask (Hamming weight).
    #[inline]
    pub fn weight(&self) -> u32 {
        self.bits.iter().map(|w| w.count_ones()).sum()
    }

    /// Returns `true` if this mask shares at least one CPU with `other`.
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.bits.iter().zip(other.bits.iter()).any(|(a, b)| (a & b) != 0)
    }

    /// Returns `true` if this mask is a subset of `other`.
    #[inline]
    pub fn is_subset(&self, other: &Self) -> bool {
        self.bits.iter().zip(other.bits.iter()).all(|(a, b)| (a & !b) == 0)
    }

    /// Returns a new mask containing CPUs that are in `self` but not in `other`.
    /// Crucial for scheduler affinity calculations (e.g., allowed & !online).
    #[inline]
    pub fn andnot(&self, other: &Self) -> Self {
        let mut bits = [0u64; CPUMASK_WORDS];
        for i in 0..CPUMASK_WORDS {
            bits[i] = self.bits[i] & !other.bits[i];
        }
        Self { bits }
    }

    /// Returns the first (lowest numbered) CPU set in the mask, or `None` if empty.
    #[inline]
    pub fn first(&self) -> Option<u32> {
        for (word_idx, &word) in self.bits.iter().enumerate() {
            if word != 0 {
                return Some((word_idx * 64 + word.trailing_zeros() as usize) as u32);
            }
        }
        None
    }

    /// Returns the next CPU set in the mask strictly after `cpu`.
    /// Wraps around to the beginning if necessary.
    #[inline]
    pub fn next_after(&self, cpu: u32) -> Option<u32> {
        let start = cpu.wrapping_add(1) as usize;
        
        // Check from `start` to `MAX_CPUS - 1`
        for word_idx in (start / 64)..CPUMASK_WORDS {
            let word = self.bits[word_idx];
            if word == 0 { continue; }
            
            let bit_offset = if word_idx * 64 == start { start % 64 } else { 0 };
            let masked_word = word & (!0u64 << bit_offset);
            
            if masked_word != 0 {
                return Some((word_idx * 64 + masked_word.trailing_zeros() as usize) as u32);
            }
        }
        
        // Wrap around: check from 0 to `cpu`
        for word_idx in 0..=(cpu as usize / 64) {
            let word = self.bits[word_idx];
            if word == 0 { continue; }
            
            let masked_word = if word_idx == (cpu as usize / 64) {
                word & (!0u64 << ((cpu as usize % 64) + 1))
            } else {
                word
            };
            
            if masked_word != 0 {
                return Some((word_idx * 64 + masked_word.trailing_zeros() as usize) as u32);
            }
        }
        None
    }

    /// Returns an iterator over the CPUs set in the mask.
    /// Iteration is O(K) where K is the number of set bits, not O(N).
    #[inline]
    pub fn iter(&self) -> CpuMaskIter<'_> {
        CpuMaskIter {
            mask: self,
            current_word_idx: 0,
            current_word: self.bits[0],
        }
    }
}

// --- Bitwise Operators ---

impl BitAnd for CpuMask {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        let mut bits = [0u64; CPUMASK_WORDS];
        for i in 0..CPUMASK_WORDS { bits[i] = self.bits[i] & rhs.bits[i]; }
        Self { bits }
    }
}

impl BitAndAssign for CpuMask {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        for i in 0..CPUMASK_WORDS { self.bits[i] &= rhs.bits[i]; }
    }
}

impl BitOr for CpuMask {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        let mut bits = [0u64; CPUMASK_WORDS];
        for i in 0..CPUMASK_WORDS { bits[i] = self.bits[i] | rhs.bits[i]; }
        Self { bits }
    }
}

impl BitOrAssign for CpuMask {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        for i in 0..CPUMASK_WORDS { self.bits[i] |= rhs.bits[i]; }
    }
}

impl BitXor for CpuMask {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        let mut bits = [0u64; CPUMASK_WORDS];
        for i in 0..CPUMASK_WORDS { bits[i] = self.bits[i] ^ rhs.bits[i]; }
        Self { bits }
    }
}

impl BitXorAssign for CpuMask {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        for i in 0..CPUMASK_WORDS { self.bits[i] ^= rhs.bits[i]; }
    }
}

impl Not for CpuMask {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        let mut bits = [0u64; CPUMASK_WORDS];
        for i in 0..CPUMASK_WORDS { bits[i] = !self.bits[i]; }
        Self { bits }
    }
}

// --- Iterator ---

/// An iterator over the CPUs set in a `CpuMask`.
/// Uses Brian Kernighan's algorithm for O(1) per-set-bit advancement.
pub struct CpuMaskIter<'a> {
    mask: &'a CpuMask,
    current_word_idx: usize,
    current_word: u64,
}

impl<'a> Iterator for CpuMaskIter<'a> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        while self.current_word_idx < CPUMASK_WORDS {
            if self.current_word != 0 {
                let bit = self.current_word.trailing_zeros() as usize;
                // Clear the lowest set bit (Brian Kernighan's algorithm)
                self.current_word &= self.current_word - 1;
                return Some((self.current_word_idx * 64 + bit) as u32);
            }
            self.current_word_idx += 1;
            if self.current_word_idx < CPUMASK_WORDS {
                self.current_word = self.mask.bits[self.current_word_idx];
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.mask.weight() as usize;
        (remaining, Some(remaining))
    }
}

impl<'a> FusedIterator for CpuMaskIter<'a> {}

impl Default for CpuMask {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

// --- Formatting ---

impl fmt::Debug for CpuMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CpuMask({:#018x}...)", self.bits[0])
    }
}

impl fmt::Display for CpuMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "<none>");
        }
        
        let mut first = true;
        let mut in_range = false;
        let mut range_start = 0u32;
        let mut last_cpu = 0u32;

        for cpu in self.iter() {
            if first {
                range_start = cpu;
                last_cpu = cpu;
                first = false;
                in_range = true;
            } else if cpu == last_cpu + 1 {
                last_cpu = cpu;
            } else {
                if in_range {
                    if range_start == last_cpu {
                        write!(f, "{}", range_start)?;
                    } else {
                        write!(f, "{}-{}", range_start, last_cpu)?;
                    }
                    in_range = false;
                }
                write!(f, ",")?;
                range_start = cpu;
                last_cpu = cpu;
                in_range = true;
            }
        }
        
        // Flush the last range
        if in_range {
            if range_start == last_cpu {
                write!(f, "{}", range_start)?;
            } else {
                write!(f, "{}-{}", range_start, last_cpu)?;
            }
        }
        Ok(())
    }
}

// --- Atomic CPU Mask ---

/// An atomic version of `CpuMask` for lock-free concurrent updates.
/// Essential for global CPU state (online/offline) or concurrent affinity updates.
#[repr(C)]
pub struct AtomicCpuMask {
    bits: [AtomicU64; CPUMASK_WORDS],
}

impl AtomicCpuMask {
    /// Creates a new `AtomicCpuMask` with no CPUs set.
    #[inline]
    pub const fn new() -> Self {
        Self {
            bits: [const { AtomicU64::new(0) }; CPUMASK_WORDS],
        }
    }

    /// Sets the specified CPU in the mask atomically.
    #[inline]
    pub fn set(&self, cpu: u32) {
        let cpu = cpu as usize;
        if cpu < MAX_CPUS {
            let word_idx = cpu / 64;
            let bit = 1u64 << (cpu % 64);
            self.bits[word_idx].fetch_or(bit, Ordering::Relaxed);
        }
    }

    /// Clears the specified CPU from the mask atomically.
    #[inline]
    pub fn clear(&self, cpu: u32) {
        let cpu = cpu as usize;
        if cpu < MAX_CPUS {
            let word_idx = cpu / 64;
            let bit = !(1u64 << (cpu % 64));
            self.bits[word_idx].fetch_and(bit, Ordering::Relaxed);
        }
    }

    /// Atomically sets the CPU and returns `true` if it was previously clear.
    #[inline]
    pub fn test_and_set(&self, cpu: u32) -> bool {
        let cpu = cpu as usize;
        if cpu < MAX_CPUS {
            let word_idx = cpu / 64;
            let bit = 1u64 << (cpu % 64);
            let prev = self.bits[word_idx].fetch_or(bit, Ordering::AcqRel);
            return (prev & bit) == 0;
        }
        false
    }

    /// Atomically clears the CPU and returns `true` if it was previously set.
    #[inline]
    pub fn test_and_clear(&self, cpu: u32) -> bool {
        let cpu = cpu as usize;
        if cpu < MAX_CPUS {
            let word_idx = cpu / 64;
            let bit = 1u64 << (cpu % 64);
            let prev = self.bits[word_idx].fetch_and(!bit, Ordering::AcqRel);
            return (prev & bit) != 0;
        }
        false
    }

    /// Returns `true` if the specified CPU is set.
    #[inline]
    pub fn test(&self, cpu: u32) -> bool {
        let cpu = cpu as usize;
        if cpu < MAX_CPUS {
            let word_idx = cpu / 64;
            let bit = 1u64 << (cpu % 64);
            (self.bits[word_idx].load(Ordering::Acquire) & bit) != 0
        } else {
            false
        }
    }

    /// Returns the number of CPUs currently set in the mask.
    #[inline]
    pub fn weight(&self) -> u32 {
        self.bits.iter().map(|w| w.load(Ordering::Relaxed).count_ones()).sum()
    }

    /// Takes a snapshot of the current mask as a regular `CpuMask`.
    #[inline]
    pub fn snapshot(&self) -> CpuMask {
        let mut bits = [0u64; CPUMASK_WORDS];
        for (i, atomic_word) in self.bits.iter().enumerate() {
            bits[i] = atomic_word.load(Ordering::Relaxed);
        }
        CpuMask { bits }
    }
}

impl Default for AtomicCpuMask {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// --- Global CPU Topology State ---

static POSSIBLE_CPUS: AtomicCpuMask = AtomicCpuMask::new();
static PRESENT_CPUS: AtomicCpuMask = AtomicCpuMask::new();
static ONLINE_CPUS: AtomicCpuMask = AtomicCpuMask::new();
static ACTIVE_CPUS: AtomicCpuMask = AtomicCpuMask::new();

const NO_TOPOLOGY: u32 = u32::MAX;
static PACKAGE_ID: [core::sync::atomic::AtomicU32; MAX_CPUS] = [const { core::sync::atomic::AtomicU32::new(NO_TOPOLOGY) }; MAX_CPUS];
static CORE_ID: [core::sync::atomic::AtomicU32; MAX_CPUS] = [const { core::sync::atomic::AtomicU32::new(NO_TOPOLOGY) }; MAX_CPUS];

// --- Lifecycle Management ---

#[inline]
pub fn mark_possible(cpu: u32) { POSSIBLE_CPUS.set(cpu); }

#[inline]
pub fn mark_present(cpu: u32) { PRESENT_CPUS.set(cpu); }

#[inline]
pub fn mark_online(cpu: u32) {
    ONLINE_CPUS.set(cpu);
    ACTIVE_CPUS.set(cpu);
}

#[inline]
pub fn mark_offline(cpu: u32) {
    ACTIVE_CPUS.clear(cpu);
    ONLINE_CPUS.clear(cpu);
}

#[inline]
pub fn mark_active(cpu: u32) { ACTIVE_CPUS.set(cpu); }

#[inline]
pub fn mark_inactive(cpu: u32) { ACTIVE_CPUS.clear(cpu); }

#[inline]
pub fn is_possible(cpu: u32) -> bool { (cpu as usize) < MAX_CPUS && POSSIBLE_CPUS.test(cpu) }

#[inline]
pub fn is_present(cpu: u32) -> bool { (cpu as usize) < MAX_CPUS && PRESENT_CPUS.test(cpu) }

#[inline]
pub fn is_online(cpu: u32) -> bool { (cpu as usize) < MAX_CPUS && ONLINE_CPUS.test(cpu) }

#[inline]
pub fn is_active(cpu: u32) -> bool { (cpu as usize) < MAX_CPUS && ACTIVE_CPUS.test(cpu) }

#[inline]
pub fn num_possible_cpus() -> u32 { POSSIBLE_CPUS.weight() }

#[inline]
pub fn num_present_cpus() -> u32 { PRESENT_CPUS.weight() }

#[inline]
pub fn num_online_cpus() -> u32 { ONLINE_CPUS.weight() }

#[inline]
pub fn num_active_cpus() -> u32 { ACTIVE_CPUS.weight() }

#[inline]
pub fn possible_mask() -> CpuMask { POSSIBLE_CPUS.snapshot() }

#[inline]
pub fn present_mask() -> CpuMask { PRESENT_CPUS.snapshot() }

#[inline]
pub fn online_mask() -> CpuMask { ONLINE_CPUS.snapshot() }

#[inline]
pub fn active_mask() -> CpuMask { ACTIVE_CPUS.snapshot() }

// --- Topology Queries ---

#[inline]
pub fn set_topology(cpu: u32, package_id: u32, core_id: u32) {
    if (cpu as usize) < MAX_CPUS {
        PACKAGE_ID[cpu as usize].store(package_id, Ordering::Release);
        CORE_ID[cpu as usize].store(core_id, Ordering::Release);
    }
}

#[inline]
pub fn package_of(cpu: u32) -> Option<u32> {
    if (cpu as usize) >= MAX_CPUS { return None; }
    let id = PACKAGE_ID[cpu as usize].load(Ordering::Acquire);
    if id == NO_TOPOLOGY { None } else { Some(id) }
}

#[inline]
pub fn core_of(cpu: u32) -> Option<u32> {
    if (cpu as usize) >= MAX_CPUS { return None; }
    let id = CORE_ID[cpu as usize].load(Ordering::Acquire);
    if id == NO_TOPOLOGY { None } else { Some(id) }
}

/// Returns a mask of all logical CPUs that share the same physical core (SMT siblings).
/// Always includes the `cpu` itself, if it is online.
#[inline]
pub fn sibling_mask(cpu: u32) -> CpuMask {
    let mut out = CpuMask::empty();
    let pkg = match package_of(cpu) { Some(p) => p, None => return out };
    let core = match core_of(cpu) { Some(c) => c, None => return out };
    
    for c in 0..MAX_CPUS as u32 {
        if is_online(c) && package_of(c) == Some(pkg) && core_of(c) == Some(core) {
            out.set(c);
        }
    }
    out
}

/// Returns a mask of all logical CPUs in the same physical package (socket).
#[inline]
pub fn package_mask(cpu: u32) -> CpuMask {
    let mut out = CpuMask::empty();
    let pkg = match package_of(cpu) { Some(p) => p, None => return out };
    
    for c in 0..MAX_CPUS as u32 {
        if is_online(c) && package_of(c) == Some(pkg) {
            out.set(c);
        }
    }
    out
}

#[inline]
pub fn are_siblings(a: u32, b: u32) -> bool {
    a != b
        && package_of(a).is_some()
        && package_of(a) == package_of(b)
        && core_of(a) == core_of(b)
}

// --- Helper Constructors ---

#[inline]
pub fn mask_from_iter<I: IntoIterator<Item = u32>>(cpus: I) -> CpuMask {
    let mut m = CpuMask::empty();
    for c in cpus { m.set(c); }
    m
}

#[inline]
pub fn restrict_to_online(mask: &CpuMask) -> CpuMask {
    *mask & online_mask()
}

#[inline]
pub fn restrict_to_active(mask: &CpuMask) -> CpuMask {
    *mask & active_mask()
}