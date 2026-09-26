//! The split virtqueue, ported from `include/uapi/linux/virtio_ring.h`.
//!
//! # What this is
//!
//! A split virtqueue is one contiguous block of memory holding three structures
//! back to back, and *two* parties advancing indices into it: the driver and the
//! device. Each side publishes a monotonically increasing 16-bit counter and
//! reads the other's. The counters wrap at 65536, not at the queue size, and
//! every comparison is therefore a wrapping comparison.
//!
//! ```text
//!   ┌──────────────────────────────────────────────────┐
//!   │ desc[num]       16 bytes each, the buffers       │
//!   ├──────────────────────────────────────────────────┤
//!   │ avail.flags     u16                             │
//!   │ avail.idx       u16  ← driver publishes         │
//!   │ avail.ring[num] u16  descriptor heads           │
//!   │ used_event      u16                             │
//!   │ ─── pad to alignment ───                         │
//!   ├──────────────────────────────────────────────────┤
//!   │ used.flags      u16                             │
//!   │ used.idx        u16  ← device publishes         │
//!   │ used.ring[num]  {id, len}, 8 bytes each         │
//!   │ avail_event     u16                             │
//!   └──────────────────────────────────────────────────┘
//! ```
//!
//! # Why the sizes are written out longhand
//!
//! `desc` is 16 bytes, the `avail` preamble is 2+2 bytes, each `used` element is
//! *eight* bytes (`id` and `len`), and then the whole thing is padded to a page
//! boundary. Getting any of that wrong does not fail loudly: the device reads
//! the descriptor table, finds a different number of free buffers than it
//! expected, and traffic simply stops. So [`layout_size`] reproduces
//! `vring_size()` from the spec and the tests check it against hand-computed
//! byte counts.
//!
//! # Memory ordering
//!
//! The two publishing stores are the entire synchronisation contract, and it is
//! easy to get backwards:
//!
//! - **Driver publishing** a descriptor for the device: write the descriptor
//!   *first*, then the index. The release on the index is what makes the
//!   descriptor visible before the device can see it indexed.
//! - **Device publishing** a completed descriptor for the driver: the buffer was
//!   filled by the device, so that store needs a release too, or the driver can
//!   observe the new index against a stale buffer.
//!
//! Getting this wrong is the classic virtio bug: the driver reads a descriptor
//! the device has not finished writing, and hands the stack a plausible mixture
//! of two packets. [`fence`] exists here so that store is not forgotten.
//!
//! # Index arithmetic
//!
//! `new.wrapping_sub(old)` is what tracks outstanding work, and it is only
//! meaningful while the ring holds fewer than `2^15` items.
//! [`validate_size`] enforces that bound at construction rather than leaving it
//! as an assumption.

use super::spec::{u16_le, VRING_DESC_F_NEXT, VRING_DESC_F_WRITE};
use core::sync::atomic::{fence, Ordering};

/// `sizeof(struct vring_desc)`: address, length, flags, next.
pub const DESC_SIZE: usize = 16;
/// `sizeof(struct vring_used_elem)`: a `u32` id *and* a `u32` length.
///
/// Eight bytes, not four. Getting this wrong shrinks `vring_size()` by `4 * num`,
/// so the device maps a ring that ends before the last used entry and the
/// completions for the tail of every batch land outside the mapping.
pub const USED_ELEM_SIZE: usize = 8;
/// The `avail` preamble: `flags` then `idx`.
pub const AVAIL_HEADER_SIZE: usize = 4;
/// The `used` preamble: `flags` then `idx`.
pub const USED_HEADER_SIZE: usize = 4;
/// A trailing `used_event` or `avail_event` word.
pub const EVENT_IDX_SIZE: usize = 2;
/// The split ring's required alignment.
pub const RING_ALIGN: usize = 4096;

/// A queue deeper than this cannot use 16-bit wrapping arithmetic safely.
pub const MAX_QUEUE_SIZE: usize = 1 << 15;

/// A little-endian `u16`, for byte order as part of the type.
pub use super::spec::u16_le as LeU16;

/// Where a queue's structures live inside its block of memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RingLayout {
    /// Number of descriptors.
    pub num: u16,
    /// Byte offset of `desc`.
    pub desc_offset: usize,
    /// Byte offset of `avail`.
    pub avail_offset: usize,
    /// Byte offset of `used`, after page alignment.
    pub used_offset: usize,
    /// Total bytes the device must be able to map.
    pub total_size: usize,
}

/// Something went wrong building or indexing a queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingError {
    /// The requested size was zero.
    ZeroSize,
    /// The requested size exceeds [`MAX_QUEUE_SIZE`].
    TooLarge,
    /// The requested size is not a power of two, which the ring requires.
    NotPowerOfTwo,
    /// A descriptor chain ran off the end of the table.
    BadChain,
    /// The device returned a descriptor identifier that is out of range.
    BadDescriptorId,
}

impl core::fmt::Display for RingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            RingError::ZeroSize => "queue size is zero",
            RingError::TooLarge => "queue size exceeds 2^15",
            RingError::NotPowerOfTwo => "queue size is not a power of two",
            RingError::BadChain => "descriptor chain runs off the table",
            RingError::BadDescriptorId => "device returned an out-of-range descriptor",
        })
    }
}

/// `vring_size()` from the spec: total bytes for a split ring.
///
/// ```text
///   ((desc_size*num + 2*(3+num) + align-1) & ~(align-1)) + 2*3 + elem_size*num
/// ```
///
/// The three `2`s are the `avail` header, the `used` header, and one trailing
/// event word; the second event word is already inside the `3 + num` term.
pub const fn layout_size(num: u16, align: usize) -> usize {
    let n = num as usize;
    let head = DESC_SIZE * n + 2 * (3 + n);
    let aligned = (head + align - 1) & !(align - 1);
    aligned + 2 * 3 + USED_ELEM_SIZE * n
}

/// Check that `num` is usable as a split queue depth.
///
/// The power-of-two requirement is not cosmetic: the index arithmetic in
/// `index_delta` only stays unambiguous while the ring is at most half full, and
/// a non-power-of-two size makes the device's own masking of descriptor indices
/// disagree with ours.
pub const fn validate_size(num: u16) -> Result<(), RingError> {
    if num == 0 {
        return Err(RingError::ZeroSize);
    }
    if num as usize > MAX_QUEUE_SIZE {
        return Err(RingError::TooLarge);
    }
    if !num.is_power_of_two() {
        return Err(RingError::NotPowerOfTwo);
    }
    Ok(())
}

/// The byte offsets of each structure, for a given size and alignment.
pub const fn layout(num: u16, align: usize) -> RingLayout {
    let n = num as usize;
    let avail = DESC_SIZE * n;
    // `used` begins after the `avail` preamble, its ring, and the `used_event`
    // word, rounded up to the alignment.
    let used = (avail + AVAIL_HEADER_SIZE + 2 * n + EVENT_IDX_SIZE + align - 1) & !(align - 1);
    RingLayout {
        num,
        desc_offset: 0,
        avail_offset: avail,
        used_offset: used,
        total_size: layout_size(num, align),
    }
}

/// One entry of the descriptor table, as the driver writes it.
///
/// `#[repr(C)]` fixes the layout and the little-endian wrappers fix the byte
/// order, because the device reads these bytes directly out of this memory over
/// DMA rather than parsing them.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Descriptor {
    /// Guest physical address of the buffer.
    pub addr: u64,
    /// Buffer length.
    pub len: u32,
    /// A mask of `VRING_DESC_F_*`.
    pub flags: u16_le,
    /// Index of the next descriptor in the chain, if `VRING_DESC_F_NEXT`.
    pub next: u16_le,
}

impl Descriptor {
    /// A descriptor the device *reads*: our transmit path.
    #[inline]
    pub const fn read(addr: u64, len: u32) -> Self {
        Self {
            addr,
            len,
            flags: u16_le::new(0),
            next: u16_le::new(0),
        }
    }

    /// A descriptor the device *writes*: our receive path.
    ///
    /// Getting this flag backwards on an RX descriptor is not a subtle
    /// malfunction, it is the device writing frame data into the descriptor table
    /// itself.
    #[inline]
    pub const fn write(addr: u64, len: u32) -> Self {
        Self {
            addr,
            len,
            flags: u16_le::new(VRING_DESC_F_WRITE),
            next: u16_le::new(0),
        }
    }

    /// Is this the device-writes direction?
    #[inline]
    pub const fn is_write(&self) -> bool {
        self.flags.get() & VRING_DESC_F_WRITE != 0
    }

    /// Does this descriptor continue into `next`?
    #[inline]
    pub const fn has_next(&self) -> bool {
        self.flags.get() & VRING_DESC_F_NEXT != 0
    }
}

/// A little-endian `u32`, for the same reason as [`u16_le`].
///
/// The name is deliberately not camel case: it is a wire type, and reading
/// `u32_le` at a use site is what tells you the byte order is part of the
/// contract. A `u32` here would be indistinguishable from a host integer and
/// would be the single easiest way to get endianness wrong.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct u32_le(pub u32);

impl u32_le {
    /// The value as a host integer.
    #[inline]
    pub const fn get(self) -> u32 {
        u32::from_le(self.0)
    }
    /// Build from a host integer.
    #[inline]
    pub const fn new(v: u32) -> Self {
        Self(v.to_le())
    }
}

/// One entry of the used ring: the device's completion record.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct UsedElem {
    /// Index of the head descriptor of the chain.
    pub id: u32_le,
    /// Bytes written into the buffer. On RX this is the frame length.
    pub len: u32_le,
}

/// The wrapping distance from `old` to `new` on a 16-bit ring index.
///
/// The only correct way to ask "how far ahead is the device?", and it relies on
/// the ring never holding `2^15` or more outstanding items. Past that the
/// subtraction wraps silently and the driver concludes the device went
/// backwards, which presents as a permanent "no free buffers".
#[inline]
pub const fn index_delta(new: u16, old: u16) -> u16 {
    new.wrapping_sub(old)
}

/// `vring_need_event()` from the spec: should we notify?
///
/// Catches the off-by-one that makes a driver notify on every single packet
/// instead of only once the device's threshold has genuinely been crossed — the
/// difference between one interrupt per packet and one per batch.
#[inline]
pub const fn need_event(event_idx: u16, new_idx: u16, old_idx: u16) -> bool {
    new_idx.wrapping_sub(event_idx).wrapping_sub(1) < new_idx.wrapping_sub(old_idx)
}

/// The driver's view of a split virtqueue.
///
/// Generic over the backing store `B` so that the index logic can be tested
/// against an ordinary `Vec` on the host and against physically contiguous
/// frames in the kernel. `B` is anything with `as_mut_ptr() -> *mut u8` and
/// `B` is anything that can hand out a raw pointer to its first byte, such as
/// `Vec<u8>` on the host or a page-aligned DMA frame in the kernel.
pub struct SplitQueue<B> {
    base: B,
    layout: RingLayout,
    /// The driver's own `avail.idx`: how far we have published.
    avail_idx: u16,
    /// The device's `used.idx`: how far the device has completed.
    used_idx: u16,
    /// The device's `avail.flags`, read once at setup.
    avail_flags: u16,
    /// Our `used.flags`, written once at setup.
    used_flags: u16,
    /// The device's `used_event` threshold.
    used_event: u16,
    /// The device's `avail_event` threshold.
    avail_event: u16,
}

impl<B: AsMut<[u8]> + AsRef<[u8]>> SplitQueue<B> {
    /// Take a zeroed backing region and lay the ring out inside it.
    ///
    /// The caller is responsible for the backing being page-aligned and, in the
    /// kernel, physically contiguous and DMA-mappable; `validate_size` guards the
    /// one property that is cheap to check and expensive to debug.
    pub fn new(mut backing: B, num: u16) -> Result<Self, RingError> {
        validate_size(num)?;
        let layout = layout(num, RING_ALIGN);
        if backing.as_mut().len() < layout.total_size {
            return Err(RingError::TooLarge);
        }
        backing.as_mut()[..layout.total_size].fill(0);
        Ok(Self {
            base: backing,
            layout,
            avail_idx: 0,
            used_idx: 0,
            avail_flags: 0,
            used_flags: 0,
            used_event: 0,
            avail_event: 0,
        })
    }

    /// The queue's layout, for the caller that must tell the device where to map.
    pub fn layout(&self) -> RingLayout {
        self.layout
    }

    /// How many descriptors this queue has.
    pub fn num(&self) -> u16 {
        self.layout.num
    }

    /// Descriptors the device has not completed yet.
    pub fn outstanding(&self) -> u16 {
        index_delta(self.avail_idx, self.used_idx)
    }

    /// Descriptors we could still hand out.
    pub fn available(&self) -> u16 {
        self.layout.num - self.outstanding()
    }

    // ── raw field access ────────────────────────────────────────────────────
    //
    // The ring is shared memory that the device reads and writes concurrently,
    // so every access to a field the *other* side owns is volatile, and every
    // access to an index the other side may advance is read through an atomic
    // load. These helpers are the only place that happens, so the rule is stated
    // once instead of being re-derived at each of the dozen call sites.

    /// Read a little-endian `u16` from a ring offset.
    ///
    /// # Safety
    /// `off` and `off + 2` must lie inside the ring.
    #[inline]
    unsafe fn read_u16(&self, off: usize) -> u16 {
        u16::from_le(unsafe { core::ptr::read_volatile(self.raw(off).cast::<u16>()) })
    }

    /// Write a little-endian `u16` into a ring offset.
    ///
    /// # Safety
    /// `off` and `off + 2` must lie inside the ring.
    #[inline]
    unsafe fn write_u16(&self, off: usize, v: u16) {
        unsafe { core::ptr::write_volatile(self.raw(off).cast::<u16>(), v.to_le()) }
    }

    /// A pointer to byte `off` of the ring.
    ///
    /// Every caller passes an absolute offset from the matching `*_off` helper,
    /// so this adds exactly once.
    #[inline]
    fn raw(&self, off: usize) -> *mut u8 {
        self.base.as_ref().as_ptr().wrapping_add(off) as *mut u8
    }

    /// Byte offset of `avail.flags`.
    const fn avail_flags_off(&self) -> usize {
        self.layout.avail_offset
    }
    /// Byte offset of `avail.idx`.

    // ── the driver's half: handing descriptors to the device ────────────────

    /// Write one descriptor into the table.
    ///
    /// # Safety
    /// `index` must be less than `num()`.
    pub unsafe fn set_descriptor(&mut self, index: u16, desc: &Descriptor) {
        let off = self.desc_off(index);
        let p = self.raw(off);
        unsafe {
            core::ptr::write_volatile(p.cast::<u64>(), desc.addr.to_le());
            core::ptr::write_volatile(p.add(8).cast::<u32>(), desc.len.to_le());
            core::ptr::write_volatile(p.add(12).cast::<u16>(), desc.flags.raw());
            core::ptr::write_volatile(p.add(14).cast::<u16>(), desc.next.raw());
        }
    }

    /// Publish one descriptor to the device.
    ///
    /// This is the store that makes the previous [`Self::set_descriptor`] call
    /// visible, and the ordering is the whole point: a release fence before the
    /// index store, so that no compiler reordering can let the device observe the
    /// new `avail.idx` against a descriptor whose contents are still in flight.
    ///
    /// # Safety
    /// The descriptor at `index` must already have been written.
    pub unsafe fn publish(&mut self, index: u16) {
        let slot = self.avail_idx;
        let pos = self.avail_ring_off(slot % self.layout.num);
        let next = slot.wrapping_add(1);

        unsafe {
            // The device must see the ring entry, and the new index, in that
            // order. A release fence on the index is the documented contract.
            core::ptr::write_volatile(self.raw(pos).cast::<u16>(), index.to_le());
            fence(Ordering::Release);
            self.write_u16(self.avail_idx_off(), next);
            fence(Ordering::Release);
        }
        self.avail_idx = next;
    }

    /// Write the ring's two flag words.
    ///
    /// Split because the two bits live in different structures and mean opposite
    /// things: `avail.flags` is *our* right to stay quiet, `used.flags` is the
    /// *device's*. Setting both from one call is what makes "interrupt on every
    /// packet" and "never interrupt" a one-line difference rather than a
    /// reasoning exercise.
    pub fn set_no_interrupt(&mut self) {
        self.used_flags |= 1; // VRING_USED_F_NO_NOTIFY
        self.avail_flags |= 1; // VRING_AVAIL_F_NO_INTERRUPT
        unsafe {
            self.write_u16(self.avail_flags_off(), self.avail_flags);
            self.write_u16(self.used_flags_off(), self.used_flags);
        }
    }

    /// Should the driver notify after publishing up to `new_idx`?
    ///
    /// Honours both the `NO_NOTIFY` flag the device may set and the
    /// `used_event` threshold, which is what turns a per-packet interrupt into a
    /// per-batch one.
    pub fn should_notify(&self, old_idx: u16, new_idx: u16) -> bool {
        if self.avail_flags & 1 != 0 {
            return false;
        }
        need_event(self.avail_event, new_idx, old_idx)
    }

    /// Should the driver kick the device after publishing up to `new_idx`?
    ///
    /// Uses `used_event`, the threshold the *device* published in the available
    /// ring, and `VRING_USED_F_NO_NOTIFY`. This is the other half of
    /// [`Self::should_notify`]: that one asks whether the device should interrupt
    /// us, this one asks whether we should interrupt the device.
    pub fn should_kick(&self, old_idx: u16, new_idx: u16) -> bool {
        if self.used_flags & 1 != 0 {
            return false;
        }
        need_event(self.used_event, new_idx, old_idx)
    }

    // ── the device's half: collecting completions ──────────────────────────

    /// Read the device's published `used.idx` without consuming anything.
    ///
    /// Note what this deliberately does *not* do: assign to `self.used_idx`.
    /// `used_idx` is the driver's own cursor — how far *we* have consumed — and
    /// conflating it with the device's index makes `next_completion` compute its
    /// slot from the device's counter. The result is a driver that returns the
    /// wrong used-ring entry: plausible ids, zero lengths, no error. The two
    /// counters only coincide on an idle ring, which is exactly why it survives a
    /// smoke test and fails on traffic.
    pub fn device_used_idx(&self) -> u16 {
        unsafe { self.read_u16(self.used_idx_off()) }
    }

    /// Take the next completed descriptor, if the device has published one.
    ///
    /// Returns `(descriptor index, bytes written)`. The length is the device's,
    /// not ours: on RX it is authoritative and must be used as-is, because
    /// trusting the buffer's full length instead is how trailing garbage gets
    /// handed to the IP stack as payload.
    pub fn next_completion(&mut self) -> Option<(u16, u32)> {
        if self.used_idx == self.device_used_idx() {
            return None;
        }
        // The slot comes from *our* cursor, never the device's: the entry we are
        // retiring is the one the device wrote `used_idx` entries ago.
        let slot = self.used_idx % self.layout.num;
        let off = self.used_ring_off(slot);
        let id = u32::from_le(unsafe { core::ptr::read_volatile(self.raw(off).cast::<u32>()) });
        let len = u32::from_le(unsafe { core::ptr::read_volatile(self.raw(off + 4).cast::<u32>()) });
        if id as u16 >= self.layout.num {
            // A device that names a descriptor we do not have is either corrupt
            // or has been reset underneath us. Ignoring the entry keeps the ring
            // draining; the caller sees it as a shorter batch.
            return None;
        }
        self.used_idx = self.used_idx.wrapping_add(1);
        Some((id as u16, len))
    }

    /// Drain every completion the device has published, handing each to `sink`.
    ///
    /// The loop is bounded by `num` on purpose. A device that keeps advancing
    /// `used.idx` while we drain cannot make us loop forever here, and a driver
    /// that trusts the device to stop is a driver that can be wedged by a faulty
    /// or hostile one.
    pub fn drain_completions<F: FnMut(u16, u32)>(&mut self, mut sink: F) -> usize {
        let mut n = 0;
        while n < self.layout.num as usize {
            match self.next_completion() {
                Some((id, len)) => {
                    sink(id, len);
                    n += 1;
                }
                None => break,
            }
        }
        n
    }

    /// Tell the device how far we have consumed its completions.
    pub fn release_used(&mut self) {
        // A store-release so the device cannot observe the advanced index before
        // it has finished reading the entries we are retiring.
        fence(Ordering::Release);
        unsafe { self.write_u16(self.used_idx_off(), self.used_idx) };
    }

    /// Read the event thresholds the device published.
    ///
    /// The two words sit at opposite ends of the ring, which is why they are easy
    /// to confuse: `used_event` is the last entry of the *available* ring, and
    /// `avail_event` the one just past the *used* ring. `virtio_ring.h` places
    /// them at the end "for backwards compatibility", and swapping them produces a
    /// driver that suppresses interrupts almost to the point of hanging rather
    /// than an outright crash.
    pub fn read_events(&mut self) {
        self.used_event = unsafe { self.read_u16(self.avail_ring_off(self.layout.num)) };
        self.avail_event = unsafe { self.read_u16(self.used_ring_off(self.layout.num)) };
    }

    // ── offsets, derived once from the layout ──────────────────────────────

    /// Byte offset of `avail.idx`.
    const fn avail_idx_off(&self) -> usize {
        self.layout.avail_offset + 2
    }
    /// Byte offset of `avail.ring[i]`.
    const fn avail_ring_off(&self, i: u16) -> usize {
        self.layout.avail_offset + AVAIL_HEADER_SIZE + 2 * i as usize
    }
    /// Byte offset of `used.flags`.
    const fn used_flags_off(&self) -> usize {
        self.layout.used_offset
    }
    /// Byte offset of `used.idx`.
    const fn used_idx_off(&self) -> usize {
        self.layout.used_offset + 2
    }
    /// Byte offset of `used.ring[i]`.
    const fn used_ring_off(&self, i: u16) -> usize {
        self.layout.used_offset + USED_HEADER_SIZE + USED_ELEM_SIZE * i as usize
    }
    /// Byte offset of `desc[i]`.
    const fn desc_off(&self, i: u16) -> usize {
        self.layout.desc_offset + DESC_SIZE * i as usize
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    /// A ring of `num` descriptors over a heap buffer, as the host can provide.
    fn queue(num: u16) -> SplitQueue<Vec<u8>> {
        let l = layout(num, RING_ALIGN);
        SplitQueue::new(vec![0u8; l.total_size], num).expect("valid queue")
    }

    /// Play the device's side: write `count` completions starting at `from`.
    ///
    /// This is the half of the protocol the kernel does not own, so a test that
    /// A ring of `num` descriptors over a heap buffer, as the host can provide.

    /// only exercises [`SplitQueue::publish`] merely proves the driver can talk to
    /// itself. Writing the device's half by hand is what makes the completion
    /// path real.
    /// Play the device's side: write `count` completions starting at `from`.
    ///
    /// `from` is the running `used.idx`; the `id` stored in each entry is the
    /// *descriptor* index, i.e. the running index modulo the queue depth, which
    /// is what the spec says and what the driver's bounds check expects. Writing
    /// the raw counter here instead would be rejected as out of range once the
    /// counter passed `num` — a good illustration of why that check exists.
    fn device_complete(q: &mut SplitQueue<Vec<u8>>, from: u16, count: u16, len: u32) {
        for k in 0..count {
            let slot = (from + k) % q.num();
            let off = q.used_ring_off(slot);
            unsafe {
                let id = u32::from(slot);
                core::ptr::write_volatile(q.raw(off).cast::<u32>(), id.to_le());
                core::ptr::write_volatile(q.raw(off + 4).cast::<u32>(), len.to_le());
            }
        }
        unsafe {
            q.write_u16(q.used_idx_off(), from.wrapping_add(count));
        }
    }

    // ── sizes: the part that fails silently on hardware ────────────────────

    #[test]
    fn ring_size_matches_the_spec() {
        // Hand-computed for num=256, align=4096:
        //   desc    256*16                = 4096
        //   avail   4 + 2*256 + 2 used_ev = 518, so the head runs to 4614
        //   pad     4614 rounds up to      = 8192
        //   used    6 + 256*8              = 2054
        //   total                          = 10246
        assert_eq!(layout_size(256, RING_ALIGN), 8192 + 6 + 256 * 8);
    }

    #[test]
    fn ring_size_grows_with_depth() {
        // Exact `vring_size()` values. Both 8 and 16 descriptors fit inside the
        // first page, so growing between them costs only the used ring: the head
        // term is page-padded and does not move at all. That is the arithmetic
        // that is easy to get wrong by hand, so the numbers are written out.
        assert_eq!(layout_size(8, RING_ALIGN), 4096 + 6 + 8 * 8);
        assert_eq!(layout_size(16, RING_ALIGN), 4096 + 6 + 8 * 16);
        // 256 crosses into the second page: 4096 desc + 518 avail, padded to 8192.
        assert_eq!(layout_size(256, RING_ALIGN), 8192 + 6 + 8 * 256);
        // 512 crosses into the third: 8192 desc + 1030 avail, padded to 12288.
        assert_eq!(layout_size(512, RING_ALIGN), 12288 + 6 + 8 * 512);
    }

    #[test]
    fn used_starts_on_a_page_boundary() {
        // The device maps this ring by page; a `used` offset that is not page
        // aligned means the mapping is one page short and the device reads
        // descriptors that are not there.
        for num in [8u16, 64, 128, 256, 512, 1024] {
            assert_eq!(layout(num, RING_ALIGN).used_offset % RING_ALIGN, 0, "num={num}");
        }
    }

    #[test]
    fn structures_do_not_overlap() {
        for num in [8u16, 64, 256, 1024] {
            let l = layout(num, RING_ALIGN);
            let avail_end = l.avail_offset + AVAIL_HEADER_SIZE + 2 * num as usize;
            assert!(l.desc_offset + DESC_SIZE * num as usize <= l.avail_offset);
            assert!(avail_end <= l.used_offset, "avail overruns used at num={num}");
            assert!(
                l.used_offset + USED_HEADER_SIZE + USED_ELEM_SIZE * num as usize <= l.total_size,
                "used overruns the ring at num={num}"
            );
        }
    }

    #[test]
    fn desc_is_sixteen_bytes_and_used_elem_eight() {
        assert_eq!(core::mem::size_of::<Descriptor>(), DESC_SIZE);
        // `vring_used_elem` is `{ u32 id; u32 len }`. Assuming it is 4 bytes
        // silently under-allocates the ring by `4 * num` bytes.
        assert_eq!(core::mem::size_of::<UsedElem>(), USED_ELEM_SIZE);
    }

    // ── size validation ─────────────────────────────────────────────────────

    #[test]
    fn valid_sizes_are_accepted() {
        for n in [1u16, 2, 8, 128, 256, 1024, 1 << 14] {
            assert_eq!(validate_size(n), Ok(()), "n={n}");
        }
    }

    #[test]
    fn zero_is_rejected() {
        assert_eq!(validate_size(0), Err(RingError::ZeroSize));
    }

    #[test]
    fn non_powers_of_two_are_rejected() {
        // 3, 100, 1000: all plausible-looking queue depths, all invalid.
        for n in [3u16, 6, 100, 1000] {
            assert_eq!(validate_size(n), Err(RingError::NotPowerOfTwo), "n={n}");
        }
    }

    // ── descriptors ─────────────────────────────────────────────────────────

    #[test]
    fn a_read_descriptor_is_not_marked_write() {
        let d = Descriptor::read(0x1000, 64);
        assert!(!d.is_write(), "a TX descriptor must be device-reads");
        assert!(!d.has_next());
    }

    #[test]
    fn a_write_descriptor_is_marked_write() {
        // This is the flag that, when missing on an RX descriptor, lets the
        // device write frames into the descriptor table itself.
        assert!(Descriptor::write(0x1000, 64).is_write());
    }

    #[test]
    fn a_descriptor_round_trips_through_ring_memory() {
        let mut q = queue(8);
        let d = Descriptor::write(0xdead_0000, 1514);
        unsafe { q.set_descriptor(3, &d) };
        unsafe {
            let off = q.desc_off(3);
            let addr = u64::from_le(core::ptr::read_volatile(q.raw(off).cast::<u64>()));
            let len = u32::from_le(core::ptr::read_volatile(q.raw(off + 8).cast::<u32>()));
            let flags = u16::from_le(core::ptr::read_volatile(q.raw(off + 12).cast::<u16>()));
            assert_eq!(addr, 0xdead_0000);
            assert_eq!(len, 1514);
            assert_eq!(flags, VRING_DESC_F_WRITE);
        }
    }

    // ── the driver's half ───────────────────────────────────────────────────

    #[test]
    fn publishing_advances_only_the_avail_index() {
        let mut q = queue(8);
        unsafe { q.set_descriptor(0, &Descriptor::write(0x1000, 64)) };
        unsafe { q.publish(0) };
        assert_eq!(q.outstanding(), 1, "one descriptor is now the device's");
        assert_eq!(q.available(), 7);
    }

    #[test]
    fn publishing_records_the_descriptor_head_in_order() {
        let mut q = queue(8);
        for i in 0..4u16 {
            unsafe { q.set_descriptor(i, &Descriptor::write(0x1000 + i as u64 * 64, 64)) };
            unsafe { q.publish(i) };
        }
        // `avail.ring[j]` names the head of the j-th published chain.
        for j in 0..4u16 {
            let v = unsafe { u16::from_le(core::ptr::read_volatile(q.raw(q.avail_ring_off(j)).cast::<u16>())) };
            assert_eq!(v, j, "avail.ring[{j}] should name descriptor {j}");
        }
    }

    #[test]
    fn a_full_queue_reports_nothing_available() {
        let mut q = queue(4);
        for i in 0..4u16 {
            unsafe { q.set_descriptor(i, &Descriptor::write(0x1000, 64)) };
            unsafe { q.publish(i) };
        }
        assert_eq!(q.outstanding(), 4);
        assert_eq!(q.available(), 0, "a full ring must not offer more work");
    }

    // ── the device's half ───────────────────────────────────────────────────

    #[test]
    fn no_completions_yields_nothing() {
        let mut q = queue(8);
        assert_eq!(q.next_completion(), None);
    }

    #[test]
    fn a_completion_is_returned_once_and_then_withheld() {
        let mut q = queue(8);
        device_complete(&mut q, 0, 1, 1514);
        assert_eq!(q.next_completion(), Some((0, 1514)));
        assert_eq!(q.next_completion(), None, "a completion is not re-delivered");
    }

    #[test]
    fn the_device_length_is_authoritative() {
        // A short frame padded into a large buffer: reporting the buffer length
        // would hand trailing garbage to the IP stack as payload.
        let mut q = queue(8);
        device_complete(&mut q, 0, 1, 64);
        let (_, len) = q.next_completion().expect("one completion");
        assert_eq!(len, 64, "the device's length, not the buffer's");
    }

    #[test]
    fn completions_arrive_in_the_order_the_device_wrote_them() {
        let mut q = queue(8);

        unsafe {
            for i in 0..4u16 {
                q.set_descriptor(i, &Descriptor::write(0x1000, 64));
                q.publish(i);
            }
        }
        device_complete(&mut q, 0, 4, 128);
        let mut seen: [u16; 4] = [u16::MAX; 4];
        let mut n = 0;
        q.drain_completions(|id, _| {
            seen[n] = id;
            n += 1;
        });
        assert_eq!(seen, [0, 1, 2, 3]);
        assert_eq!(q.outstanding(), 0, "every completion retired the work");
    }

    #[test]
    fn an_out_of_range_descriptor_id_is_refused() {
        // A device naming a descriptor we do not have is corrupt, or has been
        // reset underneath us. The entry is skipped rather than trusted.
        let mut q = queue(8);
        let off = q.used_ring_off(0);
        unsafe {
            core::ptr::write_volatile(q.raw(off).cast::<u32>(), 999u32.to_le());
            core::ptr::write_volatile(q.raw(off + 4).cast::<u32>(), 64u32.to_le());
            q.write_u16(q.used_idx_off(), 1);
        }
        assert_eq!(q.next_completion(), None, "a bogus id must not be handed up");
    }

    #[test]
    fn draining_is_bounded_by_the_queue_depth() {
        // A device that keeps advancing `used.idx` must not be able to wedge the
        // driver in an unbounded loop.
        let mut q = queue(4);
        device_complete(&mut q, 0, 4, 64);
        device_complete(&mut q, 4, 4, 64);
        let mut count = 0;
        let n = q.drain_completions(|_, _| count += 1);
        assert!(n <= 4, "drain must stop at the depth, got {n}");
        assert_eq!(count, n);
    }
    // ── index arithmetic ────────────────────────────────────────────────────

    #[test]
    fn delta_counts_outstanding_work() {
        assert_eq!(index_delta(5, 2), 3);
        assert_eq!(index_delta(2, 2), 0);
    }

    #[test]
    fn delta_wraps_at_sixteen_bits() {
        // The counters are free-running and wrap at 65536, not at the queue
        // size. A non-wrapping subtraction here reads as "the device went
        // backwards", which presents as a permanent lack of free buffers.
        assert_eq!(index_delta(1, 0xffff), 2);
    }

    #[test]
    fn a_ring_that_wraps_keeps_its_accounting() {
        let mut q = queue(4);
        // Drive both indices around the wrap point.
        for _ in 0..(u16::MAX - 4) / 2 {
            unsafe { q.set_descriptor(0, &Descriptor::write(0x1000, 64)) };
            unsafe { q.publish(0) };
            let from = q.used_idx;
            device_complete(&mut q, from, 1, 64);
            let _ = q.next_completion();
        }
        assert!(q.outstanding() <= 1, "accounting drifted: {}", q.outstanding());
        unsafe { q.set_descriptor(0, &Descriptor::write(0x1000, 64)) };
        unsafe { q.publish(0) };
        assert_eq!(q.outstanding(), 1, "wrapping must not lose a count");
    }

    #[test]
    fn need_event_fires_when_the_threshold_is_crossed() {
        // The device asked to hear about index 5.
        assert!(need_event(4, 5, 4), "crossing the threshold must notify");
        assert!(!need_event(4, 4, 4), "no progress, no notify");
    }

    #[test]
    fn need_event_batches_until_the_threshold() {
        // This is the whole difference between one interrupt per packet and one
        // per batch.
        assert!(!need_event(8, 5, 4), "below the threshold, stay quiet");
        assert!(need_event(8, 9, 4), "past the threshold, notify");
    }

    #[test]
    fn need_event_survives_wrapping() {
        assert!(need_event(0, 1, 0xffff), "a wrap must still notify");
    }

    // ── notification flags ──────────────────────────────────────────────────

    #[test]
    fn no_interrupt_suppresses_notification() {
        let mut q = queue(8);
        q.set_no_interrupt();
        assert!(!q.should_notify(0, 1), "the device asked us to stay quiet");
    }

    #[test]
    fn a_ring_without_the_flag_notifies() {
        let q = queue(8);
        assert!(q.should_notify(0, 1), "the default is to notify");
    }

    #[test]
    fn no_notify_suppresses_the_kick() {
        let mut q = queue(8);
        assert!(q.should_kick(0, 1));
        q.set_no_interrupt();
        assert!(!q.should_kick(0, 1), "the device does not want to be prodded");
    }

    #[test]
    fn events_survive_a_round_trip_through_memory() {
        let mut q = queue(8);
        // `used_event` lives in the available ring, `avail_event` just past the
        // used ring. Writing both to the same place would pass a test while the
        // driver read the wrong word.
        let used_at = q.avail_ring_off(q.num());
        let avail_at = q.used_ring_off(q.num());
        assert_ne!(used_at, avail_at, "the two thresholds are in different rings");
        unsafe {
            core::ptr::write_volatile(q.raw(used_at).cast::<u16>(), 34u16.to_le());
            core::ptr::write_volatile(q.raw(avail_at).cast::<u16>(), 12u16.to_le());
        }
        q.read_events();
        // `avail_event` = 12 governs whether the device interrupts *us*.
        assert!(!q.should_notify(0, 11), "below the threshold, stay quiet");
        assert!(q.should_notify(0, 13), "at the threshold we notify");
        // `used_event` = 34 governs whether *we* interrupt the device.
        assert!(!q.should_kick(0, 33));
        assert!(q.should_kick(0, 35));
    }
}
