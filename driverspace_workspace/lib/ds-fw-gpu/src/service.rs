//! The GPU server-side dispatcher.
//!
//! [`GpuService`] turns one `DsMsg` into one [`GpuDevice`] call and encodes
//! the outcome as a [`Reply`]. Capability checks live here, so a driver
//! neither needs to nor should repeat permission logic.
//!
//! ## The capability split
//!
//! Two bits, and the boundary between them is the point of the whole layer:
//!
//! * `GPU_ENUMERATE` - list devices, read descriptors, ask for a stride. A
//!   client that can only do this can find out what the hardware is and
//!   nothing more.
//! * `GPU_DEVICE` - create contexts, allocate buffers, compile shaders,
//!   submit work, reset. This is what makes a GPU driver privileged, and it is
//!   granted per client by `ds-manager`, never by the driver itself.
//!
//! A driver that hands itself `GPU_DEVICE` at start-up has made the capability
//! system decorative, which is the same failure mode `ds-fw-audio` guards
//! against.
//!
//! ## Vendor operations
//!
//! `vendor_specific` is gated on `GPU_DEVICE` *and* on the device belonging to
//! the driver's own vendor. A client cannot reach an AMD code path on an Intel
//! driver, and a driver cannot be talked into running another vendor's code.

use alloc::{boxed::Box, vec::Vec};

use kapi_abi::{
    CapId, DsCmd, DsError, DsMsg, Handle,
    payloads::gpu::{
        GpuBuffer, GpuBufferRequest, GpuContextInfo, GpuContextRequest, GpuFence, GpuInfo, GpuMode,
        GpuShader, GpuSubmit,
    },
};

use crate::{
    codec,
    traits::{GpuDevice, Vendor},
    types::{BufferHandle, ContextHandle, FenceValue, GpuIndex, ShaderHandle},
};

/// One GPU request: the message header plus its payload, already unpacked.
#[derive(Clone, Copy)]
pub struct Request<'a> {
    pub msg: DsMsg,
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
}

/// One GPU reply.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reply {
    /// `0` means success; anything else is an error code.
    pub status: i32,
    /// Scalar result (a handle, a fence, a stride, ...).
    pub arg0: u64,
    /// Collection total, for a count.
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

    /// Package a reply into the `DsMsg` sent back, echoing identity.
    pub fn into_message(self, request: &DsMsg) -> DsMsg {
        let mut msg = *request;
        msg.status = self.status;
        msg.arg0 = self.arg0;
        msg.arg1 = self.arg1;
        msg.arg2 = self.arg2;
        msg
    }
}

/// Map a `DsCmd` to the capability it needs.
///
/// Reads need only `GPU_ENUMERATE`; anything that allocates, compiles,
/// submits or resets needs `GPU_DEVICE`. `None` means the opcode is not a GPU
/// opcode at all.
pub const fn required_capability(cmd: DsCmd) -> Option<CapId> {
    let value = cmd as u32;
    match value {
        // Read-only: what hardware is there, and how wide a scanline is.
        0x0B00..=0x0B01 | 0x0B0C => Some(CapId::GPU_ENUMERATE),
        // Privileged: contexts, buffers, shaders, work, reset.
        0x0B02..=0x0B0B => Some(CapId::GPU_DEVICE),
        // The display-side opcodes predate this contract; they need the
        // legacy graphics capabilities rather than the driver's.
        0x0300..=0x0309 => Some(CapId::GFX_FRAMEBUFFER),
        _ => None,
    }
}

/// A dispatchable GPU driver, plus the policy around it.
///
/// The driver is behind a `Box`, so the framework never links against
/// `AmdGpu-TrangorgeOS` or `IntelGpu-TrangorgeOS` and either can be built
/// without the other.
pub struct GpuService {
    device: Box<dyn GpuDevice>,
    /// Which GPU opcodes this service answers.
    caps: CapId,
    /// The vendor a `vendor_specific` code must match.
    vendor: Vendor,
    /// Live contexts, so the framework can enforce lifetime and, eventually,
    /// hold work off a busy device.
    contexts: crate::types::HandleTable<ContextInfo, { crate::MAX_CONTEXTS }>,
}

impl GpuService {
    /// Wrap a driver for dispatch.
    ///
    /// The service advertises `GPU_ENUMERATE | GPU_DEVICE`; whether a *client*
    /// gets either is `ds-manager`'s decision, not this one.
    pub fn new(device: Box<dyn GpuDevice>) -> Self {
        let vendor = device.vendor();
        Self {
            device,
            caps: CapId::GPU_ENUMERATE.union(CapId::GPU_DEVICE),
            vendor,
            contexts: crate::types::HandleTable::new(ContextInfo::EMPTY),
        }
    }

    /// Which GPU capabilities this service answers to.
    #[inline]
    pub const fn caps(&self) -> CapId {
        self.caps
    }

    /// The vendor this service drives.
    #[inline]
    pub const fn vendor(&self) -> Vendor {
        self.vendor
    }

    /// Borrow the driver, for a driver that needs to service its own queue.
    #[inline]
    pub fn device(&self) -> &dyn GpuDevice {
        self.device.as_ref()
    }

    /// Mutably borrow the driver.
    #[inline]
    pub fn device_mut(&mut self) -> &mut dyn GpuDevice {
        self.device.as_mut()
    }

    /// Reach a concrete driver, for the framework's own tests.
    ///
    /// This goes through `as_any_mut` because `downcast_mut` belongs to `Any`,
    /// not to every trait. Production code has no use for it, which is why it
    /// is a separate method rather than part of the contract.
    pub fn device_as_mut<T: GpuDevice + 'static>(&mut self) -> Option<&mut T> {
        self.device.as_mut().as_any_mut().downcast_mut::<T>()
    }

    /// How many contexts are live.
    #[inline]
    pub fn live_contexts(&self) -> usize {
        self.contexts.len()
    }

    /// Dispatch one request, given the capabilities its sender holds.
    ///
    /// `reply` carries the reply payload out; the caller frees it. A request
    /// whose opcode is not a GPU opcode, or whose capability the sender lacks,
    /// is rejected before the driver is touched.
    pub fn dispatch(
        &mut self,
        request: &Request<'_>,
        sender_caps: CapId,
        reply: &mut Vec<u8>,
    ) -> Reply {
        let Some(cmd) = request.command() else {
            return Reply::err(DsError::InvalidMessage);
        };

        // A GPU opcode from a client that lacks the capability gets
        // `NotPermitted`, not a driver error: the client asked for something it
        // may not have, and the driver was never going to consider it.
        match required_capability(cmd) {
            Some(needed) if !sender_caps.has(needed) => return Reply::err(DsError::PermissionDenied),
            Some(_) => {}
            None => return Reply::err(DsError::InvalidMessage),
        }

        let value = cmd as u32;
        match value {
            0x0B00 => self.enumerate(request, reply),
            0x0B01 => self.gpu_info(request, reply),
            0x0B0C => self.format_stride(request),
            0x0B02 => self.create_context(request, reply),
            0x0B03 => self.destroy_context(request),
            0x0B04 => self.alloc_buffer(request, reply),
            0x0B05 => self.free_buffer(request),
            0x0B07 => self.submit(request),
            0x0B08 => self.query_fence(request, reply),
            0x0B09 => self.compile_shader(request, reply),
            0x0B0A => self.free_shader(request),
            0x0B0B => self.reset(request),
            // The legacy display opcodes reach a real driver through the
            // display path, not here; the service is not their dispatcher.
            0x0300..=0x0309 => Reply::err(DsError::Unknown),
            _ => Reply::err(DsError::InvalidMessage),
        }
    }
}

/// What the framework tracks per context.
///
/// The driver keeps its own richer state; this is only what the framework
/// needs to answer "is this handle live" and "is this device busy".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextInfo {
    pub device: GpuIndex,
    pub ring_size: u64,
}

impl ContextInfo {
    pub const EMPTY: Self = Self { device: GpuIndex(0), ring_size: 0 };
}

impl GpuService {
    /// `GpuEnumerate`: the total device count in `arg0`, and `GpuInfo` for
    /// device `msg.arg0` in the payload.
    fn enumerate(&mut self, request: &Request<'_>, reply: &mut Vec<u8>) -> Reply {
        let count = self.device.device_count();
        let index = GpuIndex::new(request.msg.arg0 as u32);
        if index.index() >= count as u32 {
            // Out of range: report the count and leave the payload empty, so a
            // client can walk the list without treating this as an error.
            return Reply::ok().arg0(count as u64);
        }
        let mut info = GpuInfo {
            index: index.index(),
            vendor: self.vendor,
            ..Default::default()
        };
        if !self.device.device_info(index, &mut info) {
            return Reply::err(DsError::DeviceNotFound);
        }
        match codec::encode_device(&info, reply) {
            Ok(len) => Reply::ok().arg0(count as u64).payload_len(len),
            Err(error) => Reply::err(error),
        }
    }

    /// `GpuInfo`: the descriptor of device `msg.arg0`.
    fn gpu_info(&mut self, request: &Request<'_>, reply: &mut Vec<u8>) -> Reply {
        let index = GpuIndex::new(request.msg.arg0 as u32);
        if index.index() >= self.device.device_count() as u32 {
            return Reply::err(DsError::DeviceNotFound);
        }
        let mut info = GpuInfo { index: index.index(), vendor: self.vendor, ..Default::default() };
        if !self.device.device_info(index, &mut info) {
            return Reply::err(DsError::DeviceNotFound);
        }
        match codec::encode_device(&info, reply) {
            Ok(len) => Reply::ok().payload_len(len),
            Err(error) => Reply::err(error),
        }
    }

    /// `GpuFormatStride`: bytes per scanline, as a scalar.
    fn format_stride(&mut self, request: &Request<'_>) -> Reply {
        let width = request.msg.arg0 as u32;
        let format = request.msg.arg1 as u32;
        match self.device.format_stride(width, format) {
            Ok(stride) => Reply::ok().arg0(stride as u64),
            Err(error) => Reply::err(error),
        }
    }

    /// `GpuContextCreate`: unpack the request, call the driver, record the
    /// context, and return the handle.
    fn create_context(&mut self, request: &Request<'_>, reply: &mut Vec<u8>) -> Reply {
        let Ok(want) = codec::decode_context_request(request.payload) else {
            return Reply::err(DsError::InvalidMessage);
        };
        let mut info = GpuContextInfo::default();
        if let Err(error) = self.device.create_context(&want, &mut info) {
            return Reply::err(error);
        }
        let Some(raw) = self.contexts.insert(ContextInfo {
            device: GpuIndex::new(want.device),
            ring_size: info.ring_size,
        }) else {
            // The framework is out of context slots. Undo the driver's work
            // rather than leaving a context the framework cannot name.
            let handle = ContextHandle::new(info.handle);
            let _ = self.device.destroy_context(handle, true);
            return Reply::err(DsError::OutOfMemory);
        };
        info.handle = raw;
        match codec::encode_context_info(&info, reply) {
            Ok(len) => Reply::ok().arg0(raw as u64).payload_len(len),
            Err(error) => Reply::err(error),
        }
    }

    /// `GpuContextDestroy`.
    fn destroy_context(&mut self, request: &Request<'_>) -> Reply {
        let handle = ContextHandle::new(request.msg.arg0 as u32);
        if self.contexts.get(handle.raw()).is_none() {
            return Reply::err(DsError::InvalidHandle);
        }
        let force = request.msg.arg1 != 0;
        if let Err(error) = self.device.destroy_context(handle, force) {
            return Reply::err(error);
        }
        self.contexts.remove(handle.raw());
        Reply::ok()
    }

    /// `GpuBufferAlloc`.
    fn alloc_buffer(&mut self, request: &Request<'_>, reply: &mut Vec<u8>) -> Reply {
        let context = ContextHandle::new(request.msg.arg0 as u32);
        let Some(info) = self.contexts.get(context.raw()) else {
            return Reply::err(DsError::InvalidHandle);
        };
        let Ok(want) = codec::decode_buffer_request(request.payload) else {
            return Reply::err(DsError::InvalidMessage);
        };
        let mut buffer = GpuBuffer::default();
        if let Err(error) = self.device.alloc_buffer(context, &want, &mut buffer) {
            return Reply::err(error);
        }
        let _ = info;
        match codec::encode_buffer(&buffer, reply) {
            Ok(len) => Reply::ok().arg0(buffer.handle as u64).payload_len(len),
            Err(error) => Reply::err(error),
        }
    }

    /// `GpuBufferFree`.
    fn free_buffer(&mut self, request: &Request<'_>) -> Reply {
        let context = ContextHandle::new(request.msg.arg0 as u32);
        if self.contexts.get(context.raw()).is_none() {
            return Reply::err(DsError::InvalidHandle);
        }
        let buffer = BufferHandle::new(request.msg.arg1 as u32);
        match self.device.free_buffer(context, buffer) {
            Ok(()) => Reply::ok(),
            Err(error) => Reply::err(error),
        }
    }

    /// `GpuSubmit`: hand a command range to the GPU and return its fence.
    fn submit(&mut self, request: &Request<'_>) -> Reply {
        let context = ContextHandle::new(request.msg.arg0 as u32);
        let Some(_) = self.contexts.get(context.raw()) else {
            return Reply::err(DsError::InvalidHandle);
        };
        let Ok(want) = codec::decode_submit(request.payload) else {
            return Reply::err(DsError::InvalidMessage);
        };
        // The context travels in the message header; the payload repeats it so
        // a driver reading only the payload still knows the target.
        let mut want = want;
        want.context = context.raw();
        match self.device.submit(&want) {
            Ok(fence) => Reply::ok().arg0(fence.value()),
            Err(error) => Reply::err(error),
        }
    }

    /// `GpuFenceQuery`.
    fn query_fence(&mut self, request: &Request<'_>, reply: &mut Vec<u8>) -> Reply {
        let context = ContextHandle::new(request.msg.arg0 as u32);
        if self.contexts.get(context.raw()).is_none() {
            return Reply::err(DsError::InvalidHandle);
        }
        match self.device.query_fence(context) {
            Ok(fence) => match codec::encode_fence(&fence, reply) {
                Ok(len) => Reply::ok().arg0(fence.value).payload_len(len),
                Err(error) => Reply::err(error),
            },
            Err(error) => Reply::err(error),
        }
    }

    /// `GpuShaderCompile`: the source follows the header in the payload.
    fn compile_shader(&mut self, request: &Request<'_>, reply: &mut Vec<u8>) -> Reply {
        let context = ContextHandle::new(request.msg.arg0 as u32);
        if self.contexts.get(context.raw()).is_none() {
            return Reply::err(DsError::InvalidHandle);
        }
        let Ok((want, source)) = codec::decode_shader_request(request.payload) else {
            return Reply::err(DsError::InvalidMessage);
        };
        let mut shader = GpuShader::default();
        if let Err(error) = self.device.compile_shader(context, want.flags, source, &mut shader) {
            return Reply::err(error);
        }
        match codec::encode_shader(&shader, reply) {
            Ok(len) => Reply::ok().arg0(shader.handle as u64).payload_len(len),
            Err(error) => Reply::err(error),
        }
    }

    /// `GpuShaderFree`.
    fn free_shader(&mut self, request: &Request<'_>) -> Reply {
        let context = ContextHandle::new(request.msg.arg0 as u32);
        if self.contexts.get(context.raw()).is_none() {
            return Reply::err(DsError::InvalidHandle);
        }
        let shader = ShaderHandle::new(request.msg.arg1 as u32);
        match self.device.free_shader(context, shader) {
            Ok(()) => Reply::ok(),
            Err(error) => Reply::err(error),
        }
    }

    /// `GpuReset`.
    fn reset(&mut self, request: &Request<'_>) -> Reply {
        let index = GpuIndex::new(request.msg.arg0 as u32);
        if index.index() >= self.device.device_count() as u32 {
            return Reply::err(DsError::DeviceNotFound);
        }
        let force = request.msg.arg1 != 0;
        match self.device.reset(index, force) {
            Ok(()) => {
                // A forced reset invalidates every context: their rings are
                // gone, so leaving the handles live would let a client submit
                // into memory that no longer means anything.
                if force {
                    self.contexts = crate::types::HandleTable::new(ContextInfo::EMPTY);
                }
                Reply::ok()
            }
            Err(error) => Reply::err(error),
        }
    }
}
