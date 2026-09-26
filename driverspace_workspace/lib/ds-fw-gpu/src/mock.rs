//! A mock GPU, for testing the framework without hardware.
//!
//! This is not a stub. It is a complete [`GpuDevice`] that records what it was
//! asked to do, so the framework's own rules can be tested directly:
//!
//! * a client without the capability is refused before the mock is touched;
//! * a stale handle is rejected rather than silently accepted;
//! * a forced reset invalidates the contexts it must invalidate;
//! * fences advance monotonically, which is what the client layer relies on.
//!
//! Both vendor drivers get their tests from this, so a test written once
//! covers the contract that `AmdGpu-TrangorgeOS` and `IntelGpu-TrangorgeOS`
//! both have to satisfy.

use alloc::{boxed::Box, vec::Vec};

use kapi_abi::{
    DsError,
    payloads::gpu::{
        GpuBuffer, GpuBufferRequest, GpuCaps, GpuContextInfo, GpuContextRequest, GpuFence,
        GpuInfo, GpuMode, GpuShader, GpuShaderFlags, GpuSubmit, GpuVendor,
    },
};

use crate::{
    traits::{GpuDevice, Vendor},
    types::{BufferHandle, ContextHandle, FenceValue, GpuIndex, ShaderHandle},
};

/// One recorded call, so a test can assert the framework routed correctly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    GetMode(u32),
    SetMode(u32, u32, u32),
    CreateContext(u32),
    DestroyContext(u32, bool),
    AllocBuffer(u32, u64),
    FreeBuffer(u32, u32),
    CompileShader(u32, u32),
    FreeShader(u32, u32),
    Submit(u32, u32, u32),
    Reset(u32, bool),
    VendorSpecific(u32, u32, u64),
}

/// A `GpuDevice` that records its calls and fabricates plausible results.
pub struct MockGpu {
    vendor: GpuVendor,
    devices: usize,
    /// Every call the framework made, in order.
    pub calls: Vec<Call>,
    /// The fence the next `submit` returns; advances by one each time.
    next_fence: u64,
    /// Make the next `submit` fail, to test error propagation.
    pub fail_submit: bool,
    /// Make `create_context` fail.
    pub fail_context: bool,
    /// Make `reset` fail.
    pub fail_reset: bool,
}

impl MockGpu {
    /// A mock for `vendor` reporting `devices` devices.
    pub fn new(vendor: GpuVendor, devices: usize) -> Self {
        Self {
            vendor,
            devices,
            calls: Vec::new(),
            next_fence: 1,
            fail_submit: false,
            fail_context: false,
            fail_reset: false,
        }
    }

    /// An AMD mock, for the AMD driver's tests.
    pub fn amd() -> Self {
        Self::new(GpuVendor::Amd, 1)
    }

    /// An Intel mock, for the Intel driver's tests.
    pub fn intel() -> Self {
        Self::new(GpuVendor::Intel, 1)
    }

    /// How many times `call` was made.
    pub fn count(&self, call: Call) -> usize {
        self.calls.iter().filter(|seen| **seen == call).count()
    }

    /// Whether `call` was ever made.
    pub fn saw(&self, call: Call) -> bool {
        self.calls.contains(&call)
    }

    /// The fence the next submission will return.
    pub fn next_fence(&self) -> u64 {
        self.next_fence
    }
}

impl GpuDevice for MockGpu {
    fn vendor(&self) -> GpuVendor {
        self.vendor
    }

    fn device_count(&self) -> usize {
        self.devices
    }

    fn device_info(&self, index: GpuIndex, out: &mut GpuInfo) -> bool {
        if index.index() as usize >= self.devices {
            return false;
        }
        *out = GpuInfo {
            index: index.index(),
            vendor: self.vendor,
            vendor_id: match self.vendor {
                GpuVendor::Intel => 0x8086,
                GpuVendor::Amd => 0x1002,
                _ => 0xFFFF,
            },
            device_id: 0x1234,
            caps: GpuCaps::RING_SUBMIT | GpuCaps::HW_FENCE,
            vram_size: 8 << 30,
            mmio_size: 16 << 20,
            clock_mhz: 1200,
            engines: 4,
        };
        true
    }

    fn get_mode(&self, index: GpuIndex, out: &mut GpuMode) -> Result<(), DsError> {
        if index.index() as usize >= self.devices {
            return Err(DsError::DeviceNotFound);
        }
        *out = GpuMode {
            width: 1920,
            height: 1080,
            format: kapi_abi::payloads::gpu::GpuFormat::XRGB8888,
            stride: 1920 * 4,
            refresh_mhz: 60_000,
        };
        Ok(())
    }

    fn set_mode(&mut self, index: GpuIndex, mode: &GpuMode) -> Result<(), DsError> {
        if index.index() as usize >= self.devices {
            return Err(DsError::DeviceNotFound);
        }
        self.calls.push(Call::SetMode(index.index(), mode.width, mode.height));
        Ok(())
    }

    fn format_stride(&self, width: u32, format: u32) -> Result<u32, DsError> {
        let bpp = kapi_abi::payloads::gpu::GpuFormat::from_u32(format).bytes_per_pixel();
        if bpp == 0 {
            return Err(DsError::Unknown);
        }
        Ok(width * bpp)
    }

    fn create_context(
        &mut self,
        request: &GpuContextRequest,
        out: &mut GpuContextInfo,
    ) -> Result<(), DsError> {
        if self.fail_context {
            return Err(DsError::OutOfMemory);
        }
        self.calls.push(Call::CreateContext(request.device));
        *out = GpuContextInfo {
            handle: 0,
            engines: request.engines.max(1),
            ring_size: 1 << 20,
            ring_tail_usable: (1 << 20) - 256,
        };
        Ok(())
    }

    fn destroy_context(&mut self, context: ContextHandle, force: bool) -> Result<(), DsError> {
        self.calls.push(Call::DestroyContext(context.raw(), force));
        Ok(())
    }

    fn alloc_buffer(
        &mut self,
        context: ContextHandle,
        request: &GpuBufferRequest,
        out: &mut GpuBuffer,
    ) -> Result<(), DsError> {
        if request.size == 0 {
            return Err(DsError::Unknown);
        }
        self.calls.push(Call::AllocBuffer(context.raw(), request.size));
        *out = GpuBuffer {
            handle: 1,
            _pad: 0,
            phys: 0x8000_0000,
            size: request.size,
            virt: 0xFFFF_8000_0000_0000,
        };
        Ok(())
    }

    fn free_buffer(&mut self, context: ContextHandle, buffer: BufferHandle) -> Result<(), DsError> {
        self.calls.push(Call::FreeBuffer(context.raw(), buffer.raw()));
        Ok(())
    }

    fn compile_shader(
        &mut self,
        context: ContextHandle,
        flags: GpuShaderFlags,
        source: &[u8],
        out: &mut GpuShader,
    ) -> Result<(), DsError> {
        // A driver refuses what it cannot compile rather than pretending, so
        // the mock does too: empty source or no stage is an error.
        if source.is_empty() || flags.is_empty() {
            return Err(DsError::Unknown);
        }
        self.calls.push(Call::CompileShader(context.raw(), source.len() as u32));
        *out = GpuShader { handle: 7, binary_len: 64, hash: 0xDEAD_BEEF };
        Ok(())
    }

    fn free_shader(
        &mut self,
        context: ContextHandle,
        shader: ShaderHandle,
    ) -> Result<(), DsError> {
        self.calls.push(Call::FreeShader(context.raw(), shader.raw()));
        Ok(())
    }

    fn submit(&mut self, submit: &GpuSubmit) -> Result<FenceValue, DsError> {
        if self.fail_submit {
            return Err(DsError::DeviceBusy);
        }
        self.calls
            .push(Call::Submit(submit.context, submit.offset, submit.length));
        let fence = self.next_fence;
        self.next_fence += 1;
        Ok(FenceValue::new(fence))
    }

    fn query_fence(&self, context: ContextHandle) -> Result<GpuFence, DsError> {
        Ok(GpuFence {
            value: self.next_fence.saturating_sub(1),
            context: context.raw(),
            signalled: 1,
            _pad: 0,
        })
    }

    fn wait_fence(&self, _context: ContextHandle, fence: FenceValue) -> Result<bool, DsError> {
        Ok(fence.value() < self.next_fence)
    }

    fn reset(&mut self, index: GpuIndex, force: bool) -> Result<(), DsError> {
        if self.fail_reset {
            return Err(DsError::DeviceBusy);
        }
        if index.index() as usize >= self.devices {
            return Err(DsError::DeviceNotFound);
        }
        self.calls.push(Call::Reset(index.index(), force));
        // A reset throws away in-flight work, so the fence counter restarts.
        self.next_fence = 1;
        Ok(())
    }

    fn vendor_specific(
        &mut self,
        index: GpuIndex,
        code: u32,
        arg: u64,
    ) -> Result<u64, DsError> {
        self.calls.push(Call::VendorSpecific(index.index(), code, arg));
        Ok(arg.wrapping_add(1))
    }
}
