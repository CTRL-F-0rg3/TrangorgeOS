//! Receive and transmit rings, and who owns recycling a buffer.
//!
//! # The ownership rule
//!
//! A buffer is owned by exactly one party at a time, and the ring tracks which.
//! On receive, the *card* owns a buffer until the driver takes the frame; the
//! *stack* owns it until the driver recycles it. Two parties touching it at once
//! is a data race that presents as a corrupted frame rather than a crash, which
//! is the worst kind of bug to have on the receive path.
//!
//! [`RxRing`] makes the state explicit rather than implicit in an index, so the
//! mistake is a rejected call and not a silent overwrite.
//!
//! # Deltas, not indices
//!
//! Rings are stored as a head and a tail, and [`RxRing::available`] is
//! `head - tail` rather than a count. That is what lets a card DMA a whole
//! batch and update the head once: the driver never walks the ring, and the
//! memory the card writes is never read by anyone else.

use crate::error::NetworkError;

/// A buffer index inside a ring.
///
/// Wrapped, not a bare `usize`, so a card-supplied index cannot be confused with
/// a driver-owned one. The wire format for both is `u16`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct BufferId(pub u16);

impl BufferId {
    #[inline]
    pub fn new(index: u16) -> Self {
        Self(index)
    }

    #[inline]
    pub fn index(self) -> u16 {
        self.0
    }
}

/// Who may touch a receive buffer right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    /// The card is DMAing into it. The driver must not read it.
    Device,
    /// The driver has taken the frame; the stack has it.
    Stack,
    /// Nobody; the buffer is free for the card to refill.
    Free,
}

/// A fixed-size receive ring.
#[derive(Debug)]
pub struct RxRing<const N: usize> {
    /// Next index the driver will look at.
    tail: u16,
    /// Next index the card will fill.
    head: u16,
    owner: [Owner; N],
    /// Buffers dropped because the stack did not recycle in time.
    drops: u64,
}

impl<const N: usize> RxRing<N> {
    /// A ring of `N` buffers, all free.
    pub fn new() -> Self {
        Self { tail: 0, head: 0, owner: [Owner::Free; N], drops: 0 }
    }

    /// How many frames are waiting.
    #[inline]
    pub fn available(&self) -> u16 {
        self.head.wrapping_sub(self.tail)
    }

    /// Is there room for another frame?
    #[inline]
    pub fn has_room(&self) -> bool {
        self.available() < N as u16
    }

    /// How many frames have been dropped.
    #[inline]
    pub fn drops(&self) -> u64 {
        self.drops
    }

    /// Hand the card one buffer to fill, returning its id.
    ///
    /// Refused when the ring is full, which is the backpressure signal: the
    /// caller counts a drop and moves on. It must not grow the ring - a ring
    /// that grows on demand grows until memory runs out, and the packets that
    /// mattered are the ones that got dropped on the way.
    pub fn give_to_device(&mut self) -> Result<BufferId, NetworkError> {
        if !self.has_room() {
            self.drops += 1;
            return Err(NetworkError::QueueFull);
        }
        // The slot after the head is what the device will fill next.
        let slot = self.head % N as u16;
        let id = BufferId::new(slot);
        if self.owner[slot as usize] != Owner::Free {
            // A buffer still held by the stack. Overwriting it is the data race
            // this type exists to prevent, so it is refused, not tolerated.
            self.drops += 1;
            return Err(NetworkError::BadDescriptor);
        }
        self.owner[slot as usize] = Owner::Device;
        self.head = self.head.wrapping_add(1);
        Ok(id)
    }

    /// Take the next finished frame, giving it to the stack.
    pub fn take(&mut self) -> Option<BufferId> {
        if self.available() == 0 {
            return None;
        }
        let slot = self.tail % N as u16;
        if self.owner[slot as usize] != Owner::Device {
            // The device never filled this one. Skipping it would desynchronise
            // the ring; stopping is honest, and the next interrupt will retry.
            return None;
        }
        self.owner[slot as usize] = Owner::Stack;
        self.tail = self.tail.wrapping_add(1);
        Some(BufferId::new(slot))
    }

    /// Give a buffer back for the device to refill.
    pub fn recycle(&mut self, id: BufferId) -> Result<(), NetworkError> {
        let slot = id.index() as usize % N;
        match self.owner.get(slot) {
            Some(Owner::Stack) => {
                self.owner[slot] = Owner::Free;
                Ok(())
            }
            _ => Err(NetworkError::BadDescriptor),
        }
    }

    /// Who owns buffer `id`?
    pub fn owner_of(&self, id: BufferId) -> Option<Owner> {
        self.owner.get(id.index() as usize % N).copied()
    }
}

impl<const N: usize> Default for RxRing<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// A fixed-size transmit ring.
///
/// Simpler than the receive side on purpose: a transmit buffer is written once
/// and the card owns it until completion, so there is no second owner and no
/// race to prevent. The interesting part is the completion accounting, which
/// tells a caller whether its frame went out.
#[derive(Debug)]
pub struct TxRing<const N: usize> {
    /// Next index the driver will submit.
    tail: u16,
    /// Next index the card will complete.
    head: u16,
    /// Submissions whose completion has not been collected.
    pending: u16,
    /// Frames the card refused or that errored out.
    errors: u64,
}

impl<const N: usize> TxRing<N> {
    /// A ring of `N` slots, all free.
    pub fn new() -> Self {
        Self { tail: 0, head: 0, pending: 0, errors: 0 }
    }

    /// How many frames are in flight.
    #[inline]
    pub fn in_flight(&self) -> u16 {
        self.pending
    }

    /// Is there room to submit?
    #[inline]
    pub fn has_room(&self) -> bool {
        self.pending < N as u16
    }

    /// How many transmissions failed.
    #[inline]
    pub fn errors(&self) -> u64 {
        self.errors
    }

    /// Claim a slot for a frame about to be written.
    pub fn submit(&mut self) -> Result<BufferId, NetworkError> {
        if !self.has_room() {
            return Err(NetworkError::QueueFull);
        }
        let id = BufferId::new(self.tail % N as u16);
        self.tail = self.tail.wrapping_add(1);
        self.pending += 1;
        Ok(id)
    }

    /// Note that the card finished one frame.
    pub fn complete(&mut self) -> Result<BufferId, NetworkError> {
        if self.pending == 0 {
            return Err(NetworkError::BadDescriptor);
        }
        let id = BufferId::new(self.head % N as u16);
        self.head = self.head.wrapping_add(1);
        self.pending -= 1;
        Ok(id)
    }

    /// Note that the card failed one frame.
    pub fn fail(&mut self) -> Result<BufferId, NetworkError> {
        let id = self.complete()?;
        self.errors += 1;
        Ok(id)
    }
}

impl<const N: usize> Default for TxRing<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_buffer_travels_device_then_stack_then_free() {
        // The ownership cycle, which is the whole point of the type.
        let mut r = RxRing::<4>::new();
        let id = r.give_to_device().expect("room");
        assert_eq!(r.owner_of(id), Some(Owner::Device));
        assert_eq!(r.available(), 1);

        let taken = r.take().expect("a frame is ready");
        assert_eq!(taken, id);
        assert_eq!(r.owner_of(id), Some(Owner::Stack));

        r.recycle(id).expect("the stack gave it back");
        assert_eq!(r.owner_of(id), Some(Owner::Free));
    }

    #[test]
    fn a_full_ring_refuses_rather_than_overwriting() {
        // The refusal is the safety property: an overwrite here is a data race
        // that presents as a corrupted frame, not a crash.
        let mut r = RxRing::<2>::new();
        r.give_to_device().unwrap();
        r.give_to_device().unwrap();
        assert!(!r.has_room());
        assert_eq!(r.give_to_device().unwrap_err(), NetworkError::QueueFull);
        assert_eq!(r.drops(), 1, "and it is counted, not hidden");
    }

    #[test]
    fn a_held_buffer_is_never_handed_out_again() {
        let mut r = RxRing::<1>::new();
        let id = r.give_to_device().unwrap();
        assert_eq!(r.take().unwrap(), id);
        // The stack still holds it. The ring's *count* says there is room - the
        // frame was already taken - but the buffer is not free, and the refusal
        // is on ownership rather than on fullness. That distinction is the
        // point: a driver that treated this as `QueueFull` would report a
        // capacity problem for what is a lifetime problem.
        assert!(r.has_room(), "the ring's accounting has already moved on");
        assert_eq!(
            r.give_to_device().unwrap_err(),
            NetworkError::BadDescriptor,
            "but the buffer must not be given out twice"
        );
        assert_eq!(r.owner_of(id), Some(Owner::Stack), "and it is still the stack's");
    }

    #[test]
    fn taking_from_an_empty_ring_yields_nothing() {
        let mut r = RxRing::<4>::new();
        assert!(r.take().is_none());
    }

    #[test]
    fn recycling_twice_is_refused() {
        // A double recycle would hand the same buffer to the card twice.
        let mut r = RxRing::<2>::new();
        let id = r.give_to_device().unwrap();
        r.take().unwrap();
        r.recycle(id).unwrap();
        assert_eq!(r.recycle(id).unwrap_err(), NetworkError::BadDescriptor);
    }

    #[test]
    fn recycling_something_never_issued_is_refused() {
        let mut r = RxRing::<4>::new();
        assert_eq!(r.recycle(BufferId::new(3)).unwrap_err(), NetworkError::BadDescriptor);
    }

    #[test]
    fn a_ring_wraps_without_losing_its_accounting() {
        // Head and tail are u16 and the ring is small, so wrapping is normal
        // operation rather than an edge case.
        let mut r = RxRing::<4>::new();
        for _ in 0..64 {
            let id = r.give_to_device().expect("room");
            r.take().expect("ready");
            r.recycle(id).expect("returned");
        }
        assert_eq!(r.available(), 0, "the ring must end up empty, not part-full");
        assert_eq!(r.drops(), 0, "and must not have dropped anything on the way");
    }

    #[test]
    fn tx_accounts_in_flight_and_completions() {
        let mut t = TxRing::<2>::new();
        assert!(t.has_room());
        t.submit().unwrap();
        t.submit().unwrap();
        assert_eq!(t.in_flight(), 2);
        assert!(!t.has_room());
        assert_eq!(t.submit().unwrap_err(), NetworkError::QueueFull);

        t.complete().unwrap();
        assert_eq!(t.in_flight(), 1);
        assert!(t.has_room());
    }

    #[test]
    fn a_tx_completion_with_nothing_in_flight_is_refused() {
        let mut t = TxRing::<2>::new();
        assert_eq!(t.complete().unwrap_err(), NetworkError::BadDescriptor);
    }

    #[test]
    fn a_tx_failure_is_counted_separately_from_a_completion() {
        // An errored frame is not a delivered one, and conflating them is how a
        // driver reports 100% success on a link that drops everything.
        let mut t = TxRing::<2>::new();
        t.submit().unwrap();
        t.fail().unwrap();
        assert_eq!(t.errors(), 1);
        assert_eq!(t.in_flight(), 0);
    }

    #[test]
    fn tx_slots_complete_in_submission_order() {
        let mut t = TxRing::<4>::new();
        let a = t.submit().unwrap();
        let b = t.submit().unwrap();
        assert_eq!(t.complete().unwrap(), a);
        assert_eq!(t.complete().unwrap(), b);
    }
}

