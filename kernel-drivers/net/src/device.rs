//! The card contract, completed.
//!
//! The original `NetworkDevice` in `kernel_Workspace/kernel/src/nic/device.rs`
//! had seven methods and no way to ask a card three questions that matter:
//!
//! | Question | Why it matters | Now |
//! |----------|----------------|------|
//! | Is the cable in? | A pulled cable looks like a silent device, and the stack spends minutes re-ARPing | [`LinkState`] |
//! | How many frames went out? | Without counters a bug is indistinguishable from a quiet network | [`Stats`] |
//! | What can this card do? | Offload and GRO change what the stack should send | [`Features`] |
//!
//! The first is the one with a real user-visible failure. `init` returning
//! `Ok` and the link never coming up means DHCP times out, and the operator is
//! told the network is fine.
//!
//! # `poll` and the budget
//!
//! [`NetworkDevice::poll`] no longer returns only when the device is done. It
//! takes a budget and reports whether the caller should come back - see
//! [`crate::napi`]. An unbounded poll is not a slower driver, it is a hung
//! machine, and it is the single most important thing changed here.

use crate::error::NetworkError;
use crate::types::MacAddress;

/// Whether the wire is connected.
///
/// Modelled as four states rather than a boolean because "the card is up and the
/// link is half-duplex" and "the card is up and the link is down" need different
/// responses from the stack, and a boolean forces one of them to guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LinkState {
    /// The card has not been initialised, or is down. The default, because a
    /// device that has not been asked anything has, in fact, been asked nothing.
    #[default]
    Down,
    /// Initialised, no carrier. Normal on a laptop with the cable out.
    NoCarrier,
    /// Up, but the two ends have not agreed yet.
    Negotiating,
    /// Up and carrying frames.
    Up,
}

impl LinkState {
    /// Can frames be exchanged right now?
    #[inline]
    pub fn is_operational(self) -> bool {
        matches!(self, Self::Up)
    }

    /// Is the card itself present and initialised?
    ///
    /// Distinct from the link: a card with no cable is a working card, and a
    /// driver that treats "no carrier" as "card dead" will keep reinitialising
    /// hardware that is perfectly fine.
    #[inline]
    pub fn is_card_present(self) -> bool {
        !matches!(self, Self::Down)
    }
}

/// Whether the link runs in both directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Duplex {
    /// One direction at a time.
    Half,
    /// Both at once. The only mode worth using.
    #[default]
    Full,
    /// The card will not say.
    Unknown,
}

/// The negotiated link rate, in bits per second.
///
/// Megabits, to match what every tool prints. A driver that reports a rate in
/// bytes per second disagrees with `ethtool` and with the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Speed(pub u32);

impl Speed {
    /// 10 Mbit/s.
    pub const M10: Self = Self(10);
    /// 100 Mbit/s.
    pub const M100: Self = Self(100);
    /// 1 Gbit/s.
    pub const G1000: Self = Self(1_000);
    /// 2.5 Gbit/s.
    pub const G2500: Self = Self(2_500);
    /// 10 Gbit/s.
    pub const G10000: Self = Self(10_000);

    /// The rate in bits per second.
    #[inline]
    pub fn bps(self) -> u64 {
        (self.0 as u64) * 1_000_000
    }

    /// Is this a rate a modern driver should accept?
    ///
    /// Ten megabits is not rejected, only called out: half-duplex 10 Mbit is a
    /// real configuration for an old switch, and a driver that refuses it
    /// produces a link that never comes up.
    #[inline]
    pub fn is_plausible(self) -> bool {
        self.0 != 0 && self.0 <= 400_000
    }
}

/// What the card can do, and what the stack may therefore ask it to do.
///
/// A bitmask, not a set of queries, for the reason `ethtool` uses one: the stack
/// must be able to ask "may I use GRO?" in a single test, before it builds a
/// path that depends on the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Features(pub u32);

impl Features {
    /// The card computes IP and TCP checksums.
    pub const CHECKSUM_OFFLOAD: u32 = 1 << 0;
    /// The card segments large packets.
    pub const TX_SEGMENTATION: u32 = 1 << 1;
    /// The card coalesces several packets into one.
    pub const TX_COALESCE: u32 = 1 << 2;
    /// The card merges received packets before the stack sees them.
    ///
    /// The biggest receive-side win available, and the easiest to get wrong:
    /// merging changes the frames the upper layers see, so a stack that enables
    /// it and then checksums the wrong span produces frames that look fine and
    /// fail in transit.
    pub const RX_GRO: u32 = 1 << 3;
    /// The card hashes flows across several queues.
    pub const RX_HASH: u32 = 1 << 4;
    /// The card may compute part of a checksum, not all of it.
    pub const TX_CHECKSUM_PARTIAL: u32 = 1 << 5;

    /// Build from a raw bitmask.
    #[inline]
    pub fn new(bits: u32) -> Self {
        Self(bits)
    }

    /// Does the card have this feature?
    #[inline]
    pub fn has(self, bit: u32) -> bool {
        self.0 & bit != 0
    }

    /// Can the stack hand checksum work to the card?
    #[inline]
    pub fn can_offload_checksum(self) -> bool {
        self.has(Self::CHECKSUM_OFFLOAD)
    }
}

/// Byte and packet counters.
///
/// Present because their absence is not neutral. With no counters, a driver that
/// silently stopped receiving is indistinguishable from one on an idle network -
/// and that is exactly the failure a user cannot debug without `ip -s link`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    /// Frames handed up to the stack.
    pub rx_packets: u64,
    /// Bytes handed up, excluding the FCS.
    pub rx_bytes: u64,
    /// Frames the ring could not take.
    pub rx_dropped: u64,
    /// Frames the hardware rejected as malformed.
    pub rx_errors: u64,
    /// Frames the card accepted.
    pub tx_packets: u64,
    /// Bytes handed to the card.
    pub tx_bytes: u64,
    /// Frames the card refused to send.
    pub tx_errors: u64,
    /// Frames discarded because the stack had no buffer.
    pub tx_dropped: u64,
}

impl Stats {
    /// Count one received frame.
    #[inline]
    pub fn on_rx(&mut self, bytes: usize) {
        self.rx_packets = self.rx_packets.wrapping_add(1);
        self.rx_bytes = self.rx_bytes.wrapping_add(bytes as u64);
    }

    /// Count one received frame that was dropped.
    #[inline]
    pub fn on_rx_dropped(&mut self) {
        self.rx_dropped = self.rx_dropped.wrapping_add(1);
    }

    /// Count one transmitted frame.
    #[inline]
    pub fn on_tx(&mut self, bytes: usize) {
        self.tx_packets = self.tx_packets.wrapping_add(1);
        self.tx_bytes = self.tx_bytes.wrapping_add(bytes as u64);
    }

    /// Count one failed transmission.
    #[inline]
    pub fn on_tx_error(&mut self) {
        self.tx_errors = self.tx_errors.wrapping_add(1);
    }

    /// What fraction of received frames were dropped, in per-mille.
    ///
    /// A ratio rather than a flag, because the question is never "did we drop
    /// one" but "how much are we losing". Returns 0 when nothing has been
    /// received, rather than dividing by zero.
    #[inline]
    pub fn rx_loss_permille(&self) -> u32 {
        let total = self.rx_packets + self.rx_dropped;
        if total == 0 {
            return 0;
        }
        ((self.rx_dropped as u128 * 1000) / total as u128) as u32
    }
}

/// What one bounded poll produced.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PollResult {
    /// Transmissions the card finished.
    pub tx_completed: u16,
    /// Frames ready to be taken.
    pub rx_available: u16,
    /// The card is wedged and needs a reset.
    pub needs_reset: bool,
}

/// A frame handed to the card for transmission.
#[derive(Debug, Clone, Copy)]
pub struct TxFrame<'a> {
    /// The whole frame, header included.
    pub bytes: &'a [u8],
}

impl<'a> TxFrame<'a> {
    #[inline]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}

/// A frame the card has received.
#[derive(Debug)]
pub struct RxFrame<'a> {
    /// Which ring buffer it is in, to be handed back on recycle.
    pub buffer_id: u16,
    /// The frame's bytes.
    pub bytes: &'a [u8],
}

/// A network card.
///
/// Object-safe, so a registry can hold `&mut dyn NetworkDevice` without knowing
/// any card's concrete type - which is what lets virtio-net and an Intel card
/// coexist.
pub trait NetworkDevice {
    /// Bring the card up.
    fn init(&mut self) -> Result<(), NetworkError>;

    /// Take the card down, releasing its rings.
    fn down(&mut self) -> Result<(), NetworkError>;

    /// The card's address, or [`MacAddress::ZERO`] before `init`.
    fn mac_address(&self) -> MacAddress;

    /// Is the wire connected? See [`LinkState`].
    fn link_state(&self) -> LinkState;

    /// The negotiated rate and duplex, if the card will say.
    fn link_speed(&self) -> (Speed, Duplex);

    /// What the card can do.
    fn features(&self) -> Features;

    /// The payload MTU. Frames larger than this are refused, not fragmented by
    /// the driver - the stack above owns fragmentation.
    fn mtu(&self) -> usize;

    /// Change the MTU, if the card can.
    fn set_mtu(&mut self, mtu: usize) -> Result<(), NetworkError>;

    /// Take a copy of the counters.
    fn stats(&self) -> Stats;

    /// Accept every frame, addressed to us or not.
    ///
    /// A sniffer. Off by default, and a card that cannot do it returns
    /// [`NetworkError::NotSupported`] rather than pretending: a sniffer that
    /// silently sees only unicast traffic produces captures that look plausible
    /// and are wrong.
    fn set_promiscuous(&mut self, on: bool) -> Result<(), NetworkError>;

    /// Ask the card to pass group traffic up regardless of its filter.
    fn set_multicast_all(&mut self, on: bool) -> Result<(), NetworkError>;

    /// Hand one frame to the card.
    fn submit_tx(&mut self, frame: TxFrame<'_>) -> Result<(), NetworkError>;

    /// One bounded poll. See [`crate::napi::Napi`].
    fn poll(&mut self, budget: u32) -> Result<PollResult, NetworkError>;

    /// Take the next received frame, if any.
    fn take_rx(&mut self) -> Option<RxFrame<'_>>;

    /// Hand a frame back so the card may refill its buffer.
    fn recycle_rx(&mut self, buffer_id: u16) -> Result<(), NetworkError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_state_separates_the_card_from_the_cable() {
        // The distinction that stops a driver reinitialising a working card
        // because somebody unplugged the cable.
        assert!(LinkState::NoCarrier.is_card_present());
        assert!(!LinkState::NoCarrier.is_operational());
        assert!(!LinkState::Down.is_card_present());
        assert!(LinkState::Up.is_operational());
        assert!(LinkState::Negotiating.is_card_present());
        assert!(!LinkState::Negotiating.is_operational());
    }

    #[test]
    fn speed_is_reported_in_the_units_a_user_expects() {
        assert_eq!(Speed::G1000.bps(), 1_000_000_000);
        assert!(Speed::M100.is_plausible());
        assert!(Speed(100_000).is_plausible(), "100 Gbit exists");
        assert!(!Speed::default().is_plausible(), "zero is not a link rate");
    }

    #[test]
    fn features_answer_one_test() {
        let f = Features::new(Features::CHECKSUM_OFFLOAD | Features::RX_GRO);
        assert!(f.can_offload_checksum());
        assert!(f.has(Features::RX_GRO));
        assert!(!f.has(Features::TX_SEGMENTATION));
    }

    #[test]
    fn a_featureless_card_offloads_nothing() {
        let f = Features::default();
        assert!(!f.can_offload_checksum());
        assert_eq!(f.0, 0);
    }

    #[test]
    fn stats_count_frames_and_bytes() {
        let mut s = Stats::default();
        s.on_rx(1514);
        s.on_rx(60);
        s.on_tx(1514);
        assert_eq!(s.rx_packets, 2);
        assert_eq!(s.rx_bytes, 1574);
        assert_eq!(s.tx_packets, 1);
    }

    #[test]
    fn a_loss_ratio_answers_the_question_a_flag_cannot() {
        // "Did we drop one" is not the question; "how much are we losing" is.
        let mut s = Stats::default();
        for _ in 0..999 {
            s.on_rx(100);
        }
        s.on_rx_dropped();
        assert_eq!(s.rx_loss_permille(), 1, "one in a thousand");

        let mut bad = Stats::default();
        for _ in 0..500 {
            bad.on_rx(100);
            bad.on_rx_dropped();
        }
        assert_eq!(bad.rx_loss_permille(), 500, "half lost is 500 permille, not 1000");
    }

    #[test]
    fn an_idle_device_reports_no_loss_rather_than_dividing_by_zero() {
        assert_eq!(Stats::default().rx_loss_permille(), 0);
    }
}


