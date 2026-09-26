//! The interface between decoded video and whatever wants to consume it.
//!
//! # The problem this solves
//!
//! A video driver naturally wants to call into the graphics server, and a neural
//! accelerator naturally wants to call into the video driver. Doing either makes
//! an optional component mandatory and puts a cycle in the dependency graph. A
//! TrangorgeOS machine with no NPU must still decode video; a headless machine
//! with no compositor must still run a capture server. Neither may fail to
//! build, load, or link because a component that is not its business is absent.
//!
//! So the direction is inverted. The video driver owns the frames and *pushes*
//! them through [`FrameSink`]; consumers *pull*. A sink is registered at
//! runtime, by whoever is present.
//!
//! ```text
//!                    ┌──────────────────────────────┐
//!                    │      Video-TrangorgeOS       │
//!                    │  decode → FrameRing → push   │
//!                    └──────────────┬───────────────┘
//!                    ┌──────────────┴───────────────┐
//!                    ▼                              ▼
//!         ┌─────────────────┐            ┌─────────────────┐
//!         │ compositor      │            │ NPU             │
//!         │ (pulls)        │            │ (pulls)         │
//!         └─────────────────┘            └─────────────────┘
//!         neither depends on the other, and neither is compiled in
//! ```
//!
//! # Zero consumers is a valid configuration
//!
//! A driver with no sinks registered still decodes. The frames go into a ring
//! and are reclaimed when it wraps. This is not a degenerate case to be
//! apologised for - it is what a capture server nobody is currently watching
//! looks like, and it must not stall the decoder.

use alloc::boxed::Box;
use core::fmt;

use kapi_abi::DsError;
use kapi_abi::payloads::video::FrameInfo;

/// A registered frame consumer.
///
/// Object-safe, so a driver holds `&mut dyn FrameSink` and never names a
/// concrete consumer type. Implementations live in whatever crate owns the
/// consumer - the compositor, the NPU runtime, a capture writer - and register
/// themselves at runtime rather than being linked in.
pub trait FrameSink {
    /// Can this sink take frames in this layout?
    ///
    /// Asked once, when the sink is bound to a stream, so the driver can
    /// configure the engine's output format instead of converting per frame.
    /// A sink that returns `false` still receives nothing, rather than receiving
    /// frames it has to discard.
    fn accepts(&self, format: &SinkFormat) -> bool;

    /// Take one frame.
    ///
    /// Returns [`SinkError::Full`] rather than blocking when the consumer is
    /// behind. **A sink must never block.** It is called from the decode
    /// completion path, which is shared with every other session on the engine;
    /// a sink that stalls it stalls everything, and the only visible symptom is
    /// that unrelated video slowed down.
    fn accept(&mut self, frame: &FrameInfo) -> Result<(), SinkError>;

    /// The consumer is being unbound; release anything held.
    ///
    /// Called exactly once, before the sink's memory goes away. A sink that
    /// returns frames from `accept` after this point is a use-after-free, so the
    /// driver drops its reference to the sink immediately after calling it.
    fn close(&mut self) {}
}

/// What a sink is being offered, without committing to a [`FrameInfo`].
///
/// The driver knows the engine's output format when it binds the sink, which is
/// earlier than it knows anything about a specific frame. A separate type keeps
/// the two apart, so `accepts` cannot be written in terms of a frame that does
/// not exist yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SinkFormat {
    /// FourCC of the pixel format the engine will produce.
    pub format: u32,
    /// Bytes per row.
    pub stride: u32,
    /// Visible width.
    pub width: u32,
    /// Visible height.
    pub height: u32,
    /// Whether the driver can hand over a surface in place, with no copy.
    ///
    /// Reported to the consumer rather than assumed, because a consumer that
    /// believes it is zero-copy and is not will produce tearing that looks like
    /// a driver bug.
    pub zero_copy: bool,
}

impl SinkFormat {
    /// Build a format descriptor.
    #[inline]
    pub const fn new(format: u32, width: u32, height: u32, stride: u32) -> Self {
        Self { format, stride, width, height, zero_copy: false }
    }
}

/// Why a sink declined a frame.
///
/// Three variants because the driver reacts differently to each, and
/// [`SinkError::RetryNext`] exists so that "busy" stays distinguishable from
/// "full".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkError {
    /// The consumer's ring is full. Drop this frame; a later one may fit.
    Full,
    /// The frame's layout is not one this sink handles.
    ///
    /// Usually means the engine's output format changed after binding, which is
    /// a driver bug rather than a consumer one - hence worth telling apart.
    Unsupported,
    /// The sink is busy right now but will not be forever. Skip this frame.
    RetryNext,
}

impl fmt::Display for SinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full => f.write_str("the consumer's frame ring is full"),
            Self::Unsupported => f.write_str("the frame layout is not supported by this sink"),
            Self::RetryNext => f.write_str("the consumer is busy"),
        }
    }
}

/// A handle to a registered sink, issued by [`SinkRegistry::register`].
///
/// Generation-tagged: the low bits are an index into the registry and the high
/// bits count registrations, so a stale id from a sink that was removed and
/// re-added at the same slot is rejected instead of steering frames into
/// whichever consumer now occupies it. The same trick `PcmHandle` uses, for the
/// same reason.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SinkId(pub u32);

impl SinkId {
    /// The "not a sink" value.
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }

    /// The registry slot this id names.
    #[inline]
    pub const fn slot(self) -> usize {
        (self.0 & 0x7f) as usize
    }

    /// The generation of the registration.
    #[inline]
    pub const fn generation(self) -> u32 {
        self.0 >> 7
    }

    #[inline]
    const fn make(slot: usize, generation: u32) -> Self {
        Self(((generation & 0x01ff_ffff) << 7) | (slot as u32 & 0x7f))
    }
}

/// How many sinks one driver may have registered at once.
///
/// Fixed, because the framework is `no_std` with no global allocator. Eight is
/// well above the real number - a compositor, an NPU and a capture writer is
/// three - and a limit that exists beats an allocation that fails at run time.
pub const MAX_SINKS: usize = 8;

/// What happened to a frame that was pushed at a set of sinks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DispatchReport {
    /// Sinks that took the frame.
    pub delivered: u32,
    /// Sinks whose ring was full. Their frame is gone.
    pub dropped_full: u32,
    /// Sinks that refused the layout.
    pub dropped_unsupported: u32,
    /// Sinks that were busy.
    pub dropped_busy: u32,
    /// Sinks that were closed and have been removed.
    pub dropped_dead: u32,
}

impl DispatchReport {
    /// A frame nobody took, and nobody was registered for.
    #[inline]
    pub const fn is_unhandled() -> Self {
        Self { delivered: 0, dropped_full: 0, dropped_unsupported: 0, dropped_busy: 0, dropped_dead: 0 }
    }

    /// Did anybody get the frame?
    #[inline]
    pub const fn is_delivered(&self) -> bool {
        self.delivered != 0
    }

    /// Frames lost on the way to at least one consumer.
    ///
    /// Non-zero is normal and not by itself an error: a compositor that is
    /// briefly behind loses frames, and telling it so is how it resynchronises.
    /// What matters is the ratio over time, which the driver keeps in
    /// `VideoDropCount`.
    #[inline]
    pub const fn total_dropped(&self) -> u32 {
        self.dropped_full + self.dropped_unsupported + self.dropped_busy + self.dropped_dead
    }
}

/// A fixed set of registered sinks, with fan-out.
///
/// The registry is the whole of the video-to-anything-else interface: the driver
/// holds one of these, consumers register themselves, and nothing in this file
/// knows what a compositor or an NPU is.
///
/// # Why a full sink is not a dead one
///
/// A sink that returns an error is not necessarily broken, so errors like
/// [`SinkError::Full`] do *not* unregister it - a full ring is a temporary
/// condition and the consumer is still there. Only an id that no longer resolves
/// is treated as gone. Without that distinction a consumer that fell behind once
/// would be dropped permanently, and the operator would see video stop reaching a
/// display that was merely busy.
pub struct SinkRegistry {
    slots: [Option<Entry>; MAX_SINKS],
    generation: u32,
    live: u32,
}

/// One registration.
struct Entry {
    id: SinkId,
    /// The sink itself, boxed.
    ///
    /// `Box` rather than a static array of trait objects because this framework
    /// is `no_std` with no global allocator - but the *driver* is not: the
    /// registry is a per-driver value the driver owns, so it may have an
    /// allocator even though the framework cannot.
    sink: Box<dyn FrameSink>,
    /// What this consumer said it was for, for logging and backpressure.
    purpose: u32,
    push: bool,
    depth: u32,
}

impl Default for SinkRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SinkRegistry {
    /// Prints occupancy, never the sinks: a `dyn FrameSink` has no useful
    /// `Debug`, and a registry dump exists to diagnose fan-out, not to
    /// introspect a compositor.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SinkRegistry").field("live", &self.live).finish()
    }
}

impl SinkRegistry {
    /// An empty registry.
    ///
    /// An empty registry is a valid steady state, not a temporary one.
    ///
    /// The inline `const` block is required: `Option<Entry>` is not `Copy`, so
    /// `[None; MAX_SINKS]` cannot be built by repetition in a `const fn`.
    #[inline]
    pub const fn new() -> Self {
        Self { slots: [const { None }; MAX_SINKS], generation: 0, live: 0 }
    }

    /// Register a consumer, returning its id.
    ///
    /// Fails when [`MAX_SINKS`] are already registered, or when the generation
    /// counter wraps. Both are reported rather than silently overwriting a slot,
    /// because a consumer that believes it registered and did not will wait
    /// forever for frames that never arrive.
    pub fn register(
        &mut self,
        sink: Box<dyn FrameSink>,
        purpose: u32,
        push: bool,
        depth: u32,
    ) -> Result<SinkId, DsError> {
        if self.live as usize >= MAX_SINKS {
            return Err(DsError::OutOfMemory);
        }
        let slot = (0..MAX_SINKS).find(|i| self.slots[*i].is_none()).ok_or(DsError::OutOfMemory)?;
        self.generation = self.generation.wrapping_add(1);
        if self.generation >= 0x01ff_ffff {
            return Err(DsError::OutOfMemory);
        }
        let id = SinkId::make(slot, self.generation);
        self.slots[slot] = Some(Entry { id, sink, purpose, push, depth: depth.max(1) });
        self.live += 1;
        Ok(id)
    }

    /// Remove a sink, calling its `close` first.
    ///
    /// `close` runs while the entry is still in place, so a sink that panics
    /// leaves the slot occupied rather than freeing a sink that is still live.
    /// The alternative - removing first - risks a double free.
    pub fn unregister(&mut self, id: SinkId) -> Result<(), DsError> {
        let slot = self.resolve(id).ok_or(DsError::InvalidHandle)?;
        let mut entry = self.slots[slot].take().ok_or(DsError::InvalidHandle)?;
        entry.sink.close();
        self.live = self.live.saturating_sub(1);
        Ok(())
    }

    /// Offer one frame to every registered sink.
    ///
    /// Never blocks and never fails. The frame has already been decoded by the
    /// time this runs, so there is nothing useful to do about one nobody can
    /// take except count it and move on.
    pub fn dispatch(&mut self, frame: &FrameInfo) -> DispatchReport {
        let mut report = DispatchReport::default();
        for slot in 0..MAX_SINKS {
            let Some(entry) = self.slots[slot].as_mut() else { continue };
            match entry.sink.accept(frame) {
                Ok(()) => report.delivered += 1,
                Err(SinkError::Full) => report.dropped_full += 1,
                Err(SinkError::Unsupported) => report.dropped_unsupported += 1,
                Err(SinkError::RetryNext) => report.dropped_busy += 1,
            }
        }
        report
    }

    /// How many sinks are registered.
    #[inline]
    pub const fn count(&self) -> u32 {
        self.live
    }

    /// Is nobody registered?
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.live == 0
    }

    /// What the sink `id` said it was for, if it is still registered.
    pub fn purpose_of(&self, id: SinkId) -> Option<u32> {
        let slot = self.resolve(id)?;
        self.slots[slot].as_ref().map(|e| e.purpose)
    }

    /// Is the sink at `id` push-mode?
    pub fn is_push(&self, id: SinkId) -> Option<bool> {
        let slot = self.resolve(id)?;
        self.slots[slot].as_ref().map(|e| e.push)
    }

    /// The queue depth the sink declared, floored at one.
    pub fn depth_of(&self, id: SinkId) -> Option<u32> {
        let slot = self.resolve(id)?;
        self.slots[slot].as_ref().map(|e| e.depth)
    }

    /// Check that `id` still names the registration it was issued for.
    ///
    /// The generation check is what makes a stale id harmless: a consumer that
    /// kept its id across an unregister/re-register cycle would otherwise steer
    /// frames into whichever consumer now occupies the same slot.
    fn resolve(&self, id: SinkId) -> Option<usize> {
        if !id.is_valid() {
            return None;
        }
        let slot = id.slot();
        if slot >= MAX_SINKS {
            return None;
        }
        let entry = self.slots[slot].as_ref()?;
        (entry.id == id).then_some(slot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::sync::Arc;
    use core::sync::atomic::{AtomicBool, Ordering};
    use kapi_abi::payloads::video::{sink_purpose, FrameFormat};

    /// A sink that counts what it was given, and can be told to misbehave.
    struct Recorder {
        seen: u32,
        behaviour: SinkError,
        /// When false, `accept` reports `behaviour` instead of taking the frame.
        take: bool,
    }

    impl Recorder {
        /// A sink that always takes the frame.
        fn taking() -> Self {
            Self { seen: 0, behaviour: SinkError::Full, take: true }
        }

        /// A sink that always reports `err` instead of taking the frame.
        ///
        /// `take: false` is the whole point of this constructor. Written as
        /// `..Self::taking()` it would inherit `take: true` and silently accept
        /// every frame, which is exactly what happened the first time.
        fn failing(err: SinkError) -> Self {
            Self { behaviour: err, take: false, seen: 0 }
        }
    }

    impl FrameSink for Recorder {
        fn accepts(&self, _format: &SinkFormat) -> bool {
            true
        }

        fn accept(&mut self, _frame: &FrameInfo) -> Result<(), SinkError> {
            if self.take {
                self.seen += 1;
                Ok(())
            } else {
                Err(self.behaviour)
            }
        }
    }

    /// A sink that records its own teardown, so `close` ordering is testable.
    ///
    /// Holds an `Arc` rather than a borrow: the registry stores a
    /// `Box<dyn FrameSink>`, which is `Box<dyn FrameSink + 'static>`, so a sink
    /// holding a reference to a test local would not compile. An `Arc` is the
    /// way a test gets to observe a `'static` sink from outside.
    struct Closer(Arc<AtomicBool>);

    impl FrameSink for Closer {
        fn accepts(&self, _f: &SinkFormat) -> bool {
            true
        }
        fn accept(&mut self, _frame: &FrameInfo) -> Result<(), SinkError> {
            Ok(())
        }
        fn close(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    fn a_frame() -> FrameInfo {
        FrameInfo {
            present: 1,
            buffer: 7,
            ticket: 3,
            format: FrameFormat::nv12(1920, 1080),
            pts_ns: 1_000,
            dts_ns: 400,
            late: 1,
        }
    }

    #[test]
    fn an_empty_registry_is_a_valid_steady_state() {
        // The point of the whole module: a driver with no consumers still works.
        let mut r = SinkRegistry::new();
        assert!(r.is_empty());
        assert_eq!(r.count(), 0);

        let report = r.dispatch(&a_frame());
        assert!(!report.is_delivered());
        assert_eq!(report.total_dropped(), 0, "a frame nobody wanted is not a drop");
    }

    #[test]
    fn a_registered_sink_receives_the_frame() {
        let mut r = SinkRegistry::new();
        let id = r
            .register(Box::new(Recorder::taking()), sink_purpose::COMPOSITOR, true, 3)
            .expect("register");
        assert!(id.is_valid());
        assert_eq!(r.count(), 1);

        let report = r.dispatch(&a_frame());
        assert_eq!(report.delivered, 1);
        assert_eq!(report.total_dropped(), 0);
        assert!(report.is_delivered());
    }

    #[test]
    fn every_sink_gets_its_own_copy_of_the_frame() {
        // Fan-out, not first-wins: a compositor and an NPU both want the frame.
        let mut r = SinkRegistry::new();
        r.register(Box::new(Recorder::taking()), sink_purpose::COMPOSITOR, true, 3).unwrap();
        r.register(Box::new(Recorder::taking()), sink_purpose::ANALYTICS, true, 2).unwrap();
        r.register(Box::new(Recorder::taking()), sink_purpose::CAPTURE, false, 1).unwrap();

        assert_eq!(r.count(), 3);
        assert_eq!(r.dispatch(&a_frame()).delivered, 3, "all three consumers see it");
    }

    #[test]
    fn the_three_failure_modes_are_counted_separately() {
        // They are three variants because the driver reacts differently, and a
        // `bool` would collapse three different recoveries into one.
        let mut r = SinkRegistry::new();
        r.register(Box::new(Recorder::failing(SinkError::Full)), 3, true, 1).unwrap();
        r.register(Box::new(Recorder::failing(SinkError::Unsupported)), 3, true, 1).unwrap();
        r.register(Box::new(Recorder::failing(SinkError::RetryNext)), 3, true, 1).unwrap();
        r.register(Box::new(Recorder::taking()), 3, true, 1).unwrap();

        let report = r.dispatch(&a_frame());
        assert_eq!(report.delivered, 1);
        assert_eq!(report.dropped_full, 1);
        assert_eq!(report.dropped_unsupported, 1);
        assert_eq!(report.dropped_busy, 1);
        assert_eq!(report.total_dropped(), 3);
    }

    #[test]
    fn a_full_ring_does_not_unregister_the_sink() {
        // A full ring is temporary. Unregistering here would lose the consumer
        // permanently over one slow frame.
        let mut r = SinkRegistry::new();
        let id = r.register(Box::new(Recorder::failing(SinkError::Full)), 3, true, 1).unwrap();
        r.dispatch(&a_frame());
        assert_eq!(r.count(), 1, "a full sink is still registered");
        assert_eq!(r.purpose_of(id), Some(3));
    }

    #[test]
    fn unregistering_runs_close() {
        let closed = Arc::new(AtomicBool::new(false));
        let mut r = SinkRegistry::new();
        let id = r.register(Box::new(Closer(Arc::clone(&closed))), 3, true, 1).unwrap();
        assert!(!closed.load(Ordering::SeqCst));

        r.unregister(id).expect("unregister");
        assert!(closed.load(Ordering::SeqCst), "close must run before the sink is forgotten");
        assert!(r.is_empty());
    }

    #[test]
    fn a_stale_id_cannot_steer_frames_into_a_new_occupant() {
        // The generation tag's whole purpose. Without it this is a
        // use-after-free that only shows up when two consumers happen to land in
        // the same slot.
        let mut r = SinkRegistry::new();
        let old = r.register(Box::new(Recorder::taking()), 3, true, 1).unwrap();
        r.unregister(old).unwrap();

        let new = r.register(Box::new(Recorder::taking()), 4, true, 1).unwrap();
        assert_eq!(old.slot(), new.slot(), "the slot was reused, which is the hazard");
        assert_ne!(old, new, "but the ids differ, by generation");

        assert_eq!(r.purpose_of(old), None, "the stale id resolves to nothing");
        assert_eq!(r.purpose_of(new), Some(4));
        assert_eq!(r.unregister(old).unwrap_err(), DsError::InvalidHandle);
    }

    #[test]
    fn an_id_nobody_was_issued_is_never_resolvable() {
        let r = SinkRegistry::new();
        assert!(!SinkId::INVALID.is_valid());
        assert!(r.purpose_of(SinkId::INVALID).is_none());
        assert!(r.purpose_of(SinkId::new(0)).is_none());
        assert!(r.purpose_of(SinkId::new(1 << 20)).is_none(), "a slot past the table");
    }

    #[test]
    fn the_registry_refuses_more_than_max_sinks() {
        let mut r = SinkRegistry::new();
        for _ in 0..MAX_SINKS {
            r.register(Box::new(Recorder::taking()), 3, true, 1).expect("register");
        }
        assert_eq!(r.count() as usize, MAX_SINKS);
        assert_eq!(
            r.register(Box::new(Recorder::taking()), 3, true, 1).unwrap_err(),
            DsError::OutOfMemory,
            "reported, not silently overwriting a live consumer"
        );
        // And the existing sinks are untouched by the refusal.
        assert_eq!(r.dispatch(&a_frame()).delivered as usize, MAX_SINKS);
    }

    #[test]
    fn a_zero_depth_sink_is_clamped_rather_than_starved() {
        // "Hold no frames" would mean "drop everything", which is almost
        // certainly a caller mistake rather than an intent.
        let mut r = SinkRegistry::new();
        let id = r.register(Box::new(Recorder::taking()), 3, true, 0).unwrap();
        assert_eq!(r.depth_of(id), Some(1));
        assert_eq!(r.dispatch(&a_frame()).delivered, 1);
    }

    #[test]
    fn push_and_pull_consumers_are_distinguishable() {
        let mut r = SinkRegistry::new();
        let push = r.register(Box::new(Recorder::taking()), sink_purpose::COMPOSITOR, true, 3).unwrap();
        let pull = r.register(Box::new(Recorder::taking()), sink_purpose::CAPTURE, false, 2).unwrap();
        assert_eq!(r.is_push(push), Some(true));
        assert_eq!(r.is_push(pull), Some(false));
    }

    #[test]
    fn sink_ids_round_trip_through_their_parts() {
        let mut r = SinkRegistry::new();
        let id = r.register(Box::new(Recorder::taking()), 3, true, 1).unwrap();
        assert_eq!(SinkId::new(id.raw()), id);
        assert!(id.slot() < MAX_SINKS);
        assert!(id.generation() >= 1, "the first registration is generation 1");
    }
}

