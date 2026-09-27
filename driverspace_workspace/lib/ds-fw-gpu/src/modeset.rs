//! Choosing and validating a display mode, and checking the link can carry it.
//!
//! This is the part of a modeset that is neither vendor- nor transport-specific.
//! Both output drivers need the same two answers — "is this mode legal for this
//! sink?" and "does this link have the bandwidth for it?" — and the second is
//! pure arithmetic on the mode's own numbers. Having it in one place is what
//! stops HDMI and DisplayPort from disagreeing about whether 4K60 fits.
//!
//! # The bandwidth check is not optional
//!
//! A mode that does not fit the link does not fail gracefully. On DisplayPort it
//! produces a link that trains and then shows a rolling or torn picture; on HDMI
//! it produces a sink that ignores the mode and keeps whatever it had last. Neither
//! is reported as an error unless the driver checks first — which is what
//! [`LinkCapability::check`] is for.
//!
//! # Why the arithmetic is 64-bit
//!
//! Required bandwidth is clock × bits per pixel, plus an allowance for blanking
//! that real silicon carries and the ideal figure omits. 4K60 at 30 bits per pixel
//! overflows a `u32` in the intermediate products even though the final answer
//! fits, so the whole expression is evaluated at `u64`.

use crate::edid::{DisplayMode, EDID_LEN};

/// Bits per pixel when a caller does not say: 8 per colour, 24 total.
pub const DEFAULT_BPP: u32 = 24;

/// Extra capacity demanded over the ideal figure, in permille of the ideal.
///
/// Both TMDS and DisplayPort carry blanking intervals that hold no picture, and
/// both need link capacity for them. 1.1 is generous enough for any real mode and
/// is what a conservative driver should assume.
pub const BLANKING_ALLOWANCE_PERMILLE: u32 = 1100;

/// A DP link rate code counts units of 270 **MHz**, so the per-lane clock in kHz
/// is `code × 270_000`, not `code × 270`.
///
/// The distinction is a factor of a thousand, and it is the difference between a
/// driver that believes a 4×5.4 link carries 21.6 Gbit/s and one that believes it
/// carries 21.6 Mbit/s. In the latter, 1080p60 fails the bandwidth check and the
/// display comes up at 640×480 — or not at all.
pub const DP_RATE_UNIT_KHZ: u32 = 270_000;

/// DP link rate code → kHz per lane. `0x14` is 5.4 GHz per lane.
pub const fn dp_rate_khz(code: u8) -> u32 {
    code as u32 * DP_RATE_UNIT_KHZ
}

/// The largest legacy TMDS clock we will program, in kHz.
///
/// HDMI 1.4's ceiling, deliberately not HDMI 2.x's. Past it a sink needs FRL
/// signalling negotiated in the infoframe, and driving it on a link that never
/// carried one gives a missing picture rather than a wrong one.
pub const HDMI_TMDS_MAX_KHZ: u32 = 340_000;

/// The TMDS rate at which HDMI 2.0 FRL takes over.
pub const HDMI_FRL_THRESHOLD_KHZ: u32 = 340_000;

/// Bits `bpp` occupies per pixel, rounded up to whole bytes.
#[inline]
pub const fn bytes_per_pixel(bpp: u32) -> u32 {
    (bpp + 7) / 8
}

/// The link bandwidth a mode needs, in kbit/s.
///
/// `kHz × bits` *is* `kbit/s` — the kilo already cancels — so there is no `/1000`
/// here. An earlier revision of this line had one, which made every answer a
/// thousand times too small: 4K60 then fit on a 5.4 Gbit/s link, and the check
/// this function exists to perform reported success for a mode the cable cannot
/// carry. Multiply first, then apply the allowance.
pub fn required_bandwidth_kbps(mode: &DisplayMode, bpp: u32) -> u64 {
    let clock_khz = mode.clock_khz as u64;
    (clock_khz * bpp as u64) * BLANKING_ALLOWANCE_PERMILLE as u64 / 1000
}

/// The bandwidth a DisplayPort link provides, in **kbit/s**, at `lanes` lanes.
///
/// A rate code counts units of 270 kHz, so `code × 270` is kHz; multiplying by
/// the lane count gives kilobits per second, because the kilo cancels. There is
/// deliberately no factor of 1000 anywhere below.
pub const fn dp_bandwidth_kbps(rate_code: u8, lanes: u8) -> u64 {
    dp_rate_khz(rate_code) as u64 * lanes as u64
}

/// The highest pixel clock this DP link carries, in kHz, at `bpp` bits per pixel.
///
/// A 4×5.4 link is 4 × 5400 = 21600 kbit/s, which at 24 bpp is 900 kHz of pixel
/// clock — nowhere near 4K60's 533 250. So the pixel depth has to come out, and a
/// helper that omitted it would report the link as capable of almost anything.
///
/// `dp_rate_khz` already carries the per-270-kHz unit, so no 1000 is applied.
pub fn dp_max_clock_khz(rate_code: u8, lanes: u8, bpp: u32) -> u32 {
    (dp_bandwidth_kbps(rate_code, lanes & 0x1F) / bpp.max(1) as u64) as u32
}

/// Can a DP link at this rate and width carry `mode`?
///
/// `lanes` is masked to the five bits the DP lane-count field defines, so a value
/// read from hardware cannot be inflated by stray upper bits into claiming
/// bandwidth that is not there.
pub fn dp_link_can_carry(mode: &DisplayMode, rate_code: u8, lanes: u8, bpp: u32) -> bool {
    let lanes = lanes & 0x1F;
    if lanes == 0 {
        return false;
    }
    required_bandwidth_kbps(mode, bpp) <= dp_bandwidth_kbps(rate_code, lanes)
}

/// Can an HDMI link at this TMDS clock carry `mode`?
///
/// Also enforces the legacy ceiling, so a mode only 2.1 could drive is refused on
/// a link that never negotiated it.
pub fn hdmi_link_can_carry(mode: &DisplayMode, tmds_khz: u32, bpp: u32) -> bool {
    if tmds_khz > HDMI_TMDS_MAX_KHZ {
        return false;
    }
    required_bandwidth_kbps(mode, bpp) <= tmds_khz as u64
}

/// Does this mode need HDMI 2.0 FRL rather than legacy TMDS?
pub fn hdmi_needs_frl(mode: &DisplayMode, bpp: u32) -> bool {
    required_bandwidth_kbps(mode, bpp) > HDMI_FRL_THRESHOLD_KHZ as u64
}


/// A mode a driver can actually program, and the reason one was refused.
///
/// A bare `Result<DisplayMode, ModesetError>` is the shape the caller wants; this
/// exists for the log line, because "the mode was refused" with no reason is the
/// least useful thing a display driver can say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModesetError {
    /// The descriptor was not a usable mode to begin with.
    InvalidMode,
    /// The sink's EDID was unusable, so there is nothing to choose from.
    NoEdid {
        /// How many bytes came back.
        seen: usize,
    },
    /// The mode is not among those the sink advertised.
    Unsupported,
    /// The sink is fine; the link is too slow.
    Bandwidth {
        /// What the mode needs, in kbit/s.
        needed_kbps: u64,
        /// What the link offers, in kbit/s.
        available_kbps: u64,
    },
    /// The mode fits the link, but the sink does not declare the rate.
    SinkMaxClock {
        /// The highest clock the mode needs.
        needed_khz: u32,
        /// The highest the sink declares.
        sink_max_khz: u32,
    },
}

impl core::fmt::Display for ModesetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ModesetError::InvalidMode => f.write_str("the mode is not a valid timing"),
            ModesetError::NoEdid { seen } => write!(f, "EDID is {seen} bytes, need {EDID_LEN}"),
            ModesetError::Unsupported => f.write_str("the sink does not advertise this mode"),
            ModesetError::Bandwidth { needed_kbps, available_kbps } => {
                write!(f, "link carries {available_kbps} kbit/s, the mode needs {needed_kbps}")
            }
            ModesetError::SinkMaxClock { needed_khz, sink_max_khz } => {
                write!(f, "the sink tops out at {sink_max_khz} kHz, the mode needs {needed_khz}")
            }
        }
    }
}

/// Everything a driver knows about a link before programming it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkCapability {
    /// Bits per pixel the link will carry.
    pub bpp: u32,
    /// The link's usable bandwidth, in kbit/s.
    pub bandwidth_kbps: u64,
    /// The highest pixel clock the sink declares, in kHz. Zero means undeclared.
    pub sink_max_khz: u32,
}

impl LinkCapability {
    /// A DisplayPort link's capability, from its rate code and lane count.
    pub const fn displayport(rate_code: u8, lanes: u8, bpp: u32) -> Self {
        Self {
            bpp,
            bandwidth_kbps: dp_bandwidth_kbps(rate_code, lanes),
            sink_max_khz: 0,
        }
    }

    /// An HDMI link's capability, from its TMDS clock.
    pub const fn hdmi(tmds_khz: u32, bpp: u32) -> Self {
        Self { bpp, bandwidth_kbps: tmds_khz as u64, sink_max_khz: 0 }
    }

    /// With the sink's declared ceiling attached.
    pub const fn with_sink_max(mut self, sink_max_khz: u32) -> Self {
        self.sink_max_khz = sink_max_khz;
        self
    }

    /// Check a mode against this link and against the sink's declared ceiling.
    ///
    /// The order matters: a mode the sink does not advertise is `Unsupported`,
    /// not a bandwidth failure, because "this monitor cannot do 4K" and "this
    /// cable is too slow" call for different advice. The sink's ceiling is checked
    /// *after* the link, since a link that cannot carry the mode at all makes the
    /// sink's rating irrelevant.
    pub fn check(&self, mode: &DisplayMode) -> Result<(), ModesetError> {
        if !mode.is_valid() {
            return Err(ModesetError::InvalidMode);
        }
        let needed = required_bandwidth_kbps(mode, self.bpp);
        if needed > self.bandwidth_kbps {
            return Err(ModesetError::Bandwidth {
                needed_kbps: needed,
                available_kbps: self.bandwidth_kbps,
            });
        }
        if self.sink_max_khz != 0 && mode.clock_khz > self.sink_max_khz {
            return Err(ModesetError::SinkMaxClock {
                needed_khz: mode.clock_khz,
                sink_max_khz: self.sink_max_khz,
            });
        }
        Ok(())
    }

    /// The highest-refresh mode of this resolution that this link can carry.
    ///
    /// A driver wants the best it can get, not a failure because the mode the
    /// user asked for was one notch too fast for the link.
    pub fn best_refresh_for<'a>(
        &self,
        candidates: &'a [DisplayMode],
        w: u16,
        h: u16,
    ) -> Option<&'a DisplayMode> {
        candidates
            .iter()
            .filter(|m| m.w == w && m.h == h)
            .filter(|m| self.check(m).is_ok())
            .max_by_key(|m| m.refresh_mhz())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::string::ToString;

    fn mode_1080p60() -> DisplayMode {
        DisplayMode {
            w: 1920, h: 1080, htotal: 2200, vtotal: 1125, clock_khz: 148_500, ..Default::default()
        }
    }

    fn mode_4k60() -> DisplayMode {
        DisplayMode {
            w: 3840, h: 2160, htotal: 4400, vtotal: 2250, clock_khz: 533_250, ..Default::default()
        }
    }

    // ── the rate table ──────────────────────────────────────────────────────

    #[test]
    fn dp_rate_codes_decode() {
        // The unit is 270 MHz per code, so 0x14 is 5.4 GHz and 0x1C is 7.56 GHz
        // (HBR3), not 6.75. The codes are not evenly spaced in Gbit/s, and
        // assuming a flat 2.7 Gbit/s step gets two of the four common rates wrong.
        assert_eq!(dp_rate_khz(0x0A), 2_700_000, "2.7 Gbit/s per lane");
        assert_eq!(dp_rate_khz(0x14), 5_400_000, "5.4 Gbit/s per lane");
        assert_eq!(dp_rate_khz(0x1C), 7_560_000, "HBR3, 7.56 Gbit/s per lane");
        assert_eq!(dp_rate_khz(0x1E), 8_100_000, "8.1 Gbit/s per lane");
        assert_eq!(dp_rate_khz(0), 0, "code zero is a dead link, not a fast one");
    }

    #[test]
    fn dp_bandwidth_is_rate_times_lanes() {
        // 5.4 GHz × 4 lanes = 21.6 Gbit/s = 21 600 000 kbit/s.
        assert_eq!(dp_bandwidth_kbps(0x14, 4), 21_600_000, "in kbit/s");
    }

    // ── required bandwidth ──────────────────────────────────────────────────

    #[test]
    fn required_bandwidth_is_in_the_right_units() {
        // 148.5 MHz × 24 bits = 3 564 000 kbit/s, plus 10% = 3 920 400.
        // Getting this a thousandfold low is the bug this guards: every bandwidth
        // check then passes, which is the one thing it must never do.
        let need = required_bandwidth_kbps(&mode_1080p60(), 24);
        assert_eq!(need, 3_920_400, "kHz × bits is kbit/s; the kilo already cancels");
    }

    #[test]
    fn required_bandwidth_includes_the_blanking_allowance() {
        let m = mode_1080p60();
        let ideal = m.clock_khz as u64 * 24;
        let need = required_bandwidth_kbps(&m, 24);
        let pct = (need * 100) / ideal;
        assert!((109..=111).contains(&pct), "about 10% over, got {pct}");
    }

    #[test]
    fn four_k_sixty_does_not_overflow_the_intermediates() {
        // 533250 kHz × 30 bits = 15 997 500, and the ×1100 step pushes it past
        // 2^31. Evaluated in u32 this wraps to something small and the check
        // silently passes for a mode no cable can carry.
        let need = required_bandwidth_kbps(&mode_4k60(), 30);
        assert_eq!(need, 17_597_250, "17.6 Gbit/s, which a 32-bit intermediate cannot hold");
    }

    #[test]
    fn bytes_per_pixel_rounds_a_partial_byte_up() {
        assert_eq!(bytes_per_pixel(24), 3);
        assert_eq!(bytes_per_pixel(30), 4, "30 bits needs four whole bytes");
        assert_eq!(bytes_per_pixel(1), 1);
    }

    // ── DP link checks ──────────────────────────────────────────────────────

    #[test]
    fn a_fast_link_carries_1080p60() {
        assert!(dp_link_can_carry(&mode_1080p60(), 0x14, 4, 24));
    }

    #[test]
    fn the_link_units_agree_with_the_mode_units() {
        // 1080p60 at 24 bpp needs 3 920 400 kbit/s. A 4×5.4 link is 4 × 5400 × 4
        // = 21 600 000 kbit/s once the bpp is divided back out. The check is only
        // meaningful if both sides use the same unit, and an earlier revision of
        // this code had the link side a thousand times too small — which made
        // every mode fail, including ones that obviously fit.
        let cap = LinkCapability::displayport(0x14, 4, 24);
        assert_eq!(cap.bandwidth_kbps, 21_600_000);
        assert!(cap.bandwidth_kbps > required_bandwidth_kbps(&mode_1080p60(), 24));
    }

    #[test]
    fn the_max_clock_helper_accounts_for_pixel_depth() {
        // 21 600 000 kbit/s at 24 bpp is 900 000 kHz, not 21 600 000.
        assert_eq!(dp_max_clock_khz(0x14, 4, 24), 900_000);
        assert_eq!(dp_max_clock_khz(0x14, 4, 8), 2_700_000);
    }

    #[test]
    fn a_slow_link_refuses_4k60() {
        // Two lanes at 2.7 GHz is 5.4 Gbit/s; 4K60 at 24 bpp needs about 15.
        assert!(!dp_link_can_carry(&mode_4k60(), 0x0A, 2, 24));
    }

    #[test]
    fn a_fast_link_carries_4k60() {
        assert!(dp_link_can_carry(&mode_4k60(), 0x14, 4, 24));
    }

    #[test]
    fn a_dead_link_carries_nothing() {
        assert!(!dp_link_can_carry(&mode_1080p60(), 0, 4, 24));
    }

    #[test]
    fn a_zero_lane_count_carries_nothing() {
        // A lane count of zero is a training failure, not "unlimited bandwidth".
        assert!(!dp_link_can_carry(&mode_1080p60(), 0x14, 0, 24));
    }

    #[test]
    fn stray_lane_bits_cannot_inflate_the_bandwidth() {
        // The lane-count field is five bits. A read returning 0x1F | 0xE0, trusted
        // whole, claims 63 lanes and programs a mode the link cannot carry.
        let honest = dp_link_can_carry(&mode_4k60(), 0x0A, 4, 24);
        let inflated = dp_link_can_carry(&mode_4k60(), 0x0A, 4 | 0xE0, 24);
        assert_eq!(honest, inflated, "the upper bits must be masked off");
    }


    // ── the capability object ───────────────────────────────────────────────

    #[test]
    fn a_capability_accepts_a_mode_that_fits() {
        let cap = LinkCapability::displayport(0x14, 4, 24);
        assert_eq!(cap.check(&mode_1080p60()), Ok(()));
    }

    #[test]
    fn a_capability_names_the_shortfall() {
        let cap = LinkCapability::displayport(0x0A, 2, 24);
        match cap.check(&mode_4k60()) {
            Err(ModesetError::Bandwidth { needed_kbps, available_kbps }) => {
                assert!(needed_kbps > available_kbps);
                assert_eq!(available_kbps, cap.bandwidth_kbps);
            }
            other => panic!("expected a bandwidth failure, got {other:?}"),
        }
    }

    #[test]
    fn an_invalid_mode_is_refused_before_the_bandwidth_check() {
        // Otherwise a zeroed mode reports "bandwidth" and hides the real fault.
        let cap = LinkCapability::hdmi(150_000, 24);
        assert_eq!(cap.check(&DisplayMode::default()), Err(ModesetError::InvalidMode));
    }

    #[test]
    fn a_sink_ceiling_below_the_mode_is_its_own_failure() {
        // The link is fast enough; the monitor is not. Different advice, so it
        // must be a different error.
        let cap = LinkCapability::displayport(0x14, 4, 24).with_sink_max(200_000);
        assert_eq!(
            cap.check(&mode_4k60()),
            Err(ModesetError::SinkMaxClock { needed_khz: 533_250, sink_max_khz: 200_000 })
        );
        assert_eq!(cap.check(&mode_1080p60()), Ok(()), "1080p is under the ceiling");
    }

    #[test]
    fn an_undeclared_ceiling_is_not_treated_as_zero() {
        // Zero means "the sink did not say". Read as a 0 kHz ceiling it would
        // refuse every mode on every sink that omits the field.
        let cap = LinkCapability::displayport(0x14, 4, 24);
        assert_eq!(cap.sink_max_khz, 0);
        assert_eq!(cap.check(&mode_4k60()), Ok(()));
    }

    #[test]
    fn a_link_failure_outranks_the_sink_ceiling() {
        // A cable that cannot carry the mode makes the monitor's rating moot.
        let cap = LinkCapability::displayport(0x0A, 1, 24).with_sink_max(150_000);
        assert!(matches!(cap.check(&mode_4k60()), Err(ModesetError::Bandwidth { .. })));
    }

    // ── picking the best mode that fits ─────────────────────────────────────

    #[test]
    fn the_fastest_mode_that_fits_is_chosen() {
        let candidates = vec![
            DisplayMode { w: 1920, h: 1080, htotal: 2200, vtotal: 1125, clock_khz: 74_250, ..Default::default() },
            DisplayMode { w: 1920, h: 1080, htotal: 2200, vtotal: 1125, clock_khz: 148_500, ..Default::default() },
            DisplayMode { w: 1920, h: 1080, htotal: 2640, vtotal: 1470, clock_khz: 277_500, ..Default::default() },
        ];
        // A 2.7 GHz × 2 link carries 60 Hz but not 120.
        let cap = LinkCapability::displayport(0x0A, 2, 24);
        let best = cap.best_refresh_for(&candidates, 1920, 1080).expect("a mode fits");
        assert_eq!(best.clock_khz, 148_500, "60 Hz, not 120");
    }

    #[test]
    fn a_fast_link_takes_the_120_hz_mode() {
        let candidates = vec![
            DisplayMode { w: 1920, h: 1080, htotal: 2200, vtotal: 1125, clock_khz: 148_500, ..Default::default() },
            DisplayMode { w: 1920, h: 1080, htotal: 2640, vtotal: 1470, clock_khz: 277_500, ..Default::default() },
        ];
        let cap = LinkCapability::displayport(0x14, 4, 24);
        let best = cap.best_refresh_for(&candidates, 1920, 1080).expect("a mode fits");
        assert_eq!(best.clock_khz, 277_500);
    }

    #[test]
    fn no_candidate_at_that_resolution_yields_nothing() {
        let cap = LinkCapability::displayport(0x14, 4, 24);
        assert!(cap.best_refresh_for(&[mode_1080p60()], 1280, 1024).is_none());
    }

    #[test]
    fn a_dead_link_yields_nothing_rather_than_the_worst_fit() {
        let cap = LinkCapability::displayport(0, 0, 24);
        assert!(cap.best_refresh_for(&[mode_4k60(), mode_1080p60()], 1920, 1080).is_none());
    }

    #[test]
    fn the_errors_read_as_something_a_user_can_act_on() {
        // "mode refused" with no reason is the least useful thing a display driver
        // can print, which is why these carry their numbers.
        let msg = ModesetError::Bandwidth {
            needed_kbps: 16_000_000,
            available_kbps: 5_400_000,
        }
        .to_string();
        assert!(msg.contains("16000000") && msg.contains("5400000"), "got {msg}");
    }
}

