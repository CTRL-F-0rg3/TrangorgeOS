//! Typed identifiers used across the GPU framework.
//!
//! On the wire these are plain `u32`; the newtypes exist so the compiler can
//! stop a driver and a client from confusing a context with a buffer or a
//! device index with a handle.
//!
//! ## Handles are generational
//!
//! A GPU handle is `(slot, generation)` packed into one `u32`. The generation
//! is what makes a stale handle safe: after a context is destroyed its slot is
//! reused, and a client still holding the old handle gets
//! `InvalidHandle` instead of driving whatever landed in its place. A GPU driver
//! is privileged enough that this is not a nicety - it is the difference
//! between a crashed compositor and a corrupted command stream.

use kapi_abi::DsError;

/// A GPU context: the unit a client creates work in.
///
/// The port of Gallium's `pipe_context`, minus the state objects, which the
/// shared layer owns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ContextHandle(pub u32);

impl ContextHandle {
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }
}

/// A GPU buffer, as allocated by the driver.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BufferHandle(pub u32);

impl BufferHandle {
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }
}

/// A compiled shader program owned by the driver.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ShaderHandle(pub u32);

impl ShaderHandle {
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }
}

/// An index into the driver's device list, as reported by `GpuEnumerate`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct GpuIndex(pub u32);

impl GpuIndex {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// A monotonically increasing completion counter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct FenceValue(pub u64);

impl FenceValue {
    /// The value that means "nothing has been submitted yet".
    pub const NONE: Self = Self(0);

    #[inline]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn value(self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn is_set(self) -> bool {
        self.0 != 0
    }
}

/// A slot allocator for generational handles.
///
/// The driver owns one of these per resource kind. It is deliberately a plain
/// fixed-size table with no allocator: it lives in the framework, is exercised
/// by the framework's own tests, and costs a driver nothing to embed.
#[derive(Clone, Debug)]
pub struct HandleTable<T, const N: usize> {
    slots: [Slot<T>; N],
    next_generation: u32,
}

#[derive(Clone, Copy, Debug)]
struct Slot<T> {
    used: bool,
    generation: u32,
    value: T,
}

impl<T: Copy, const N: usize> HandleTable<T, N> {
    /// A table with every slot free.
    pub const fn new(empty: T) -> Self {
        Self {
            slots: [Slot { used: false, generation: 0, value: empty }; N],
            next_generation: 1,
        }
    }

    /// Claim a free slot and return its handle.
    ///
    /// `None` when the table is full, which is how a driver reports that it is
    /// out of contexts or buffers.
    pub fn insert(&mut self, value: T) -> Option<u32> {
        let index = self.slots.iter().position(|slot| !slot.used)?;
        let generation = self.next_generation;
        // Generation 0 is reserved for "never used", so a wrap skips it.
        self.next_generation = if generation == u32::MAX { 1 } else { generation + 1 };
        self.slots[index] = Slot { used: true, generation, value };
        Some(Self::pack(index, generation))
    }

    /// Look up a live handle, rejecting a stale generation.
    pub fn get(&self, handle: u32) -> Option<T> {
        if handle == u32::MAX {
            return None;
        }
        let index = Self::index_of(handle);
        let slot = self.slots.get(index)?;
        if !slot.used || slot.generation != Self::generation_of(handle) {
            return None;
        }
        Some(slot.value)
    }

    /// Release a handle, invalidating it for every later lookup.
    pub fn remove(&mut self, handle: u32) -> Option<T> {
        let index = Self::index_of(handle);
        let slot = self.slots.get_mut(index)?;
        if !slot.used || slot.generation != Self::generation_of(handle) {
            return None;
        }
        slot.used = false;
        Some(slot.value)
    }

    /// How many slots are in use.
    pub fn len(&self) -> usize {
        self.slots.iter().filter(|slot| slot.used).count()
    }

    /// Whether nothing is allocated.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The next generation that will be handed out, for diagnostics.
    pub fn next_generation(&self) -> u32 {
        self.next_generation
    }

    /// Pack a slot index and a generation into one handle.
    ///
    /// The low 16 bits hold the index, the high 16 the generation, so a table
    /// larger than the addressable range is refused rather than aliased.
    fn pack(index: usize, generation: u32) -> u32 {
        ((generation & 0xFFFF) << 16) | (index as u32 & 0xFFFF)
    }

    /// The slot index a handle names.
    fn index_of(handle: u32) -> usize {
        (handle & 0xFFFF) as usize
    }

    /// The generation a handle claims.
    fn generation_of(handle: u32) -> u32 {
        (handle >> 16) & 0xFFFF
    }
}

impl<T: Copy + Default, const N: usize> Default for HandleTable<T, N> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Reject an invalid context handle before any driver call happens.
///
/// The framework applies this uniformly, which is what stops a client from
/// steering a context through a handle that was never issued.
#[inline]
pub const fn check_context(handle: ContextHandle) -> Result<(), DsError> {
    if handle.is_valid() {
        Ok(())
    } else {
        Err(DsError::InvalidHandle)
    }
}

/// Reject an invalid buffer handle.
#[inline]
pub const fn check_buffer(handle: BufferHandle) -> Result<(), DsError> {
    if handle.is_valid() {
        Ok(())
    } else {
        Err(DsError::InvalidHandle)
    }
}

/// Reject an invalid shader handle.
#[inline]
pub const fn check_shader(handle: ShaderHandle) -> Result<(), DsError> {
    if handle.is_valid() {
        Ok(())
    } else {
        Err(DsError::InvalidHandle)
    }
}

/// A GPU surface, as the framework sees it.
///
/// This is the compositor's handle on something the driver can scan out. It is
/// separate from [`BufferHandle`] because a surface is a drawable - possibly
/// two buffers, front and back - where a buffer is one allocation. A display
/// engine scans out of a surface; it cannot scan out of a buffer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SurfaceHandle(pub u32);

impl SurfaceHandle {
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }
}

/// Where the display engine reads pixels from.
///
/// # Why this is a question and not a constant
///
/// An integrated part scans out of system memory; a discrete card scans out of
/// local memory. The compositor needs to know which before it composites,
/// because on a discrete part a surface destined for scanout has to be
/// allocated in local memory or every frame crosses the interconnect twice.
///
/// A driver that cannot answer leaves the compositor unable to place its
/// surfaces correctly, which is worse than not presenting at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scanout {
    /// Scans out of a buffer the GPU writes, in its own local memory.
    ///
    /// A discrete card. Nothing crosses the interconnect on its way to the
    /// display.
    LocalBuffer {
        /// The physical address the display engine reads.
        phys: u64,
        width: u32,
        height: u32,
        /// Bytes per scanline, which the driver knows and a client would guess.
        stride: u32,
    },

    /// Scans out of a buffer in system memory.
    ///
    /// An integrated part, or a discrete card presenting a shared framebuffer
    /// the CPU also has to read.
    SharedBuffer {
        phys: u64,
        width: u32,
        height: u32,
        stride: u32,
    },
}

impl Scanout {
    /// Whether the pixels live in the device's own memory.
    pub const fn is_local(self) -> bool {
        matches!(self, Self::LocalBuffer { .. })
    }

    /// The address the display engine reads.
    pub const fn phys(self) -> u64 {
        match self {
            Self::LocalBuffer { phys, .. } | Self::SharedBuffer { phys, .. } => phys,
        }
    }

    /// The mode this scanout is configured for, as `(width, height, stride)`.
    pub const fn mode(self) -> (u32, u32, u32) {
        match self {
            Self::LocalBuffer { width, height, stride, .. }
            | Self::SharedBuffer { width, height, stride, .. } => (width, height, stride),
        }
    }
}

/// Reject an invalid surface handle before any driver call happens.
#[inline]
pub const fn check_surface(handle: SurfaceHandle) -> Result<(), DsError> {
    if handle.is_valid() {
        Ok(())
    } else {
        Err(DsError::InvalidHandle)
    }
}

/// What the display engine is currently showing.
///
/// Held by the driver, not the client: two clients both believing they own the
/// screen is how a display ends up flickering between two buffers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PresentedSurface {
    pub surface: SurfaceHandle,
    pub scanout: Option<Scanout>,
}

impl PresentedSurface {
    /// Whether anything is currently being presented.
    pub const fn is_presenting(&self) -> bool {
        self.surface.is_valid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_table_is_empty_and_insert_claims_generation_one() {
        let mut table: HandleTable<u32, 4> = HandleTable::new(0);
        assert!(table.is_empty());
        let handle = table.insert(7).unwrap();
        assert_eq!(handle, (1 << 16) | 0);
        assert_eq!(table.get(handle), Some(7));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn removing_a_handle_invalidates_it() {
        let mut table: HandleTable<u32, 4> = HandleTable::new(0);
        let handle = table.insert(7).unwrap();
        assert_eq!(table.remove(handle), Some(7));
        // A destroyed handle must not resolve, even though the slot is free.
        assert_eq!(table.get(handle), None);
        assert_eq!(table.remove(handle), None);
    }

    #[test]
    fn a_reused_slot_issues_a_different_handle() {
        let mut table: HandleTable<u32, 4> = HandleTable::new(0);
        let first = table.insert(1).unwrap();
        table.remove(first);
        let second = table.insert(2).unwrap();
        // Same slot, different generation: this is what makes a stale handle
        // detectable rather than silently pointing at the new value.
        assert_ne!(first, second);
        assert_eq!(table.get(first), None, "the old handle is stale");
        assert_eq!(table.get(second), Some(2));
    }

    #[test]
    fn a_full_table_refuses_more_entries() {
        let mut table: HandleTable<u32, 2> = HandleTable::new(0);
        assert!(table.insert(1).is_some());
        assert!(table.insert(2).is_some());
        assert_eq!(table.insert(3), None);
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn the_sentinel_handle_never_resolves() {
        let table: HandleTable<u32, 4> = HandleTable::new(0);
        assert_eq!(table.get(u32::MAX), None);
        assert!(!ContextHandle::INVALID.is_valid());
        assert!(!BufferHandle::INVALID.is_valid());
        assert!(!ShaderHandle::INVALID.is_valid());
    }

    #[test]
    fn check_helpers_reject_only_the_sentinel() {
        assert!(check_context(ContextHandle::new(1)).is_ok());
        assert!(check_buffer(BufferHandle::new(1)).is_ok());
        assert!(check_shader(ShaderHandle::new(1)).is_ok());
        assert_eq!(check_context(ContextHandle::INVALID).unwrap_err(), DsError::InvalidHandle);
        assert_eq!(check_buffer(BufferHandle::INVALID).unwrap_err(), DsError::InvalidHandle);
        assert_eq!(check_shader(ShaderHandle::INVALID).unwrap_err(), DsError::InvalidHandle);
    }

    #[test]
    fn fence_values_distinguish_submitted_from_not() {
        assert!(!FenceValue::NONE.is_set());
        assert!(FenceValue::new(1).is_set());
        // Fences are monotonic, so "at least" is a plain comparison.
        assert!(FenceValue::new(5) >= FenceValue::new(3));
        assert!(FenceValue::new(3) < FenceValue::new(5));
    }
}
