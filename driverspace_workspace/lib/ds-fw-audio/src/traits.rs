//! The audio device-class contract.
//!
//! Drivers (`drivers/Alsa-TrangorgeOS`) implement this trait; `AudioService`
//! routes IPC requests onto these methods. The framework itself contains no
//! hardware knowledge.
//!
//! ## Shape of the contract
//!
//! The method set follows ALSA's own decomposition, because that is what a
//! ported tool expects to drive:
//!
//! 1. **Enumerate** - cards, PCM endpoints, mixer elements, jacks, hwdep.
//! 2. **Negotiate** - query and narrow hardware parameters field by field.
//! 3. **Run** - open, prepare, start, transfer, drop, close.
//! 4. **Configure** - read and write mixer elements.
//!
//! Two properties are deliberate. Parameter negotiation is *stateful* and lives
//! on the handle, so the driver can reject an impossible combination instead
//! of silently substituting something else. And the sample-transfer methods
//! take frames, not bytes, so the driver owns the format arithmetic and a
//! caller cannot desynchronise the ring by miscounting.
//!
//! ## Resource ownership
//!
//! Implementations are `no_std` and self-contained: MMIO, DMA memory and IRQs
//! must be requested from `ds-manager` with a `DsCmd`. Touching a physical
//! address directly is not allowed. The `Alsa-TrangorgeOS` driver does this
//! through its `Platform` trait.

use kapi_abi::{
    DsError,
    payloads::audio::{
        HwParamsResult, JackState, MixerSelem, MixerValue, PcmInfo, PcmOpenPayload, PcmParams,
        PcmState,
    },
};

use crate::types::{DropMode, JackId, MixerId, PcmHandle, PcmId, Transfer};

/// An audio device that `ds-manager` can schedule.
///
/// The trait is object-safe: a service owns `Box<dyn AudioDevice>` and never
/// needs to know the driver's concrete type.
pub trait AudioDevice {
    // ── enumeration ───────────────────────────────────────────────────────────────────────────────────

    /// Number of PCM endpoints the driver exposes.
    fn pcm_count(&self) -> usize;

    /// Read the descriptor of PCM endpoint `pcm`.
    ///
    /// Returns `false` and leaves `out` untouched when out of range, so a
    /// caller walking the list cannot be handed a half-written descriptor.
    fn pcm_info(&self, pcm: PcmId, out: &mut PcmInfo) -> bool;

    /// Number of mixer elements the driver exposes.
    fn mixer_count(&self) -> usize;

    /// Read the descriptor of mixer element `mixer`.
    fn mixer_info(&self, mixer: MixerId, out: &mut MixerSelem) -> bool;

    /// Number of jack / plug detectors the driver exposes.
    fn jack_count(&self) -> usize;

    /// Read the state of jack `jack`.
    fn jack_state(&mut self, jack: JackId, out: &mut JackState) -> bool;

    /// Number of hwdep nodes the driver exposes.
    fn hwdep_count(&self) -> usize;

    // ── parameter negotiation ─────────────────────────────────────────────────────────────────────────

    /// Read or narrow one hardware-parameter field of an open PCM.
    ///
    /// A query with `set == false` reports the current candidates; a query with
    /// `set == true` narrows them to `value` and fails with
    /// [`DsError::NotSupported`] if the combination is impossible. Narrowing
    /// is order-dependent and cumulative, exactly like
    /// `snd_pcm_hw_params_set_*` followed by `..._test_*`.
    fn hw_params(
        &mut self,
        pcm: PcmHandle,
        field: u32,
        set: bool,
        value: u64,
        out: &mut HwParamsResult,
    ) -> Result<(), DsError>;

    /// The parameters currently negotiated for `pcm`.
    fn params(&self, pcm: PcmHandle, out: &mut PcmParams) -> Result<(), DsError>;

    // ── stream lifecycle ───────────────────────────────────────────────────────────────────────────────

    /// Open a PCM endpoint and return a handle to it.
    fn open(&mut self, request: &PcmOpenPayload) -> Result<PcmHandle, DsError>;

    /// Release a handle and everything its buffer owns.
    fn close(&mut self, pcm: PcmHandle) -> Result<(), DsError>;

    /// Move a stream to `Prepared`; the buffer becomes writable.
    fn prepare(&mut self, pcm: PcmHandle) -> Result<(), DsError>;

    /// Move a `Prepared` stream to `Running`.
    fn start(&mut self, pcm: PcmHandle) -> Result<(), DsError>;

    /// Stop a running stream, either discarding or playing out the tail.
    fn drop_stream(&mut self, pcm: PcmHandle, mode: DropMode) -> Result<(), DsError>;

    /// The current state of `pcm`.
    fn state(&self, pcm: PcmHandle) -> Result<PcmState, DsError>;

    /// Write `frames` frames of interleaved samples from `data`.
    ///
    /// `data` holds exactly `params(pcm).bytes_for(frames)` bytes. The driver
    /// reports how many frames actually moved, and a short count is what puts
    /// the stream into `XRun`.
    fn write(&mut self, pcm: PcmHandle, data: &[u8], frames: u32) -> Result<Transfer, DsError>;

    /// Read `frames` frames of interleaved samples into `data`.
    ///
    /// `data` is pre-sized by the caller to `params(pcm).bytes_for(frames)`.
    fn read(&mut self, pcm: PcmHandle, data: &mut [u8], frames: u32) -> Result<Transfer, DsError>;

    // ── mixer ──────────────────────────────────────────────────────────────────────────────────────────

    /// Read the current value of mixer element `mixer`.
    fn mixer_read(&mut self, mixer: MixerId, out: &mut MixerValue) -> Result<(), DsError>;

    /// Write a new value to mixer element `mixer`.
    ///
    /// The driver must reject an element that does not advertise
    /// `CtlAccess::WRITE` with [`DsError::PermissionDenied`].
    fn mixer_write(&mut self, mixer: MixerId, value: &MixerValue) -> Result<(), DsError>;
}
