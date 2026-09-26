//! The GPU device-class contract.
//!
//! Drivers implement this trait; `GpuService` routes IPC requests onto it. The
//! framework contains no hardware knowledge, and the two drivers that
//! implement it - `AmdGpu-TrangorgeOS` and `IntelGpu-TrangorgeOS` - share
//! nothing of it, which is deliberate: see [`Vendor`].
//!
//! ## Why Intel and AMD are separate drivers
//!
//! They are not one driver with a switch. Their register maps, memory models
//! and command encodings have no common subset worth naming, and the traits
//! below are the *shape* they share - not a lowest common denominator of
//! behaviour. Anything genuinely specific to one vendor lives behind
//! [`GpuDevice::vendor_specific`] and stays there.
//!
//! ## What the contract does *not* cover
//!
//! It does not cover the command stream's contents. A client writes those
//! into memory it has mapped and names the range in `GpuSubmit`; the driver
//! decides what that means. Encoding a vendor command is the driver's job, and
//! duplicating it here would put a second, divergent copy in the framework.

use kapi_abi::{
    DsError,
    payloads::gpu::{
        GpuBuffer, GpuBufferRequest, GpuContextInfo, GpuContextRequest, GpuFence, GpuInfo, GpuMode,
        GpuShader, GpuShaderFlags, GpuSubmit,
    },
};

use crate::types::{BufferHandle, ContextHandle, FenceValue, GpuIndex, ShaderHandle};

/// Which vendor's hardware a driver drives.
///
/// Re-exported from `kapi-abi` so a driver names one type rather than two.
pub use kapi_abi::payloads::gpu::GpuVendor as Vendor;

/// A GPU that `ds-manager` can schedule.
///
/// The trait is object-safe: a service owns `Box<dyn GpuDevice>` and never
/// needs to know the driver's concrete type.
pub trait GpuDevice {
    // ── discovery ───────────────────────────────────────────────────────────────────────────────────

    /// Which vendor this driver speaks for.
    fn vendor(&self) -> Vendor;

    /// How many devices the driver found.
    fn device_count(&self) -> usize;

    /// Read the descriptor of device `index`.
    ///
    /// Returns `false` and leaves `out` untouched when out of range.
    fn device_info(&self, index: GpuIndex, out: &mut GpuInfo) -> bool;

    /// Read the current display mode of device `index`.
    fn get_mode(&self, index: GpuIndex, out: &mut GpuMode) -> Result<(), DsError>;

    /// Program a display mode on device `index`.
    ///
    /// Fails with `DeviceBusy` when work is in flight: changing the scanout
    /// target under a running command stream is how a display ends up showing a
    /// half-updated surface.
    fn set_mode(&mut self, index: GpuIndex, mode: &GpuMode) -> Result<(), DsError>;

    /// Bytes one scanline of `format` occupies at `width` pixels.
    ///
    /// Exposed as an opcode because a client that guesses wrong writes a
    /// buffer the scanout engine reads out of bounds - a value it can get from
    /// the driver instead.
    fn format_stride(&self, width: u32, format: u32) -> Result<u32, DsError>;

    // ── contexts ───────────────────────────────────────────────────────────────────────────────────

    /// Create a context on device `index`.
    fn create_context(
        &mut self,
        request: &GpuContextRequest,
        out: &mut GpuContextInfo,
    ) -> Result<(), DsError>;

    /// Destroy a context and everything that belonged to it.
    ///
    /// Must refuse while work is in flight unless `force`, so a half-submitted
    /// command stream is not left pointing at a destroyed ring.
    fn destroy_context(&mut self, context: ContextHandle, force: bool) -> Result<(), DsError>;

    // ── buffers ─────────────────────────────────────────────────────────────────────────────────────

    /// Allocate a buffer for use in a context.
    fn alloc_buffer(
        &mut self,
        context: ContextHandle,
        request: &GpuBufferRequest,
        out: &mut GpuBuffer,
    ) -> Result<(), DsError>;

    /// Free a buffer.
    fn free_buffer(&mut self, context: ContextHandle, buffer: BufferHandle) -> Result<(), DsError>;

    // ── shaders ────────────────────────────────────────────────────────────────────────────────────

    /// Compile a program for a context.
    ///
    /// The source arrives in `source`; the binary the driver produces stays
    /// driver-side and is referenced by the returned handle.
    fn compile_shader(
        &mut self,
        context: ContextHandle,
        flags: GpuShaderFlags,
        source: &[u8],
        out: &mut GpuShader,
    ) -> Result<(), DsError>;

    /// Free a compiled program.
    fn free_shader(
        &mut self,
        context: ContextHandle,
        shader: ShaderHandle,
    ) -> Result<(), DsError>;

    // ── submission ─────────────────────────────────────────────────────────────────────────────────

    /// Hand a command range to the GPU.
    ///
    /// Returns the fence the submission will signal. A submission is refused
    /// when the range is outside the ring the context was given, because the
    /// driver would otherwise be asked to read arbitrary shared memory.
    fn submit(&mut self, submit: &GpuSubmit) -> Result<FenceValue, DsError>;

    /// Read the current fence of a context.
    fn query_fence(&self, context: ContextHandle) -> Result<GpuFence, DsError>;

    /// Block until `fence` has been signalled, or report that it has not.
    ///
    /// Non-blocking on purpose: the framework has no scheduler to yield to, so
    /// a wait here would be a spin a client can only escape by polling.
    fn wait_fence(&self, context: ContextHandle, fence: FenceValue) -> Result<bool, DsError>;

    // ── recovery ──────────────────────────────────────────────────────────────────────────────────

    /// Reset the device, discarding in-flight work.
    ///
    /// Refuses while a context is live unless `force`, because a reset that
    /// throws away a live ring leaves every context pointing at nothing.
    fn reset(&mut self, index: GpuIndex, force: bool) -> Result<(), DsError>;

    // ── the vendor escape hatch ───────────────────────────────────────────────────────────────────

    /// A driver-specific operation, identified by a driver-chosen code.
    ///
    /// This is the sanctioned place for "only Intel has this": anything
    /// dispatched here stays inside the driver that claims the code, so the
    /// framework never grows a method only one vendor needs.
    fn vendor_specific(
        &mut self,
        index: GpuIndex,
        code: u32,
        arg: u64,
    ) -> Result<u64, DsError>;
}
