//! Budgeted polling - the NAPI model.
//!
//! # The problem this solves
//!
//! The original `NetworkDevice::poll()` had no bound. It returned only when the
//! device reported "nothing left", and on a fast link with small buffers the
//! device produced packets faster than the stack consumed them, so that
//! condition was never reached. The softirq stayed in `poll` and the machine
//! stopped running anything else.
//!
//! Linux hit this with 10 Mbit half-duplex in 1998 and solved it by inverting
//! the interrupt model: the NIC raises one interrupt, the driver enters a
//! *budgeted* poll, and when the budget runs out it tells the caller to come
//! back rather than continuing to spin. The bound is the entire mechanism.
//!
//! # What a driver does with this
//!
//! ```text
//!   on interrupt:  napi.schedule()          → armed, budget refilled
//!   on softirq:    loop { napi.poll(dev) }  → bounded work per call
//!   after budget:  napi.done()              → reschedule if the ring is full
//! ```
//!
//! The subtlety is that `done` must distinguish "the ring is empty, stop" from
//! "the budget ran out with packets still waiting, come back". Conflating them
//! either livelocks or drops. [`PollOutcome`] makes the two a return value
//! rather than a convention.
//!
//! # The budget is a starting point, not a constant
//!
//! [`Napi::adapt`] doubles the budget while the ring keeps overflowing and
//! halves it when polls end empty, which converges to the machine's actual rate
//! without anyone tuning it. Baking a constant into a trait is how you end up
//! with a driver that is wasteful on a slow link and starved on a fast one.

/// How many packets one poll will hand up before giving the CPU back.
///
/// The Linux default: large enough that a busy 1 Gbit link is not dominated by
/// per-packet overhead, small enough that a softirq does not hold a core.
pub const DEFAULT_BUDGET: u32 = 64;

/// The floor [`Napi::adapt`] will shrink the budget to.
///
/// A budget of 1 would be correct and practically terrible: per-poll overhead
/// would dominate and throughput would collapse.
pub const MIN_BUDGET: u32 = 4;

/// The ceiling [`Napi::adapt`] will grow the budget to.
///
/// Above this a softirq holds the CPU long enough to be felt, which is the
/// failure this module exists to prevent.
pub const MAX_BUDGET: u32 = 1024;

/// How one bounded poll ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollOutcome {
    /// The device has nothing more for now. Stop, and wait for the next
    /// interrupt.
    Idle,
    /// The budget ran out with packets still queued. Come straight back.
    ///
    /// The distinction from [`PollOutcome::Idle`] is the whole point: reporting
    /// this as idle drops packets, and reporting idle as this one spins.
    Reschedule,
    /// The device is wedged and the poll cannot proceed.
    Faulted,
}

/// The budgeted poll state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Napi {
    /// Packets one poll may hand up.
    budget: u32,
    /// Is a poll currently owed?
    armed: bool,
    /// Packets handed up by the most recent poll.
    last_batched: u32,
    /// Total packets handed up since the driver started.
    total: u64,
}

impl Napi {
    /// A controller with the default budget and no poll owed.
    #[inline]
    pub const fn new() -> Self {
        Self { budget: DEFAULT_BUDGET, armed: false, last_batched: 0, total: 0 }
    }

    /// A controller with an explicit budget.
    ///
    /// Clamped into `MIN_BUDGET..=MAX_BUDGET`, because a zero budget would
    /// return `Reschedule` forever and a caller that trusts it would spin -
    /// reintroducing exactly the livelock this module exists to remove.
    #[inline]
    pub const fn with_budget(budget: u32) -> Self {
        let clamped = if budget < MIN_BUDGET {
            MIN_BUDGET
        } else if budget > MAX_BUDGET {
            MAX_BUDGET
        } else {
            budget
        };
        Self { budget: clamped, armed: false, last_batched: 0, total: 0 }
    }

    /// Note that the device has work, so the next poll is owed.
    #[inline]
    pub const fn schedule(&mut self) {
        self.armed = true;
    }

    /// Is a poll owed?
    #[inline]
    pub const fn is_armed(&self) -> bool {
        self.armed
    }

    /// The current budget.
    #[inline]
    pub const fn budget(&self) -> u32 {
        self.budget
    }

    /// How many packets the last poll handed up.
    #[inline]
    pub const fn last_batched(&self) -> u32 {
        self.last_batched
    }

    /// How many packets have been handed up in total.
    #[inline]
    pub const fn total(&self) -> u64 {
        self.total
    }

    /// Run one bounded poll: take up to `budget` packets, then report what
    /// happened.
    ///
    /// `take` pops one received frame and returns `true`, or returns `false`
    /// for "the ring is empty". It is a closure rather than a trait method so
    /// this stays testable without a device, and so the budget arithmetic has
    /// exactly one place it can be wrong.
    ///
    /// The loop runs at most `budget` times. When the budget is spent, the
    /// decision between [`PollOutcome::Idle`] and [`PollOutcome::Reschedule`]
    /// comes from `has_more` - which must **not** consume a packet.
    ///
    /// That is not a style point. An earlier version took the deciding packet,
    /// which meant the probe reported "there is more" *and* swallowed the packet
    /// it was checking for: a burst that landed exactly on the boundary lost one
    /// frame per burst, invisibly, and `last_batched` still said the budget was
    /// met. The property this module exists to protect - a poll never loses a
    /// packet - is destroyed by a probe that eats one.
    ///
    /// So there are two closures: `take` hands a packet up and consumes it, and
    /// `has_more` only looks. On a real driver the first pops the device's used
    /// ring and the second reads its "more descriptors" bit.
    pub fn poll<T, M>(&mut self, mut take: T, has_more: M) -> PollOutcome
    where
        T: FnMut() -> bool,
        M: Fn() -> bool,
    {
        let mut taken = 0u32;
        while taken < self.budget {
            if !take() {
                return self.finish(taken, false);
            }
            taken += 1;
        }
        // Budget spent, nothing more consumed. Ask without taking.
        self.finish(taken, has_more())
    }

    /// Record the end of a poll and produce its outcome.
    fn finish(&mut self, taken: u32, more: bool) -> PollOutcome {
        self.last_batched = taken;
        self.total += u64::from(taken);
        if more {
            PollOutcome::Reschedule
        } else {
            self.armed = false;
            PollOutcome::Idle
        }
    }

    /// Adjust the budget from how the last poll went.
    ///
    /// The shrink is deliberately more willing than the growth: a budget that is
    /// too large shows up as latency, which is visible, while one that is too
    /// small shows up as throughput loss, which is not noticed until somebody
    /// benchmarks it.
    pub fn adapt(&mut self, last_outcome: PollOutcome) -> u32 {
        match last_outcome {
            // Hit the ceiling with more waiting: this machine can take more.
            PollOutcome::Reschedule => self.budget = (self.budget * 2).min(MAX_BUDGET),
            // Ended empty, possibly well under budget: the machine is idle
            // enough that a smaller budget costs nothing and returns the CPU
            // sooner.
            PollOutcome::Idle => {
                let halved = (self.budget / 2).max(MIN_BUDGET);
                self.budget = if halved < self.budget { halved } else { self.budget };
            }
            // A faulted device is not evidence about the right budget. Leave it.
            PollOutcome::Faulted => {}
        }
        self.budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ring with a fixed number of packets in it, drained by the poll.
    ///
    /// The counters are `Cell`s rather than plain fields so that `pop` and
    /// `more` can both borrow it immutably. With plain fields, `poll`'s two
    /// closures - one consuming, one looking - would need a mutable and an
    /// immutable borrow of the same ring at once, and would not compile. The
    /// `Cell` is the honest shape: the ring is shared state that the poll reads
    /// and writes from inside one call.
    struct Ring {
        left: core::cell::Cell<u32>,
        produced: core::cell::Cell<u32>,
    }

    impl Ring {
        fn new(count: u32) -> Self {
            Self {
                left: core::cell::Cell::new(count),
                produced: core::cell::Cell::new(0),
            }
        }

        fn left(&self) -> u32 {
            self.left.get()
        }
    }

    /// Pop one packet, consuming it.
    fn pop(r: &Ring) -> impl FnMut() -> bool + '_ {
        move || {
            if r.left.get() == 0 {
                return false;
            }
            r.left.set(r.left.get() - 1);
            r.produced.set(r.produced.get() + 1);
            true
        }
    }

    /// Look at the ring without consuming from it.
    fn more(r: &Ring) -> impl Fn() -> bool + '_ {
        move || r.left.get() > 0
    }

    #[test]
    fn an_empty_device_polls_idle_and_disarms() {
        let mut n = Napi::new();
        n.schedule();
        assert!(n.is_armed());
        assert_eq!(n.poll(|| false, || false), PollOutcome::Idle);
        assert!(!n.is_armed(), "an idle poll must not leave work owed");
        assert_eq!(n.last_batched(), 0);
    }

    #[test]
    fn a_poll_never_exceeds_the_budget() {
        // The property the whole module exists for. Without it, a fast device
        // holds the CPU inside one poll until the machine stops.
        let mut n = Napi::with_budget(8);
        let endless = Ring::new(u32::MAX);
        let outcome = n.poll(pop(&endless), more(&endless));
        assert_eq!(n.last_batched(), 8, "exactly the budget");
        assert_eq!(outcome, PollOutcome::Reschedule);
        assert!(endless.produced.get() <= 9, "one look past the end, got {}", endless.produced.get());
    }

    #[test]
    fn a_device_with_fewer_packets_than_the_budget_ends_idle() {
        let mut n = Napi::with_budget(64);
        let r = Ring::new(5);
        assert_eq!(n.poll(pop(&r), more(&r)), PollOutcome::Idle);
        assert_eq!(n.last_batched(), 5);
        assert_eq!(r.left(), 0);
    }

    #[test]
    fn a_device_with_exactly_the_budget_ends_idle() {
        // The extra probe after the budget is spent is what makes this honest:
        // four packets taken with a budget of four, one more look, the ring is
        // empty, so there is genuinely nothing to reschedule for. Reporting
        // `Reschedule` here would be a lie that costs a pointless extra poll;
        // reporting `Idle` with packets *left over* is the lie that drops them.
        let mut n = Napi::with_budget(4);
        let r = Ring::new(4);
        assert_eq!(n.poll(pop(&r), more(&r)), PollOutcome::Idle);
        assert_eq!(n.last_batched(), 4, "all four were still handed up");
        assert_eq!(r.left(), 0);
    }

    #[test]
    fn a_device_with_one_more_than_the_budget_reports_reschedule() {
        // The case the extra probe exists for: the budget is spent, the ring is
        // not empty, and a driver that reported `Idle` here would strand a
        // packet until the next unrelated interrupt.
        let mut n = Napi::with_budget(4);
        let r = Ring::new(5);
        assert_eq!(n.poll(pop(&r), more(&r)), PollOutcome::Reschedule);
        assert_eq!(n.last_batched(), 4);
        assert_eq!(r.left(), 1, "exactly one is left for the next poll");
    }

    #[test]
    fn a_zero_budget_is_clamped_rather_than_spinning() {
        // A zero budget would return Reschedule with nothing consumed, and a
        // caller that trusts it would spin - the livelock, reintroduced.
        let mut n = Napi::with_budget(0);
        assert_eq!(n.budget(), MIN_BUDGET);
        assert_eq!(n.poll(|| false, || false), PollOutcome::Idle);
    }

    #[test]
    fn a_budget_above_the_ceiling_is_clamped() {
        assert_eq!(Napi::with_budget(100_000).budget(), MAX_BUDGET);
    }

    #[test]
    fn the_total_counts_every_packet_handed_up() {
        let mut n = Napi::with_budget(4);
        for _ in 0..3 {
            let r = Ring::new(4);
            let _ = n.poll(pop(&r), more(&r));
        }
        assert_eq!(n.total(), 12);
    }



    #[test]
    fn the_budget_grows_while_the_device_cannot_be_drained() {
        let mut n = Napi::with_budget(8);
        assert_eq!(n.adapt(PollOutcome::Reschedule), 16);
        assert_eq!(n.adapt(PollOutcome::Reschedule), 32);
    }

    #[test]
    fn the_budget_shrinks_when_polls_end_empty_and_never_collapses() {
        let mut n = Napi::with_budget(64);
        assert_eq!(n.adapt(PollOutcome::Idle), 32);
        for _ in 0..20 {
            n.adapt(PollOutcome::Idle);
        }
        assert_eq!(n.budget(), MIN_BUDGET, "it must stop at the floor, not go to 1");
    }

    #[test]
    fn the_budget_stays_inside_its_ceiling() {
        let mut n = Napi::with_budget(MAX_BUDGET);
        for _ in 0..10 {
            n.adapt(PollOutcome::Reschedule);
        }
        assert_eq!(n.budget(), MAX_BUDGET);
    }

    #[test]
    fn a_faulted_device_is_not_evidence_about_the_budget() {
        let mut n = Napi::with_budget(32);
        assert_eq!(n.adapt(PollOutcome::Faulted), 32);
    }

    #[test]
    fn a_burst_then_quiet_ends_the_sequence() {
        // The realistic cycle: a burst that does not fill the budget, then
        // silence. Once the burst is drained the driver must stop claiming work.
        let mut n = Napi::with_budget(4);
        n.schedule();
        let r = Ring::new(3);
        assert_eq!(n.poll(pop(&r), more(&r)), PollOutcome::Idle);
        assert!(!n.is_armed());
        assert_eq!(n.poll(|| false, || false), PollOutcome::Idle);
    }
}


impl Default for Napi {
    fn default() -> Self {
        Self::new()
    }
}
