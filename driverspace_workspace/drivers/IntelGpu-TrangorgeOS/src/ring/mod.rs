//! A command ring, and the fence that goes with it.
//!
//! # What a ring is
//!
//! A ring is a circular buffer in pinned memory that the GPU reads commands
//! from. The CPU writes, the GPU reads, and both need to know where each has
//! got to: `tail` is what the CPU has written, `head` is what the GPU has
//! consumed. Neither may pass the other, and the wrap is handled by the size
//! being a power of two so the mask is the wrap.
//!
//! # Why the ring size is a power of two
//!
//! Because the full/empty test has to be unambiguous. With a non-power-of-two
//! size, "is the ring full" is either `tail + 1 == head` or "one byte wasted",
//! and getting it wrong is a ring the GPU overruns. A power of two plus a
//! reserved byte makes it exact.
//!
//! # Why Intel needs a guard band
//!
//! Intel's command processors read in 8-byte units and will happily read a
//! few bytes past the end of a command. A submission that ends exactly at the
//! end of the ring would therefore have the GPU executing whatever the CPU
//! wrote next. The tail is held back by [`GUARD_BYTES`] so that cannot happen.

/// The power-of-two size a ring is rounded up to.
pub const fn ring_size_class(requested: u64) -> u64 {
    // A ring below 16 KiB cannot hold a batch plus a guard band; a ring above
    // 16 MiB is larger than any Intel command stream and only costs TLB
    // pressure.
    const MIN: u64 = 16 * 1024;
    const MAX: u64 = 16 * 1024 * 1024;

    let clamped = if requested < MIN { MIN } else if requested > MAX { MAX } else { requested };
    clamped.next_power_of_two()
}

/// Bytes at the end of a ring the CPU must not write into.
///
/// Intel's front ends fetch command words in aligned units and may prefetch
/// past the last one, so the tail is held back by this much.
pub const GUARD_BYTES: u64 = 4096;

/// Where the CPU has written to, and where the GPU has read to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RingState {
    tail: u64,
    head: u64,
    size: u64,
    /// Set when the GPU has stopped without signalling, which is a hang.
    hung: bool,
}

impl RingState {
    /// A ring of `size` bytes, both ends at zero.
    pub const fn new(size: u64) -> Self {
        Self { tail: 0, head: 0, size, hung: false }
    }

    /// The ring's size, always a power of two.
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// The usable capacity: the size less the guard band.
    pub const fn usable(&self) -> u64 {
        self.size - GUARD_BYTES
    }

    /// Where the CPU has written.
    pub const fn tail(&self) -> u64 {
        self.tail
    }

    /// Where the GPU has read to.
    pub const fn head(&self) -> u64 {
        self.head
    }

    /// Bytes the GPU has yet to execute.
    pub const fn pending(&self) -> u64 {
        self.tail.wrapping_sub(self.head) & (self.size - 1)
    }

    /// Bytes free for the CPU to write.
    pub const fn free(&self) -> u64 {
        self.usable().wrapping_sub(self.pending())
    }

    /// Whether the ring has hung.
    ///
    /// "Hung" means the head has not moved while the tail has. Intel signals
    /// this through a separate error bit rather than by the head stalling, so
    /// this is the driver's judgement, not a register read.
    pub const fn is_hung(&self) -> bool {
        self.hung
    }

    /// Record that the GPU has consumed up to `head`.
    ///
    /// A `head` that moves backwards, or ahead of the tail, is ignored: a
    /// device reporting an impossible position is a device to be reset, not
    /// one to be believed.
    pub fn advance_head(&mut self, head: u64) -> bool {
        let pending = self.tail.wrapping_sub(self.head) & (self.size - 1);
        if head == self.head {
            return true;
        }
        // A backwards head would mean the GPU unwound work it already did.
        if head.wrapping_sub(self.head) & (self.size - 1) > pending {
            return false;
        }
        self.head = head;
        // A ring that has been full and is now draining is not hung.
        if self.head != self.tail {
            self.hung = false;
        }
        true
    }

    /// Write `length` bytes at `offset`, refusing to pass the guard band.
    ///
    /// `offset` is a ring-relative offset, which is what a client names in
    /// `GpuSubmit`.
    pub fn write(&mut self, offset: u64, length: u64) -> Result<u64, RingError> {
        if length == 0 {
            return Err(RingError::EmptySubmission);
        }
        if offset >= self.size {
            return Err(RingError::OutOfRange);
        }
        let end = offset.checked_add(length).ok_or(RingError::OutOfRange)?;
        if end > self.usable() {
            // Past the guard band. Allowing this is how a submission ends up
            // executing the next batch's bytes.
            return Err(RingError::PastGuardBand);
        }
        self.tail = end & (self.size - 1);
        Ok(self.tail)
    }

    /// Mark the ring as hung, so a wait can give up instead of spinning.
    pub const fn set_hung(&mut self, hung: bool) {
        self.hung = hung;
    }
}

/// Why a ring operation was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingError {
    /// A zero-length submission names no work.
    EmptySubmission,
    /// The offset lies outside the ring.
    OutOfRange,
    /// The write would run into the guard band at the end of the ring.
    PastGuardBand,
}
