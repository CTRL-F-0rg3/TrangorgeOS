//! Power / battery IPC message payloads.
//!
//! The vocabulary here is the one already published in
//! `kernel/src/interfejs/trangorge.h` - `BATT_STATE_DISCHARGING = 0`,
//! `CHARGING = 1`, `FULL = 2`, and a status of `{present, state, percent,
//! voltage_mv, temp_c, health}`. Keeping the numbers identical is deliberate:
//! it means the existing C header and this framework describe the same thing,
//! so a port does not have to choose between them.
//!
//! What is *added* on top is the part a laptop actually needs and a stub
//! battery driver never had: an energy figure rather than only a percentage,
//! and a charge policy that is clamped to the cell's limits.

/// What the machine is doing with its power source.
///
/// Numbered to match `BATT_STATE_*` in `trangorge.h`. `NotPresent` is a fourth
/// value that the C header does not have, because the C header reports absence
/// through a separate `present` field; here `present` is still a field, and
/// this state means "the source is gone", which is a different thing from
/// "there was never a source here".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum PowerState {
    /// Drawing from the battery, or draining it.
    Discharging = 0,
    /// Taking in charge.
    Charging = 1,
    /// At the top of charge.
    Full = 2,
    /// The source is not connected.
    #[default]
    NotPresent = 3,
    /// The source exists but is not doing anything: an AC adapter that is
    /// present but not charging because the battery is full, or a machine
    /// between thresholds.
    Idle = 4,
    /// The source is present and the hardware reports a fault. Check `health`.
    Fault = 5,
}

impl PowerState {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::Discharging,
            1 => Self::Charging,
            2 => Self::Full,
            3 => Self::NotPresent,
            4 => Self::Idle,
            5 => Self::Fault,
            _ => Self::NotPresent,
        }
    }

    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Discharging => "discharging",
            Self::Charging => "charging",
            Self::Full => "full",
            Self::NotPresent => "absent",
            Self::Idle => "idle",
            Self::Fault => "fault",
        }
    }

    /// Is the source giving the machine power right now?
    #[inline]
    pub const fn is_online(self) -> bool {
        matches!(self, Self::Charging | Self::Full | Self::Idle)
    }
}

/// How worn the cell is.
///
/// An enum rather than a percentage because "82%" as a health figure is
/// meaningless without knowing what it is out of, and the four states a user
/// can act on are genuinely different: two mean "replace it", one means
/// "calibrate it", and one means nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum Health {
    /// The cell reports full design capacity, or close enough that the gauge
    /// cannot tell.
    Good = 0,
    /// Worn, but above the point where capacity is user-visible.
    Fair = 1,
    /// Below the point where runtime is materially shorter than it was.
    Poor = 2,
    /// Effectively end of life.
    Critical = 3,
    /// A different chemistry, and the capacity figures need a conversion table
    /// the driver does not have.
    #[default]
    Unknown = 4,
}

impl Health {
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::Good,
            1 => Self::Fair,
            2 => Self::Poor,
            3 => Self::Critical,
            _ => Self::Unknown,
        }
    }

    /// Should the user be told?
    ///
    /// The threshold is `Poor`, not `Fair`: a cell at `Fair` is normal for its
    /// age, and warning about it trains people to ignore the warning.
    #[inline]
    pub const fn warrants_warning(self) -> bool {
        matches!(self, Self::Poor | Self::Critical)
    }
}

/// What a [`PowerInfo::kind`] is.
///
/// A module rather than a Rust enum, and at module scope rather than inside the
/// `impl`, for two reasons. An `impl` cannot contain a module at all; and a back
/// end may report a kind this framework has never heard of, which an enum could
/// not represent without a catch-all that carries no information.
pub mod kind {
    /// The main pack.
    pub const BATTERY: u32 = 0;
    /// A mains adapter.
    pub const AC_ADAPTER: u32 = 1;
    /// A USB-C port that can deliver power.
    pub const USB_C: u32 = 2;
    /// A wireless charge pad.
    pub const WIRELESS: u32 = 3;
}

/// The static descriptor of one power source.
///
/// Read once at enumeration. Everything here is a property of the hardware and
/// does not change, which is why it is separate from [`PowerStatus`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct PowerInfo {
    /// Index within the driver's source list.
    pub index: u32,
    /// What the source is, from [`PowerInfo::kind`].
    pub kind: u32,
    /// Design capacity, in milliwatt-hours. Zero when the gauge cannot say.
    pub design_mwh: u64,
    /// Full charge capacity, in milliwatt-hours.
    pub full_mwh: u64,
    /// Design voltage, in millivolts.
    pub design_mv: u32,
    /// The lowest charge percentage the pack allows.
    pub min_percent: u32,
    /// The highest charge percentage the pack allows.
    pub max_percent: u32,
    /// The charge rate the cell was validated for, in percent per hour.
    pub max_charge_rate_pct_h: u32,
}

impl PowerInfo {
    /// Is this a source that can hold charge?
    #[inline]
    pub const fn is_battery(self) -> bool {
        self.kind == kind::BATTERY
    }

    /// The charge window, always at least one percentage point wide.
    ///
    /// A source whose minimum is at or above its maximum cannot be charged at
    /// all. That is a hardware fault rather than a policy choice, and the clamp
    /// here is what stops a caller computing a negative window out of it.
    #[inline]
    pub const fn charge_window(self) -> (u32, u32) {
        let lo = if self.min_percent > 99 { 0 } else { self.min_percent };
        let hi = if self.max_percent > 100 { 100 } else { self.max_percent };
        if hi <= lo {
            (lo, lo + 1)
        } else {
            (lo, hi)
        }
    }
}

/// A sampled reading from one power source.
///
/// The `percent` field is the *raw* gauge value. It is not what a user should
/// see - see `ds_fw_power::smooth` for why, and for the filter the driver puts
/// in front of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct PowerStatus {
    /// The source this reading came from.
    pub index: u32,
    /// What the source is doing.
    pub state: u32,
    /// Raw charge, 0-100, straight from the gauge.
    pub percent: u32,
    /// Terminal voltage, in millivolts.
    pub voltage_mv: u32,
    /// Cell temperature, in tenths of a degree Celsius. Signed, because a cold
    /// pack is a normal thing to find and `-150` is not a fault.
    pub temp_deci_c: i32,
    /// `Health` as a number.
    pub health: u32,
    /// Instantaneous current, in milliamps. Positive when charging.
    pub current_ma: i32,
    /// Charge remaining, in milliwatt-hours. The unit is a tenth of a watt-hour
    /// to keep the arithmetic in integers, which is where all of it happens.
    pub energy_mwh: u64,
    /// The raw reading's age when it was taken, in milliseconds.
    ///
    /// A gauge can be up to a minute stale, and a stale reading presented as
    /// current is how a battery indicator ends up lying.
    pub age_ms: u32,
}

impl PowerStatus {
    /// The decoded state.
    #[inline]
    pub fn state(&self) -> PowerState {
        PowerState::from_u32(self.state)
    }

    /// The decoded health.
    #[inline]
    pub fn health(&self) -> Health {
        Health::from_u32(self.health)
    }

    /// Seconds of runtime left, or `u64::MAX` when there is no honest answer.
    ///
    /// The condition for an honest answer is that the battery is *discharging*.
    /// Extrapolating runtime from a charging pack, or from one whose current
    /// reading is too small to be trusted, produces a confident wrong number -
    /// and a UI that shows "8 hours remaining" on a full laptop is worse than
    /// one that shows nothing.
    pub fn runtime_seconds(&self) -> u64 {
        if self.state() != PowerState::Discharging {
            return u64::MAX;
        }
        // Below this the current reading is mostly noise, and dividing by noise
        // produces a huge number that looks like a real estimate.
        if self.current_ma <= 100 {
            return u64::MAX;
        }
        let mah = self.energy_mwh / 1000;
        (mah * 3600) / self.current_ma.unsigned_abs() as u64
    }
}

/// What the machine should do when charging.
///
/// A threshold pair rather than a flag, because every real user preference is
/// "start at X, stop at Y" and a driver that offers only "start/stop" makes
/// the client invent the other half.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct PowerPolicy {
    /// Which source this applies to.
    pub index: u32,
    /// Start charging below this percentage.
    pub start_percent: u32,
    /// Stop charging at or above this percentage.
    pub stop_percent: u32,
    /// Non-zero to let the pack age naturally rather than hold a full charge.
    ///
    /// On by default on real hardware for a reason: a cell parked at 100% and
    /// warm is the worst case for cycle life, and nobody asks for that on
    /// purpose. It is a default, not a requirement.
    pub charge_avoid_full: u32,
}

impl PowerPolicy {
    /// A policy that charges all the way, the way a machine with no battery
    /// management would.
    #[inline]
    pub const fn full_charge() -> Self {
        Self { index: 0, start_percent: 0, stop_percent: 100, charge_avoid_full: 0 }
    }

    /// Clamp to the window the pack actually allows.
    ///
    /// This is the function that makes a policy safe to accept from a client.
    /// Two things are enforced: the thresholds stay inside the pack's own
    /// limits, and `stop` stays above `start`, because a policy that charges
    /// toward a lower number than it starts from will oscillate - the pack
    /// reaches `stop`, the driver stops, current falls, the gauge reads `start`
    /// again, and the pack charges. A user who set that up should get a
    /// corrected policy, not a laptop that oscillates visibly.
    pub fn clamp_to(&self, info: PowerInfo) -> PowerPolicy {
        let (lo, hi) = info.charge_window();
        let mut start = self.start_percent.clamp(lo, hi);
        let mut stop = self.stop_percent.clamp(lo, hi);
        if stop <= start {
            // Push the pair apart, preferring to keep `start` where it was asked
            // for and move `stop`, because the stop is what the user sees.
            start = start.min(hi.saturating_sub(1));
            stop = (start + 1).min(hi);
        }
        PowerPolicy {
            index: self.index,
            start_percent: start,
            stop_percent: stop,
            charge_avoid_full: self.charge_avoid_full,
        }
    }

    /// Is the pack where it should be given this policy?
    ///
    /// `Idle` is the state where the charger is attached but the pack is being
    /// held, which is exactly what a threshold policy is asking for. Treating it
    /// as "wrong" would report a misconfigured machine on every threshold-holding
    /// laptop.
    #[inline]
    pub fn is_satisfied_by(&self, status: &PowerStatus) -> bool {
        match status.state() {
            PowerState::Charging | PowerState::Full => true,
            PowerState::Idle => true,
            _ => status.percent >= self.start_percent,
        }
    }
}

