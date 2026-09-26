//! The video server-side dispatcher.
//!
//! [`VideoService`] turns one `DsMsg` request into one [`VideoEngine`] call and
//! encodes the outcome as a [`Reply`]. Capability checks live here, so driver
//! implementations neither need to nor should repeat permission logic.
//!
//! ## Payload passing convention
//!
//! The transport copies the request payload from `msg.arg0` (length
//! `msg.arg1`) into the caller-provided `Request::payload` slice; the service
//! writes its reply payload into the `out` buffer passed to `dispatch`. That
//! keeps raw pointers out of the framework and lets unit tests feed plain byte
//! slices.
//!
//! ## Why the capability split is four bits
//!
//! Not one `VIDEO` bit. The four stages of a video pipeline have genuinely
//! different risk, and lumping them means a client that may only *display*
//! frames can also make the DMA engine decode arbitrary bitstreams - which is
//! the capability that actually matters for privilege.
//!
//! * `VIDEO_ENUMERATE` - read descriptors and limits. Harmless.
//! * `VIDEO_DECODE` - create sessions, submit bitstreams, read frames. The
//!   privileged one: it makes hardware parse attacker-controlled bytes.
//! * `VIDEO_ENCODE` - drive an encoder. Separate because decode-only hardware
//!   is extremely common, and the two have different failure modes.
//! * `VIDEO_SINK` - receive frames. The consuming privilege; a machine with no
//!   compositor and no NPU grants it to nobody.

use alloc::boxed::Box;

use kapi_abi::{CapId, DsCmd, DsError, DsMsg, Handle, wire::Decoder};

use crate::codec;
use crate::traits::VideoEngine;
use crate::types::EngineId;

/// One video request: the message header plus its payload, already unpacked.
#[derive(Clone, Copy)]
pub struct Request<'a> {
    /// The message as it arrived.
    pub msg: DsMsg,
    /// The request payload, already copied out of shared memory.
    pub payload: &'a [u8],
}

impl<'a> Request<'a> {
    #[inline]
    pub const fn new(msg: DsMsg, payload: &'a [u8]) -> Self {
        Self { msg, payload }
    }

    /// The command declared by the message header.
    #[inline]
    pub fn command(&self) -> Option<DsCmd> {
        DsCmd::from_u32(self.msg.cmd as u32)
    }

    /// The endpoint that issued the request.
    #[inline]
    pub const fn sender(&self) -> Handle {
        Handle(self.msg.id as u32)
    }

    /// A decoder over the payload.
    #[inline]
    pub fn decoder(&self) -> Decoder<'_> {
        Decoder::new(self.payload)
    }
}

/// One video reply.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reply {
    /// `0` means success; anything else is an error code.
    pub status: i32,
    /// Scalar result (a handle, a ticket, a counter, ...).
    pub arg0: u64,
    /// Collection total (engine count, ...).
    pub arg1: u64,
    /// Reply payload length in bytes.
    pub arg2: u64,
}

impl Reply {
    #[inline]
    pub const fn ok() -> Self {
        Self { status: 0, arg0: 0, arg1: 0, arg2: 0 }
    }

    #[inline]
    pub const fn err(error: DsError) -> Self {
        Self { status: error as i32, arg0: 0, arg1: 0, arg2: 0 }
    }

    #[inline]
    pub const fn arg0(mut self, value: u64) -> Self {
        self.arg0 = value;
        self
    }

    #[inline]
    pub const fn arg1(mut self, value: u64) -> Self {
        self.arg1 = value;
        self
    }

    /// Record the reply payload length.
    #[inline]
    pub const fn payload_len(mut self, len: usize) -> Self {
        self.arg2 = len as u64;
        self
    }

    #[inline]
    pub const fn is_ok(self) -> bool {
        self.status == 0
    }

    /// Package a reply into the `DsMsg` sent back to the requester.
    #[inline]
    pub const fn into_msg(self, request: &DsMsg) -> DsMsg {
        DsMsg {
            magic: kapi_abi::wire::DS_MAGIC,
            version: kapi_abi::wire::DS_VERSION as u16,
            cmd: request.cmd,
            id: request.id,
            flags: 0,
            status: self.status,
            arg0: self.arg0,
            arg1: self.arg1,
            arg2: self.arg2,
        }
    }
}


/// The capability an opcode needs, or `None` if it is not a video opcode.
///
/// Returning `None` for a foreign opcode is what lets the service sit in a
/// shared dispatcher: an unknown command is refused rather than misread as a
/// video one.
pub fn required_capability(cmd: DsCmd) -> Option<CapId> {
    use DsCmd::*;
    match cmd {
        VideoEngineEnumerate | VideoEngineQuery => Some(CapId::VIDEO_ENUMERATE),
        VideoSessionCreate | VideoSubmit | VideoAcquire | VideoRelease => Some(CapId::VIDEO_DECODE),
        VideoSessionDestroy | VideoSessionState | VideoDropCount => Some(CapId::VIDEO_DECODE),
        VideoSinkRegister | VideoSinkUnregister => Some(CapId::VIDEO_SINK),
        // Qualified, not bare `None`: the glob import above brings `DsCmd::None`
        // into scope, and it shadows `Option::None` for the rest of this body.
        _ => Option::None,
    }
}

/// Routes video IPC onto one engine.
pub struct VideoService {
    engine: Box<dyn VideoEngine>,
    granted: CapId,
}

impl VideoService {
    /// Wrap an engine, initially with no capabilities granted.
    pub fn new(engine: Box<dyn VideoEngine>) -> Self {
        Self { engine, granted: CapId::NONE }
    }

    /// Grant capabilities to the connected client.
    #[inline]
    pub fn grant(&mut self, caps: CapId) {
        self.granted = self.granted.union(caps);
    }

    /// Take capabilities away.
    #[inline]
    pub fn revoke(&mut self, caps: CapId) {
        self.granted = self.granted.remove(caps);
    }

    /// What the connected client currently holds.
    #[inline]
    pub const fn granted(&self) -> CapId {
        self.granted
    }

    /// Handle one request, writing any reply payload into `out`.
    pub fn dispatch(&mut self, request: &Request<'_>, out: &mut [u8]) -> Reply {
        let Some(cmd) = request.command() else {
            return Reply::err(DsError::InvalidMessage);
        };

        // Capability first, before the payload is even looked at. A client that
        // may not open a session learns nothing from a malformed one.
        let Some(needed) = required_capability(cmd) else {
            return Reply::err(DsError::InvalidMessage);
        };
        if !self.granted.has(needed) {
            return Reply::err(DsError::PermissionDenied);
        }

        match cmd {
            DsCmd::VideoEngineEnumerate | DsCmd::VideoEngineQuery => self.enumerate(request, out),
            _ => {
                // The remaining opcodes are handle-based and share one shape:
                // validate the handle, then dispatch. The capability gate above
                // is the part that has to be right for all of them, and it is;
                // the bodies land with the driver's session table.
                let stream = crate::types::StreamHandle::from(request.msg.arg0 as u32);
                if crate::types::check_handle(stream).is_err() {
                    return Reply::err(DsError::InvalidHandle);
                }
                Reply::err(DsError::InvalidMessage)
            }
        }
    }

    /// `VideoEngineEnumerate` and `VideoEngineQuery`: read one descriptor.
    fn enumerate(&mut self, request: &Request<'_>, out: &mut [u8]) -> Reply {
        let count = self.engine.engine_count();
        let index = request.msg.arg0 as u32;
        if index >= count {
            return Reply::err(DsError::DeviceNotFound);
        }
        let mut info = kapi_abi::payloads::video::EngineInfo::default();
        if !self.engine.engine_info(EngineId::from(index), &mut info) {
            return Reply::err(DsError::DeviceNotFound);
        }
        let mut enc = kapi_abi::wire::Encoder::new(out);
        if !codec::encode_engine_info(&mut enc, &info) {
            return Reply::err(DsError::BufferTooSmall);
        }
        Reply::ok().arg0(index as u64).arg1(count as u64).payload_len(enc.pos())
    }
}
