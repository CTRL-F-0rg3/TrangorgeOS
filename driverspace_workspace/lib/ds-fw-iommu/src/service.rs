//! IOMMU 服务侧分发器。
//!
//! [`IommuService`] 把一条 `DsMsg` 请求翻译成一次 [`IommuDevice`] 调用，并把
//! 结果编码成 [`Reply`]。能力检查集中在这里——驱动实现体里不需要、也不应该
//! 重复做权限判断。
//!
//! ## 载荷传递约定
//!
//! 传输层负责把请求载荷从 `msg.arg0`（长度 `msg.arg1`）拷进调用方提供的
//! `Request::payload` 切片；服务把回复载荷写进 `dispatch` 的 `out` 缓冲。
//! 这样框架内部不含裸指针，单元测试可以直接喂字节切片。

use kapi_abi::{
    CapId, DsCmd, DsError, DsMsg, Handle,
    payloads::iommu::{
        IommuControllerInfo, IommuFaultPayload, IommuKind, IommuReservedRegionPayload, IommuStage,
    },
    wire::{Decoder, Encoder},
};

use crate::{
    codec,
    traits::IommuDevice,
    types::{ControllerId, DomainId, RequesterId},
};

/// 一个 IOMMU 请求：消息头 + 已解包到本地的载荷字节。
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

    /// 消息头声明的命令。
    #[inline]
    pub fn command(&self) -> Option<DsCmd> {
        DsCmd::from_u32(self.msg.cmd as u32)
    }

    /// 发起请求的端点。
    #[inline]
    pub const fn sender(&self) -> Handle {
        Handle(self.msg.id as u32)
    }
}

/// 一个 IOMMU 回复。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reply {
    /// `0` 表示成功，否则是错误码。
    pub status: i32,
    /// 标量结果（域 id、物理基址、作用域……）。
    pub arg0: u64,
    /// 集合总数（控制器数、保留区数、故障数……）。
    pub arg1: u64,
    /// 回复载荷字节数。
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

    /// 记录回复载荷长度。
    #[inline]
    pub const fn payload_len(mut self, len: usize) -> Self {
        self.arg2 = len as u64;
        self
    }

    #[inline]
    pub const fn is_ok(self) -> bool {
        self.status == 0
    }

    /// 把回复包成一条回送给请求方的 `DsMsg`（回显 `id` 与 `cmd`）。
    pub fn into_message(self, request: &DsMsg) -> DsMsg {
        let mut msg = *request;
        msg.status = self.status;
        msg.arg0 = self.arg0;
        msg.arg1 = self.arg1;
        msg.arg2 = self.arg2;
        msg
    }
}

/// 把一个 `DsCmd` 映射到它所需的能力位。
///
/// 读类操作只需要 `IOMMU_ENUMERATE`；改动硬件状态的操作各自需要对应能力。
/// `None` 表示该 opcode 不属于 IOMMU 设备类。
pub fn required_capability(cmd: DsCmd) -> Option<CapId> {
    let cap = match cmd {
        DsCmd::IommuEnumerate
        | DsCmd::IommuQueryController
        | DsCmd::IommuReservedRegions
        | DsCmd::IommuFaultRead => CapId::IOMMU_ENUMERATE,
        DsCmd::IommuDomainCreate | DsCmd::IommuDomainDestroy => CapId::IOMMU_DOMAIN,
        DsCmd::IommuBind | DsCmd::IommuUnbind => CapId::IOMMU_BIND,
        DsCmd::IommuMap | DsCmd::IommuUnmap | DsCmd::IommuInvalidate => CapId::IOMMU_MAP,
        _ => return None,
    };
    Some(cap)
}

/// IOMMU 服务：持有一个设备实现，对外提供 IPC 分发。
pub struct IommuService<D> {
    device: D,
    capabilities: CapId,
}

impl<D: IommuDevice> IommuService<D> {
    /// 以“尚未授予任何能力”创建服务。
    #[inline]
    pub const fn new(device: D) -> Self {
        Self { device, capabilities: CapId::NONE }
    }

    /// 授予调用方一组能力。通常由 `ds-manager` 在设备注册时设置。
    #[inline]
    pub fn grant(&mut self, capabilities: CapId) {
        self.capabilities = self.capabilities.union(capabilities);
    }

    /// 撤销一组能力。
    #[inline]
    pub fn revoke(&mut self, capabilities: CapId) {
        self.capabilities = self.capabilities.remove(capabilities);
    }

    #[inline]
    pub const fn capabilities(&self) -> CapId {
        self.capabilities
    }

    #[inline]
    pub const fn device(&self) -> &D {
        &self.device
    }

    #[inline]
    pub const fn device_mut(&mut self) -> &mut D {
        &mut self.device
    }
}

impl<D: IommuDevice> IommuService<D> {
    /// 处理一条请求。
    ///
    /// 回复载荷（若非空）被写入 `out` 的开头，长度记录在 [`Reply::arg2`]。
    /// `out` 太小会被报告为 `DsError::BufferTooSmall`，因此调用方总能通过
    /// 状态码判断是否写入成功。
    pub fn dispatch(&mut self, request: &Request<'_>, out: &mut [u8]) -> Reply {
        let Some(cmd) = request.command() else {
            return Reply::err(DsError::InvalidMessage);
        };
        let Some(required) = required_capability(cmd) else {
            return Reply::err(DsError::InvalidMessage);
        };
        if !self.capabilities.has(required) {
            return Reply::err(DsError::PermissionDenied);
        }

        match cmd {
            DsCmd::IommuEnumerate => Reply::ok().arg0(self.device.controller_count() as u64),
            DsCmd::IommuQueryController => self.query_controller(request, out),
            DsCmd::IommuDomainCreate => {
                let controller = ControllerId(request.msg.arg0 as u32);
                match self.device.domain_create(controller, request.msg.arg1 as u32) {
                    Ok(domain) => Reply::ok().arg0(domain.raw() as u64),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::IommuDomainDestroy => {
                let domain = DomainId(request.msg.arg0 as u32);
                match self.device.domain_destroy(domain) {
                    Ok(()) => Reply::ok(),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::IommuBind => {
                let Some(payload) = decode(request.payload, codec::decode_bind) else {
                    return Reply::err(DsError::InvalidMessage);
                };
                match self.device.bind(&payload) {
                    Ok(()) => Reply::ok(),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::IommuUnbind => {
                let controller = ControllerId(request.msg.arg0 as u32);
                let requester = RequesterId(request.msg.arg1 as u32);
                match self.device.unbind(controller, requester) {
                    Ok(()) => Reply::ok(),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::IommuMap => {
                let Some(payload) = decode(request.payload, codec::decode_map) else {
                    return Reply::err(DsError::InvalidMessage);
                };
                match self.device.map(&payload) {
                    Ok(phys_base) => Reply::ok().arg0(phys_base),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::IommuUnmap => {
                let Some(payload) = decode(request.payload, codec::decode_unmap) else {
                    return Reply::err(DsError::InvalidMessage);
                };
                match self.device.unmap(&payload) {
                    Ok(()) => Reply::ok(),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::IommuInvalidate => {
                let Some(payload) = decode(request.payload, codec::decode_invalidate) else {
                    return Reply::err(DsError::InvalidMessage);
                };
                match self.device.invalidate(&payload) {
                    Ok(scope) => Reply::ok().arg0(scope as u64),
                    Err(error) => Reply::err(error),
                }
            }
            DsCmd::IommuReservedRegions => self.reserved_region(request, out),
            DsCmd::IommuFaultRead => self.fault_read(request, out),
            _ => Reply::err(DsError::InvalidMessage),
        }
    }

    fn query_controller(&mut self, request: &Request<'_>, out: &mut [u8]) -> Reply {
        let index = request.msg.arg0 as usize;
        let mut info = IommuControllerInfo {
            controller: 0,
            kind: IommuKind::Unknown,
            capabilities: 0,
            stage: IommuStage::Stage1,
            segment: 0,
            mmio_base: 0,
            mmio_size: 0,
        };
        if !self.device.controller_info(index, &mut info) {
            return Reply::err(DsError::DeviceNotFound);
        }
        encode(out, |enc| codec::encode_controller_info(enc, &info))
    }

    fn reserved_region(&mut self, request: &Request<'_>, out: &mut [u8]) -> Reply {
        let total = self.device.reserved_region_count();
        let index = request.msg.arg0 as usize;
        // 索引越界不是错误：调用方靠 arg0 的总数自行收敛。
        if index >= total {
            return Reply::ok().arg0(total as u64);
        }
        let mut region = IommuReservedRegionPayload {
            base: 0,
            limit: 0,
            requester: u32::MAX,
            _pad: 0,
        };
        if !self.device.reserved_region(index, &mut region) {
            return Reply::err(DsError::DeviceNotFound);
        }
        encode(out, |enc| codec::encode_reserved_region(enc, &region)).arg0(total as u64)
    }

    fn fault_read(&mut self, request: &Request<'_>, out: &mut [u8]) -> Reply {
        let total = self.device.pending_faults();
        let index = request.msg.arg0 as usize;
        if index >= total {
            return Reply::ok().arg0(total as u64);
        }
        let mut fault = IommuFaultPayload {
            controller: 0,
            reason: 0,
            requester: u32::MAX,
            _pad: 0,
            iova: 0,
            faulting_phys: 0,
        };
        if !self.device.read_fault(index, &mut fault) {
            return Reply::err(DsError::DeviceNotFound);
        }
        encode(out, |enc| codec::encode_fault(enc, &fault)).arg0(total as u64)
    }
}

/// 解码请求载荷；格式错误时返回 `None`，由调用方转成
/// `DsError::InvalidMessage`。
fn decode<T>(payload: &[u8], read: fn(&mut Decoder<'_>) -> Option<T>) -> Option<T> {
    let mut decoder = Decoder::new(payload);
    read(&mut decoder)
}

/// 把载荷写进 `out`，把写入长度记进 `arg2`。
fn encode(out: &mut [u8], write: impl FnOnce(&mut Encoder<'_>) -> bool) -> Reply {
    if out.is_empty() {
        return Reply::err(DsError::BufferTooSmall);
    }
    let mut encoder = Encoder::new(out);
    if write(&mut encoder) {
        Reply::ok().payload_len(encoder.pos())
    } else {
        Reply::err(DsError::BufferTooSmall)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kapi_abi::{
        payloads::iommu::{
            IommuBindPayload, IommuInvalidateScope, IommuMapFlags, IommuMapPayload,
            IommuPermission, IommuUnmapPayload,
        },
    };

    /// 一个完全在内存里的假设备，用来验证分发器而不碰硬件。
    struct FakeDevice {
        controllers: usize,
        domains: u32,
        last_bind: Option<RequesterId>,
        /// 已建立映射的条数（测试用计数器；真实驱动持有页表）。
        mapped: u32,
    }

    impl FakeDevice {
        fn new(controllers: usize) -> Self {
            Self { controllers, domains: 0, last_bind: None, mapped: 0 }
        }
    }

    impl IommuDevice for FakeDevice {
        fn controller_count(&self) -> usize {
            self.controllers
        }

        fn controller_info(&self, index: usize, out: &mut IommuControllerInfo) -> bool {
            if index >= self.controllers {
                return false;
            }
            *out = IommuControllerInfo {
                controller: index as u32,
                kind: IommuKind::IntelVtd,
                capabilities: 0x1f,
                stage: IommuStage::Stage1,
                segment: 0,
                mmio_base: 0xfed0_0000,
                mmio_size: 0x1000,
            };
            true
        }

        fn domain_create(
            &mut self,
            _controller: ControllerId,
            _hint: u32,
        ) -> Result<DomainId, DsError> {
            self.domains += 1;
            Ok(DomainId(self.domains))
        }

        fn domain_destroy(&mut self, domain: DomainId) -> Result<(), DsError> {
            if !domain.is_valid() || domain.raw() > self.domains {
                return Err(DsError::InvalidMessage);
            }
            Ok(())
        }

        fn bind(&mut self, request: &IommuBindPayload) -> Result<(), DsError> {
            self.last_bind = Some(RequesterId(request.requester));
            Ok(())
        }

        fn unbind(
            &mut self,
            _controller: ControllerId,
            requester: RequesterId,
        ) -> Result<(), DsError> {
            if self.last_bind == Some(requester) {
                self.last_bind = None;
            }
            Ok(())
        }

        fn map(&mut self, request: &IommuMapPayload) -> Result<u64, DsError> {
            self.mapped += 1;
            Ok(request.phys_base)
        }

        fn unmap(&mut self, request: &IommuUnmapPayload) -> Result<(), DsError> {
            if request.size != 0 {
                self.mapped = self.mapped.saturating_sub(1);
            }
            Ok(())
        }

        fn invalidate(
            &mut self,
            _request: &kapi_abi::payloads::iommu::IommuInvalidatePayload,
        ) -> Result<IommuInvalidateScope, DsError> {
            Ok(IommuInvalidateScope::Global)
        }

        fn reserved_region_count(&self) -> usize {
            0
        }

        fn reserved_region(&self, _index: usize, _out: &mut IommuReservedRegionPayload) -> bool {
            false
        }

        fn pending_faults(&self) -> usize {
            0
        }

        fn read_fault(&mut self, _index: usize, _out: &mut IommuFaultPayload) -> bool {
            false
        }
    }

    fn request(cmd: DsCmd) -> DsMsg {
        let mut msg = DsMsg::new(cmd as u32);
        msg.id = 7; // 发起者 handle
        msg
    }

    #[test]
    fn capabilities_gate_every_mutation() {
        let mut service = IommuService::new(FakeDevice::new(1));
        let mut out = [0u8; crate::MAX_PAYLOAD_LEN];

        // 未授权 -> 拒绝
        let reply =
            service.dispatch(&Request::new(request(DsCmd::IommuDomainCreate), &[]), &mut out);
        assert_eq!(reply.status, DsError::PermissionDenied as i32);

        // 授权后 -> 成功，并分配出新域
        service.grant(CapId::IOMMU_DOMAIN);
        let reply =
            service.dispatch(&Request::new(request(DsCmd::IommuDomainCreate), &[]), &mut out);
        assert!(reply.is_ok());
        assert_eq!(reply.arg0, 1);
    }

    #[test]
    fn read_only_opcodes_need_only_enumerate_capability() {
        let mut service = IommuService::new(FakeDevice::new(2));
        service.grant(CapId::IOMMU_ENUMERATE);
        let mut out = [0u8; crate::MAX_PAYLOAD_LEN];

        let reply = service.dispatch(&Request::new(request(DsCmd::IommuEnumerate), &[]), &mut out);
        assert!(reply.is_ok());
        assert_eq!(reply.arg0, 2);

        let mut msg = request(DsCmd::IommuQueryController);
        msg.arg0 = 1;
        let reply = service.dispatch(&Request::new(msg, &[]), &mut out);
        assert!(reply.is_ok());
        assert!(reply.arg2 > 0);

        let mut decoder = Decoder::new(&out);
        let info = codec::decode_controller_info(&mut decoder).expect("decodes");
        assert_eq!(info.controller, 1);
        assert_eq!(info.mmio_base, 0xfed0_0000);

        // 越界
        let mut msg = request(DsCmd::IommuQueryController);
        msg.arg0 = 9;
        let reply = service.dispatch(&Request::new(msg, &[]), &mut out);
        assert_eq!(reply.status, DsError::DeviceNotFound as i32);
    }

    #[test]
    fn map_requires_payload_and_map_capability() {
        let mut service = IommuService::new(FakeDevice::new(1));
        let mut out = [0u8; crate::MAX_PAYLOAD_LEN];

        let payload = IommuMapPayload {
            domain: 1,
            flags: IommuMapFlags::FIXED,
            permission: IommuPermission::RW,
            _pad: 0,
            iova: 0x4000,
            phys_base: 0x8000_0000,
            size: 0x1000,
        };
        let mut encoded = [0u8; crate::MAX_PAYLOAD_LEN];
        let len = {
            let mut encoder = Encoder::new(&mut encoded);
            assert!(codec::encode_map(&mut encoder, &payload));
            encoder.pos()
        };
        let encoded = &encoded[..len];

        // 载荷齐全但没能力
        let reply = service.dispatch(&Request::new(request(DsCmd::IommuMap), encoded), &mut out);
        assert_eq!(reply.status, DsError::PermissionDenied as i32);

        service.grant(CapId::IOMMU_MAP);
        let reply = service.dispatch(&Request::new(request(DsCmd::IommuMap), encoded), &mut out);
        assert!(reply.is_ok());
        assert_eq!(reply.arg0, 0x8000_0000);

        // 载荷被截断
        let reply =
            service.dispatch(&Request::new(request(DsCmd::IommuMap), &encoded[..3]), &mut out);
        assert_eq!(reply.status, DsError::InvalidMessage as i32);
    }

    #[test]
    fn bind_round_trips_through_the_wire() {
        let mut service = IommuService::new(FakeDevice::new(1));
        service.grant(CapId::IOMMU_BIND.union(CapId::IOMMU_ENUMERATE));
        let mut out = [0u8; crate::MAX_PAYLOAD_LEN];

        let requester = RequesterId::new(0, 0x2a, 5, 3);
        let payload = IommuBindPayload {
            controller: 0,
            requester: requester.raw(),
            domain: 1,
            selector: 0,
        };
        let mut encoded = [0u8; crate::MAX_PAYLOAD_LEN];
        let len = {
            let mut encoder = Encoder::new(&mut encoded);
            assert!(codec::encode_bind(&mut encoder, &payload));
            encoder.pos()
        };
        let encoded = &encoded[..len];

        let msg = request(DsCmd::IommuBind);
        let reply = service.dispatch(&Request::new(msg, encoded), &mut out);
        assert!(reply.is_ok());
        assert_eq!(service.device().last_bind, Some(requester));

        // 解绑走寄存器约定（controller 在 arg0、requester 在 arg1）
        let mut msg = request(DsCmd::IommuUnbind);
        msg.arg0 = 0;
        msg.arg1 = requester.raw() as u64;
        let reply = service.dispatch(&Request::new(msg, &[]), &mut out);
        assert!(reply.is_ok());
        assert_eq!(service.device().last_bind, None);
    }

    #[test]
    fn non_iommu_opcode_is_rejected() {
        let mut service = IommuService::new(FakeDevice::new(1));
        service.grant(CapId::KERNEL_ALL);
        let mut out = [0u8; crate::MAX_PAYLOAD_LEN];
        let reply = service.dispatch(&Request::new(request(DsCmd::BlkRead), &[]), &mut out);
        assert_eq!(reply.status, DsError::InvalidMessage as i32);
    }

    #[test]
    fn tiny_reply_buffer_reports_buffer_too_small() {
        let mut service = IommuService::new(FakeDevice::new(1));
        service.grant(CapId::IOMMU_ENUMERATE);
        let mut out = [0u8; 2];
        let reply =
            service.dispatch(&Request::new(request(DsCmd::IommuQueryController), &[]), &mut out);
        assert_eq!(reply.status, DsError::BufferTooSmall as i32);
    }

    #[test]
    fn reply_message_echoes_request_identity() {
        let req = request(DsCmd::IommuEnumerate);
        let msg = Reply::ok().arg0(4).into_message(&req);
        assert_eq!(msg.id, req.id);
        assert_eq!(msg.cmd, req.cmd);
        assert_eq!(msg.status, 0);
        assert_eq!(msg.arg0, 4);
    }
}
