//! The power device-class contract.
//!
//! Drivers (`drivers/Power-TrangorgeOS`) implement [`PowerBackend`]; the
//! framework routes requests onto it. The framework itself contains no hardware
//! knowledge and does not know whether it is talking to ACPI, an SMBus Smart
//! Battery, or a fuel gauge chip.
//!
//! ## One trait, three back ends, none mandatory
//!
//! This is the whole point of the trait existing. A machine may have one power
//! source or several, from one transport or three, and the code above it - the
//! indicator, the low-battery warning, the charge policy - must be identical in
//! all those cases. So the trait is defined in terms of *what a source reports*,
//! never *how it is reached*.
//!
//! ## Sampling is push-based, and that is deliberate
//!
//! [`PowerBackend::sample`] is called by the driver on a timer rather than
//! interrupting on change. Battery state does not change fast enough to justify
//! an interrupt, and a polling loop is dramatically easier to reason about when
//! the thing being polled is a piece of firmware that can return garbage.

use kapi_abi::{
    DsError,
    payloads::power::{PowerInfo, PowerPolicy, PowerStatus},
};

/// One way of reaching a power source.
///
/// Object-safe, so a driver holds `Box<dyn PowerBackend>` and never names a
/// concrete transport.
pub trait PowerBackend {
    /// A short name for logs, e.g. `"acpi"` or `"smbus"`.
    ///
    /// A diagnostic only. Nothing may branch on it: a driver that checks for
    /// `"acpi"` has reintroduced exactly the coupling the trait removes.
    fn name(&self) -> &'static str;

    /// How many sources this back end can see.
    fn source_count(&self) -> usize;

    /// Read the static descriptor of source `index`.
    ///
    /// Returns `false` and leaves `out` untouched when out of range, so a caller
    /// walking the list cannot be handed a half-written descriptor.
    fn info(&self, index: usize, out: &mut PowerInfo) -> bool;

    /// Take one reading.
    ///
    /// `Err(DsError::DeviceNotFound)` means the source is gone, which on a hot-
    /// unplugged laptop is normal and not an error the caller should surface.
    /// `Err(DsError::DeviceFault)` means the transport failed, which is.
    fn sample(&mut self, index: usize, out: &mut PowerStatus) -> Result<(), DsError>;

    /// Is the source still reachable?
    ///
    /// Checked separately from [`PowerBackend::sample`] because a pack that has
    /// been removed should be reported as *absent* rather than as a transport
    /// failure, and a driver that conflates the two makes an unplug look like a
    /// hardware fault.
    fn present(&mut self, index: usize) -> bool;

    /// Apply a charge policy to this source.
    ///
    /// # The driver clamps, and the caller must too
    ///
    /// Implementations are required to clamp the policy to the source's own
    /// window (see [`PowerPolicy::clamp_to`]) and to refuse anything the cell
    /// was not validated for. This is the one method in the power contract where
    /// a bad argument can damage hardware, and it is why
    /// `CapId::POWER_POLICY` is separate from everything else.
    ///
    /// A back end with no way to set a threshold - most fuel gauges do not
    /// charge themselves - returns `Err(DsError::NotSupported)`. It must not
    /// return `Ok(())` and ignore the request, because the user will believe
    /// their setting took effect.
    fn set_policy(&mut self, index: usize, policy: &PowerPolicy) -> Result<(), DsError>;
}
