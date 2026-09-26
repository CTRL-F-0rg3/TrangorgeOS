//! Typed identifiers and buffer bookkeeping for the video framework.
//!
//! On the wire these are plain `u32`; the newtypes exist so the compiler can stop
//! a driver and a caller from confusing an engine index with a session handle,
//! or a surface with a ticket. The ticket is the subtle one: it is what a caller
//! holds between submitting a frame and collecting it, and conflating it with
//! the buffer handle makes a decode loop release the wrong surface.

use kapi_abi::{DsError, payloads::video::{FrameFormat, Fourcc}};

/// An index into the driver's engine list.
///
/// Not a handle: engines are enumerated at start-up and live as long as the
/// driver does, so an index is stable and needs no generation tag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct EngineId(pub u32);

impl EngineId {
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl From<u32> for EngineId {
    #[inline]
    fn from(v: u32) -> Self {
        Self(v)
    }
}

/// An open decoding or encoding session.
///
/// Generation-tagged exactly like [`crate::sink::SinkId`], and for the same
/// reason: a caller that keeps a handle past `destroy` must not find a *new*
/// session in its place and start writing frames into it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct StreamHandle(pub u32);

impl StreamHandle {
    /// The "not a handle" value.
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

    /// The slot in the driver's session table.
    #[inline]
    pub const fn slot(self) -> usize {
        (self.0 & 0xff) as usize
    }

    /// The generation of the session occupying that slot.
    #[inline]
    pub const fn generation(self) -> u32 {
        self.0 >> 8
    }

    #[inline]
    pub const fn make(slot: usize, generation: u32) -> Self {
        Self(((generation & 0x00ff_ffff) << 8) | (slot as u32 & 0xff))
    }
}

impl From<u32> for StreamHandle {
    #[inline]
    fn from(v: u32) -> Self {
        Self(v)
    }
}

/// A surface the driver owns, handed to a consumer and returned afterwards.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct BufferId(pub u32);

impl BufferId {
    /// The "not a buffer" value.
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

impl From<u32> for BufferId {
    #[inline]
    fn from(v: u32) -> Self {
        Self(v)
    }
}

/// Reject a handle the service does not own, before any driver call happens.
///
/// The framework applies this uniformly, which is what stops a client from
/// steering a session through a handle that was never issued.
#[inline]
pub const fn check_handle(handle: StreamHandle) -> Result<(), DsError> {
    if handle.is_valid() {
        Ok(())
    } else {
        Err(DsError::InvalidHandle)
    }
}

/// Reject a buffer id that is not a real one.
#[inline]
pub const fn check_buffer(buffer: BufferId) -> Result<(), DsError> {
    if buffer.is_valid() {
        Ok(())
    } else {
        Err(DsError::InvalidHandle)
    }
}

/// How many surfaces one session may own.
///
/// A hard bound, and a small one: 4K NV12 is about 12 MB, so sixteen slots is
/// nearly 200 MB of device memory for a stream that may need three. Exceeding
/// the pool is reported rather than grown, because a pool that grows on demand
/// grows until the machine runs out.
pub const MAX_POOL_SLOTS: usize = 16;

/// A fixed-size surface pool for one session.
///
/// # Why the driver owns the memory
///
/// A consumer receives a [`BufferId`], never an address. The surface belongs to
/// the driver for the whole life of the pool and is recycled when released, so
/// the address a consumer sees for frame 1 is the same one for frame 500. That
/// is what makes the zero-copy path possible: nobody maps or unmaps anything
/// per frame, and a consumer can hold a pointer across an arbitrary number of
/// later frames.
///
/// The cost is that a consumer which holds a frame forever stalls the pool, and
/// eventually the decode. [`BufferPool::available`] is the number a driver
/// watches for exactly that reason.
pub struct BufferPool {
    slots: [Slot; MAX_POOL_SLOTS],
    live: u32,
}

/// One surface slot.
struct Slot {
    id: BufferId,
    /// The layout currently in this slot. Valid only while `in_use` is set.
    format: FrameFormat,
    /// Is a consumer currently holding this surface?
    in_use: bool,
    /// The ticket of the frame this slot holds, for matching an acquire to a
    /// submit.
    ticket: u32,
}

impl Slot {
    /// A slot that has never been allocated.
    const EMPTY: Self = Self {
        id: BufferId::INVALID,
        format: FrameFormat { format: Fourcc(0), stride: 0, width: 0, height: 0 },
        in_use: false,
        ticket: 0,
    };
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

impl BufferPool {
    /// An empty pool with no surfaces allocated.
    #[inline]
    pub const fn new() -> Self {
        Self { slots: [Slot::EMPTY; MAX_POOL_SLOTS], live: 0 }
    }

    /// How many surfaces are allocated.
    #[inline]
    pub const fn allocated(&self) -> u32 {
        self.live
    }

    /// How many are currently held by a consumer.
    #[inline]
    pub fn available(&self) -> u32 {
        self.slots.iter().filter(|s| s.in_use).count() as u32
    }

    /// How many are allocated and free.
    #[inline]
    pub fn free(&self) -> u32 {
        self.allocated() - self.available()
    }

    /// Is the pool exhausted?
    #[inline]
    pub const fn is_exhausted(&self) -> bool {
        self.live as usize >= MAX_POOL_SLOTS
    }

    /// Claim a surface for a new frame.
    ///
    /// Returns `None` when the pool is full. That is *backpressure*, not a
    /// failure: the caller drops the frame, counts it, and carries on. A
    /// seventeenth surface would mean either unbounded memory growth or evicting
    /// a frame somebody is still holding.
    pub fn acquire(&mut self, ticket: u32, format: FrameFormat) -> Option<BufferId> {
        // Reuse a free surface if there is one. Recycling is what keeps a
        // surface's address stable across frames, which is exactly what a
        // zero-copy consumer relies on when it holds a pointer.
        let index = match self.slots.iter().position(|s| s.in_use) {
            Some(i) => i,
            None if self.is_exhausted() => return None,
            None => (0..MAX_POOL_SLOTS).find(|i| self.slots[*i].id == BufferId::INVALID)?,
        };
        if self.slots[index].id == BufferId::INVALID {
            self.live += 1;
        }
        self.slots[index].id = BufferId::new(index as u32);
        self.slots[index].format = format;
        self.slots[index].ticket = ticket;
        self.slots[index].in_use = true;
        Some(self.slots[index].id)
    }

    /// Give a surface back to the pool.
    pub fn release(&mut self, id: BufferId) -> Result<(), DsError> {
        let slot = self
            .slots
            .iter_mut()
            .find(|s| s.in_use && s.id == id)
            .ok_or(DsError::InvalidHandle)?;
        slot.in_use = false;
        Ok(())
    }

    /// The layout of a held surface.
    pub fn format_of(&self, id: BufferId) -> Option<FrameFormat> {
        self.slots.iter().find(|s| s.in_use && s.id == id).map(|s| s.format)
    }

    /// The ticket of a held surface.
    pub fn ticket_of(&self, id: BufferId) -> Option<u32> {
        self.slots.iter().find(|s| s.in_use && s.id == id).map(|s| s.ticket)
    }

    /// Release every surface. Used when a session is torn down.
    pub fn release_all(&mut self) {
        for slot in &mut self.slots {
            slot.in_use = false;

#[cfg(test)]
mod tests {
    use super::*;

    fn nv12() -> FrameFormat {
        FrameFormat::nv12(1920, 1080)
    }

    #[test]
    fn handles_distinguish_valid_from_invalid() {
        assert!(StreamHandle::new(0).is_valid());
        assert!(!StreamHandle::INVALID.is_valid());
        assert!(BufferId::new(3).is_valid());
        assert!(!BufferId::INVALID.is_valid());
        assert_eq!(EngineId::from(2u32).index(), 2);
    }

    #[test]
    fn the_sentinels_are_rejected_before_any_driver_call() {
        assert_eq!(check_handle(StreamHandle::INVALID).unwrap_err(), DsError::InvalidHandle);
        assert_eq!(check_buffer(BufferId::INVALID).unwrap_err(), DsError::InvalidHandle);
        assert!(check_handle(StreamHandle::new(1)).is_ok());
        assert!(check_buffer(BufferId::new(1)).is_ok());
    }

    #[test]
    fn a_pool_hands_out_a_fresh_surface_per_acquire() {
        let mut p = BufferPool::new();
        let a = p.acquire(1, nv12()).expect("first");
        let b = p.acquire(2, nv12()).expect("second");
        assert_ne!(a, b, "two frames in flight need two surfaces");
        assert_eq!(p.available(), 2);
        assert_eq!(p.allocated(), 2);
    }

    #[test]
    fn a_released_surface_is_reused_rather_than_grown() {
        // The point of the pool: a consumer that releases lets the driver
        // recycle, so the address it saw stays valid.
        let mut p = BufferPool::new();
        let a = p.acquire(1, nv12()).unwrap();
        p.release(a).unwrap();
        assert_eq!(p.available(), 0);
        assert_eq!(p.allocated(), 1, "releasing does not free the surface");

        let b = p.acquire(2, nv12()).unwrap();
        assert_eq!(a, b, "the same surface comes back, so a held pointer stays good");
        assert_eq!(p.allocated(), 1);
    }

    #[test]
    fn a_full_pool_reports_backpressure_rather_than_growing() {
        let mut p = BufferPool::new();
        let mut held = [BufferId::INVALID; MAX_POOL_SLOTS];
        for (i, slot) in held.iter_mut().enumerate() {
            *slot = p.acquire(i as u32, nv12()).expect("within the pool");
        }
        assert!(p.is_exhausted());
        assert_eq!(p.acquire(999, nv12()), None, "a seventeenth surface is refused");
        assert_eq!(p.allocated() as usize, MAX_POOL_SLOTS, "and nothing was allocated");

        p.release(held[0]).unwrap();
        assert!(p.acquire(999, nv12()).is_some());
    }

    #[test]
    fn a_surface_keeps_its_format_and_ticket_until_released() {
        let mut p = BufferPool::new();
        let id = p.acquire(42, nv12()).unwrap();
        assert_eq!(p.ticket_of(id), Some(42));
        assert_eq!(p.format_of(id).unwrap().width, 1920);
        assert_eq!(p.format_of(id).unwrap().height, 1080);

        p.release(id).unwrap();
        assert_eq!(p.ticket_of(id), None, "a released surface is not a held frame");
    }

    #[test]
    fn releasing_a_surface_that_is_not_held_is_an_error_not_a_silent_no_op() {
        let mut p = BufferPool::new();
        assert_eq!(p.release(BufferId::new(0)).unwrap_err(), DsError::InvalidHandle);
        let id = p.acquire(1, nv12()).unwrap();
        p.release(id).unwrap();
        assert_eq!(
            p.release(id).unwrap_err(),
            DsError::InvalidHandle,
            "a double release would hand one surface to two consumers"
        );
    }

    #[test]
    fn tearing_down_a_session_releases_everything_at_once() {
        let mut p = BufferPool::new();
        p.acquire(1, nv12()).unwrap();
        p.acquire(2, nv12()).unwrap();
        assert_eq!(p.available(), 2);
        p.release_all();
        assert_eq!(p.available(), 0);
    }

    #[test]
    fn session_handles_carry_a_generation() {
        let h = StreamHandle::make(3, 7);
        assert_eq!(h.slot(), 3);
        assert_eq!(h.generation(), 7);
        assert_ne!(h, StreamHandle::make(3, 8), "a new session in the same slot differs");
    }
}

        }
    }
}

