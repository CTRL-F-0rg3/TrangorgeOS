//! The video device-class contract.
//!
//! Drivers (`drivers/Video-TrangorgeOS`) implement this trait; `VideoService`
//! routes IPC requests onto these methods. The framework itself contains no
//! hardware knowledge and names no vendor.
//!
//! ## Shape of the contract
//!
//! The method set follows the ALSA decomposition again, for the same reason the
//! audio one does: it is a decomposition a ported tool already knows.
//!
//! 1. **Enumerate** - the engines, their capabilities and their limits.
//! 2. **Negotiate** - a session request, checked against the engine's limits.
//! 3. **Run** - open, submit, acquire, release, close.
//!
//! Two properties are deliberate and both come from video hardware rather than
//! from the analogy. A submit is **asynchronous**: it returns a ticket, and the
//! frame appears later at `acquire`, so the driver is never blocked waiting for
//! a consumer. And a surface is owned by the **driver**, not by the caller,
//! which is what makes a zero-copy hand-off possible at all.
//!
//! ## Resource ownership
//!
//! Implementations are `no_std` and self-contained: MMIO, DMA memory and IRQs
//! must be requested from `ds-manager` with a `DsCmd`. Touching a physical
//! address directly is not allowed. `Video-TrangorgeOS` does this through its
//! `Platform` trait.

use kapi_abi::{
    DsError,
    payloads::video::{EngineInfo, FrameInfo, SessionRequest, SessionState, SinkRequest, SubmitRequest},
};

use crate::types::{BufferId, EngineId, StreamHandle};

/// A video engine that `ds-manager` can schedule.
///
/// The trait is object-safe: a service owns `Box<dyn VideoEngine>` and never
/// needs to know the driver's concrete type.
pub trait VideoEngine {
    // ── enumeration ────────────────────────────────────────────────────────

    /// How many engines the driver exposes.
    fn engine_count(&self) -> u32;

    /// Read the descriptor of engine `engine`.
    ///
    /// Returns `false` and leaves `out` untouched when out of range, so a
    /// caller walking the list cannot be handed a half-written descriptor.
    fn engine_info(&self, engine: EngineId, out: &mut EngineInfo) -> bool;

    /// Does this engine decode `codec`?
    ///
    /// Separate from `engine_info` because the codec set is the one thing a
    /// client checks before building a pipeline, and putting sixteen bits of
    /// codec list in a `#[repr(C)]` descriptor would make that descriptor the
    /// size of a packet.
    fn supports_codec(&self, engine: EngineId, codec: u32) -> bool;

    // ── session lifecycle ──────────────────────────────────────────────────

    /// Open a session and return a handle to it.
    ///
    /// The driver must reject a request the engine cannot satisfy - a codec it
    /// does not have, a resolution above its limit - rather than silently
    /// substituting something else. `SessionRequest::surfaces` is the
    /// exception: it is a lower bound, and the driver clamps it.
    fn create_session(&mut self, request: &SessionRequest) -> Result<StreamHandle, DsError>;

    /// Release a session and every surface its pool owns.
    fn close_session(&mut self, stream: StreamHandle) -> Result<(), DsError>;

    /// The current state of a session.
    fn session_state(&self, stream: StreamHandle) -> Result<SessionState, DsError>;

    // ── the data path ──────────────────────────────────────────────────────

    /// Hand one compressed frame to the engine, returning a ticket.
    ///
    /// Returns immediately. The frame appears later through `acquire`. A driver
    /// that blocks here has defeated the only reason hardware decoders exist,
    /// which is that they run in parallel with everything else.
    fn submit(
        &mut self,
        stream: StreamHandle,
        request: &SubmitRequest,
    ) -> Result<u32, DsError>;

    /// Take the frame `ticket` produced, if it is ready.
    ///
    /// Returns `Ok(None)` when it is not ready *yet*, which is the normal case
    /// and not an error. `acquire` failing is a real failure - the session is
    /// in `Error`, or the ticket was never issued.
    fn acquire(
        &mut self,
        stream: StreamHandle,
        ticket: u32,
        out: &mut FrameInfo,
    ) -> Result<Option<BufferId>, DsError>;

    /// Hand a surface back so the pool may reuse it.
    fn release(&mut self, buffer: BufferId) -> Result<(), DsError>;

    /// How many frames have been dropped because a consumer was too slow.
    ///
    /// A cumulative counter rather than a status flag, because the interesting
    /// question is never "did we drop one" but "what fraction".
    fn dropped_frames(&self, stream: StreamHandle) -> Result<u32, DsError>;

    // ── sinks ──────────────────────────────────────────────────────────────

    /// Register a frame consumer and return its id.
    ///
    /// The engine asks each sink whether it accepts the output format before
    /// committing, so a compositor that cannot take NV12 is bound to a session
    /// configured for BGRA instead of being fed frames it will drop.
    fn register_sink(
        &mut self,
        stream: StreamHandle,
        request: &SinkRequest,
    ) -> Result<crate::sink::SinkId, DsError>;

    /// Remove a sink, running its `close`.
    fn unregister_sink(
        &mut self,
        stream: StreamHandle,
        sink: crate::sink::SinkId,
    ) -> Result<(), DsError>;
}
