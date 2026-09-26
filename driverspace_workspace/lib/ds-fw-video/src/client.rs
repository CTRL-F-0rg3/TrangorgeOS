//! The video client: typed wrappers over `kapi-syscall` IPC.
//!
//! Deliberately thin. Every method builds a `DsMsg`, sends it, and turns the
//! reply into a typed value or a `DsError`. A client that has to encode a
//! payload by hand is a client that will encode it wrong, and the video payloads
//! are exactly the kind that are easy to get subtly wrong - a `FrameInfo` with
//! the timestamps in the wrong order is still the right length and still decodes.
//!
//! # The two stubs, stated plainly
//!
//! [`VideoClient::engine_info`] and [`VideoClient::acquire`] are the only two
//! methods that do not return a decoded payload, because both need the
//! transport's reply-buffer copy and this client has no handle on that buffer.
//! Neither fabricates a value: `engine_info` leaves `out` untouched and
//! `acquire` answers `Ok(None)`. Both are marked at the call site. They are not
//! finished, and a caller that reads a payload from either is reading
//! uninitialised memory.

use kapi_abi::{
    DsCmd, DsError, DsMsg,
    payloads::video::{EngineInfo, FrameInfo, SessionRequest, SessionState, SinkRequest, SubmitRequest},
    wire::Encoder,
};
use kapi_syscall::sys_ipc_call;

use crate::codec;
use crate::types::{BufferId, EngineId, StreamHandle};

/// A connection to the video driver.
///
/// No driver handle, deliberately. `sys_ipc_call` takes the command and three
/// argument registers and routes to the calling process's endpoint, so a
/// "which driver" field here would look like it worked while being ignored.
/// A client that needs to talk to two video drivers needs two transports, and
/// that is a transport question rather than a payload one.
#[derive(Clone, Copy, Debug, Default)]
pub struct VideoClient;

impl VideoClient {
    /// Connect to the video driver.
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    /// Send one message and translate a non-zero status.
    ///
    /// Takes a whole `DsMsg` so each method reads like the request it is
    /// describing, and unpacks it here - the one place that knows the transport's
    /// signature is `(cmd, arg0, arg1, arg2)` rather than a message.
    fn call(&self, msg: DsMsg) -> Result<DsMsg, DsError> {
        let Some(cmd) = DsCmd::from_u32(msg.cmd as u32) else {
            return Err(DsError::InvalidMessage);
        };
        let reply = sys_ipc_call(cmd, msg.arg0, msg.arg1, msg.arg2);
        if reply.is_ok() {
            Ok(reply)
        } else {
            Err(DsError::from_u32(reply.status as u32))
        }
    }

    /// How many engines the driver exposes.
    pub fn engine_count(&self) -> Result<u32, DsError> {
        let mut msg = DsMsg::new(DsCmd::VideoEngineEnumerate as u32);
        msg.arg0 = u32::MAX as u64;
        let reply = self.call(msg)?;
        Ok(reply.arg1 as u32)
    }

    /// Read one engine's descriptor.
    ///
    /// # Incomplete
    ///
    /// Sends the request and validates the reply, but does not write the
    /// descriptor into `out`: the payload arrives through the transport's reply
    /// buffer, which this client cannot reach. `out` is left **untouched**
    /// rather than defaulted, because a caller handed a default would report
    /// "an engine with no capabilities" instead of "the read failed", and those
    /// two bugs are diagnosed in completely different places.
    pub fn engine_info(&self, engine: EngineId, out: &mut EngineInfo) -> Result<(), DsError> {
        let mut msg = DsMsg::new(DsCmd::VideoEngineQuery as u32);
        msg.arg0 = u64::from(engine.index());
        self.call(msg)?;
        let _ = out;
        Ok(())
    }

    /// Open a decoding or encoding session.
    pub fn create_session(&self, request: &SessionRequest) -> Result<StreamHandle, DsError> {
        let mut payload = [0u8; codec::MAX_PAYLOAD_LEN];
        let mut enc = Encoder::new(&mut payload);
        if !codec::encode_session_request(&mut enc, request) {
            return Err(DsError::BufferTooSmall);
        }
        let mut msg = DsMsg::new(DsCmd::VideoSessionCreate as u32);
        let len = enc.pos();
        msg.arg0 = &payload as *const _ as u64;
        msg.arg1 = len as u64;
        Ok(StreamHandle::from(self.call(msg)?.arg0 as u32))
    }

    /// Close a session and release every surface it owned.
    pub fn close_session(&self, stream: StreamHandle) -> Result<(), DsError> {
        let mut msg = DsMsg::new(DsCmd::VideoSessionDestroy as u32);
        msg.arg0 = u64::from(stream.raw());
        self.call(msg).map(|_| ())
    }

    /// Ask what a session is doing.
    pub fn session_state(&self, stream: StreamHandle) -> Result<SessionState, DsError> {
        let mut msg = DsMsg::new(DsCmd::VideoSessionState as u32);
        msg.arg0 = u64::from(stream.raw());
        self.call(msg).map(|r| SessionState::from_u32(r.arg0 as u32))
    }

    /// Hand one compressed frame to the engine, returning a ticket.
    pub fn submit(&self, stream: StreamHandle, request: &SubmitRequest) -> Result<u32, DsError> {
        let mut payload = [0u8; codec::MAX_PAYLOAD_LEN];
        let mut enc = Encoder::new(&mut payload);
        if !codec::encode_submit_request(&mut enc, request) {
            return Err(DsError::BufferTooSmall);
        }
        let mut msg = DsMsg::new(DsCmd::VideoSubmit as u32);
        // The bitstream is addressed, not inlined: an I-frame is megabytes and
        // `DsMsg` is 64 bytes. `SubmitRequest` says where it lives.
        let len = enc.pos();
        msg.arg0 = &payload as *const _ as u64;
        msg.arg1 = len as u64;
        msg.arg2 = u64::from(stream.raw());
        Ok(self.call(msg)?.arg0 as u32)
    }

    /// Collect the frame `ticket` produced, if it is ready.
    ///
    /// `Ok(None)` means "not yet", which is the normal case between a submit and
    /// its completion. It is deliberately not an error: modelling asynchrony as a
    /// failure makes every client spin on a status code.
    ///
    /// # Incomplete
    ///
    /// The presence check is real - the driver puts it in `arg0` - but the
    /// `FrameInfo` body still needs the transport's reply buffer. It is never
    /// fabricated, so this currently always answers `Ok(None)`.
    pub fn acquire(&self, stream: StreamHandle, ticket: u32) -> Result<Option<FrameInfo>, DsError> {
        let mut msg = DsMsg::new(DsCmd::VideoAcquire as u32);
        msg.arg0 = u64::from(stream.raw());
        msg.arg1 = u64::from(ticket);
        let reply = self.call(msg)?;
        if reply.arg0 == 0 {
            return Ok(None);
        }
        Ok(None)
    }

    /// Hand a surface back so the driver's pool may reuse it.
    pub fn release(&self, buffer: BufferId) -> Result<(), DsError> {
        let mut msg = DsMsg::new(DsCmd::VideoRelease as u32);
        msg.arg0 = u64::from(buffer.raw());
        self.call(msg).map(|_| ())
    }

    /// How many frames this session has dropped because a consumer was slow.
    pub fn dropped_frames(&self, stream: StreamHandle) -> Result<u32, DsError> {
        let mut msg = DsMsg::new(DsCmd::VideoDropCount as u32);
        msg.arg0 = u64::from(stream.raw());
        Ok(self.call(msg)?.arg0 as u32)
    }

    /// Register a frame consumer - a compositor, an NPU, a capture writer.
    pub fn register_sink(
        &self,
        stream: StreamHandle,
        request: &SinkRequest,
    ) -> Result<crate::sink::SinkId, DsError> {
        let mut payload = [0u8; codec::MAX_PAYLOAD_LEN];
        let mut enc = Encoder::new(&mut payload);
        if !codec::encode_sink_request(&mut enc, request) {
            return Err(DsError::BufferTooSmall);
        }
        let mut msg = DsMsg::new(DsCmd::VideoSinkRegister as u32);
        let len = enc.pos();
        msg.arg0 = &payload as *const _ as u64;
        msg.arg1 = len as u64;
        msg.arg2 = u64::from(stream.raw());
        Ok(crate::sink::SinkId::new(self.call(msg)?.arg0 as u32))
    }

    /// Remove a frame consumer. Its `close` runs before this returns.
    pub fn unregister_sink(
        &self,
        stream: StreamHandle,
        sink: crate::sink::SinkId,
    ) -> Result<(), DsError> {
        let mut msg = DsMsg::new(DsCmd::VideoSinkUnregister as u32);
        msg.arg0 = u64::from(sink.raw());
        msg.arg1 = u64::from(stream.raw());
        self.call(msg).map(|_| ())
    }
}

