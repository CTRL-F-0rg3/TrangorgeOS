#![no_std]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DsCmd {
    None = 0,

    SysLog = 0x0001,
    SysAlloc = 0x0002,
    SysFree = 0x0003,
    SysMapMmio = 0x0004,
    SysUnmapMmio = 0x0005,
    SysPagePhys = 0x0006,
    SysAllocDma = 0x0007,
    SysFreeDma = 0x0008,
    SysYield = 0x0009,
    SysInfo = 0x000A,
    SysShutdown = 0x000B,
    /// Fetch one ACPI SDT by its four-character signature.
    /// The request payload is `payloads::sys::AcpiTableRequest`; the reply
    /// carries the virtual base in `arg0` and the length in `arg1`.
    SysAcpiTable = 0x000C,

    IpcCreate = 0x0100,
    IpcDestroy = 0x0101,
    IpcSend = 0x0102,
    IpcRecv = 0x0103,
    IpcReply = 0x0104,
    IpcShareMem = 0x0105,
    IpcGrant = 0x0106,
    IpcRevoke = 0x0107,

    DevRegister = 0x0200,
    DevUnregister = 0x0201,
    DevAttach = 0x0202,
    DevDetach = 0x0203,
    DevSuspend = 0x0204,
    DevResume = 0x0205,
    DevIrqBind = 0x0206,
    DevIrqUnbind = 0x0207,
    DevMmioGrant = 0x0208,
    DevMmioRevoke = 0x0209,
    DevDmaGrant = 0x020A,
    DevDmaRevoke = 0x020B,

    PciFind = 0x0210,
    PciRead = 0x0211,
    PciWrite = 0x0212,

    GfxFbInfo = 0x0300,
    GfxFbTakeover = 0x0301,
    GfxFbRelease = 0x0302,
    GfxModeSet = 0x0303,
    GfxModeGet = 0x0304,
    GfxSurfaceCreate = 0x0305,
    GfxSurfaceDestroy = 0x0306,
    GfxSurfaceCommit = 0x0307,
    GfxCmdSubmit = 0x0308,
    GfxCmdWait = 0x0309,

    AudPlay = 0x0400,
    AudStop = 0x0401,
    AudCapture = 0x0402,
    AudJackQuery = 0x0403,
    AudAmpSet = 0x0404,
    AudStreamCreate = 0x0405,
    AudStreamDestroy = 0x0406,

    // ── ALSA / Alsa-TrangorgeOS (0x04xx) ─────────────────────────────
    // The opcodes above are the legacy whole-buffer API used by the old
    // `audiodriver`. The ones below are the ALSA-shaped vocabulary that
    // `Alsa-TrangorgeOS` speaks: a card owns several PCM endpoints, each
    // with its own handle, and hardware parameters are negotiated field by
    // field before a stream can be prepared.
    //
    // arg0 = PCM index; reply arg0 = total endpoint count, reply payload is
    // `PcmInfo`.
    AlsaPcmEnumerate = 0x0410,
    /// arg0 = index into the enumeration; reply payload is `PcmInfo`.
    AlsaPcmInfo = 0x0411,
    /// Payload is `PcmOpenPayload`; reply arg0 = the new PCM handle.
    AlsaPcmOpen = 0x0412,
    /// arg0 = PCM handle.
    AlsaPcmClose = 0x0413,
    /// arg0 = PCM handle; reply arg0 = the current `PcmState`.
    AlsaPcmState = 0x0414,
    /// Payload is `HwParamsQuery`; reply payload is `HwParamsResult`.
    AlsaPcmHwParams = 0x0415,
    /// arg0 = PCM handle; reply payload is the negotiated `PcmParams`.
    AlsaPcmParams = 0x0416,
    /// arg0 = PCM handle; the stream must be `Prepared`.
    AlsaPcmPrepare = 0x0417,
    /// arg0 = PCM handle; transitions `Prepared` -> `Running`.
    AlsaPcmStart = 0x0418,
    /// arg0 = PCM handle; arg1 = `DROP` or `DRAIN`.
    AlsaPcmDrop = 0x0419,
    /// arg0 = PCM handle; `Prepare` + `Start` in one step, as ALSA's
    /// `snd_pcm_prepare` / `snd_pcm_start` pair.
    AlsaPcmRecover = 0x041A,
    /// arg0 = PCM handle; arg1 = frame count. The sample payload is
    /// transferred through shared memory, not through the message registers,
    /// because a period is far larger than `DsMsg` is.
    AlsaPcmWrite = 0x041B,
    /// arg0 = PCM handle; arg1 = frame count.
    AlsaPcmRead = 0x041C,
    /// arg0 = index; reply arg0 = total, reply payload is `MixerSelem`.
    AlsaMixerEnumerate = 0x041D,
    /// arg0 = element index; reply payload is `MixerValue`.
    AlsaMixerRead = 0x041E,
    /// arg0 = element index; reply payload is `MixerValue` with the new
    /// value. The element must advertise `CtlAccess::WRITE`.
    AlsaMixerWrite = 0x041F,
    /// arg0 = jack index; reply payload is `JackState`.
    AlsaJackState = 0x0420,
    /// arg0 = index; reply arg0 = total, reply payload is `HwdepInfo`.
    AlsaHwdepEnumerate = 0x0421,

    BlkRead = 0x0500,
    BlkWrite = 0x0501,
    BlkFlush = 0x0502,
    BlkInfo = 0x0503,
    BlkTrim = 0x0504,

    InputPoll = 0x0600,
    InputSubscribe = 0x0601,
    InputUnsubscribe = 0x0602,

    NetSend = 0x0700,
    NetRecv = 0x0701,
    NetBind = 0x0702,
    NetUnbind = 0x0703,

    CapQuery = 0x0800,
    CapGrant = 0x0801,
    CapRevoke = 0x0802,
    CapDelegate = 0x0803,

    GfxViseCreateCtx = 0x0900,
    GfxViseDestroyCtx = 0x0901,
    GfxViseSwapBuffers = 0x0902,
    GfxViseUploadTex = 0x0903,
    GfxViseDrawCall = 0x0904,
    GfxViseSetPipeline = 0x0905,
    GfxViseSetUniform = 0x0906,

    // ── IOMMU (0x0Axx) ──────────────────────────────────────────────
    // Controller discovery: arg0 = controller index, reply arg0 = count.
    IommuEnumerate = 0x0A00,
    /// arg0 = controller index; reply payload is `IommuControllerInfo`.
    IommuQueryController = 0x0A01,
    /// arg0 = controller index; reply arg0 = the new domain id.
    IommuDomainCreate = 0x0A02,
    /// arg0 = domain id.
    IommuDomainDestroy = 0x0A03,
    /// Payload is `IommuBindPayload`.
    IommuBind = 0x0A04,
    /// arg0 = controller index, arg1 = packed requester id.
    IommuUnbind = 0x0A05,
    /// Payload is `IommuMapPayload`; without `FIXED` the reply arg0 is the
    /// allocated physical base.
    IommuMap = 0x0A06,
    /// Payload is `IommuUnmapPayload`.
    IommuUnmap = 0x0A07,
    /// Payload is `IommuInvalidatePayload`; reply arg0 is the `InvalidateScope`
    /// actually completed.
    IommuInvalidate = 0x0A08,
    /// arg0 = reservation index; reply arg0 = total, reply payload is
    /// `IommuReservedRegionPayload`.
    IommuReservedRegions = 0x0A09,
    /// arg0 = fault index; reply arg0 = pending total, reply payload is
    /// `IommuFaultPayload`.
    IommuFaultRead = 0x0A0A,
}

impl DsCmd {
    // --- Legacy aliases (old caller names -> canonical opcodes) ---
    // Allows existing ds-* / driver code to keep compiling.
    pub const Log: Self = Self::SysLog;
    pub const MemMapMmio: Self = Self::SysMapMmio;
    pub const MemUnmapMmio: Self = Self::SysUnmapMmio;
    pub const MemAllocDma: Self = Self::SysAllocDma;
    pub const MemFreeDma: Self = Self::SysFreeDma;
    pub const BindIrq: Self = Self::DevIrqBind;
    pub const PagePhys: Self = Self::SysPagePhys;
    pub const VideoFbInfo: Self = Self::GfxFbInfo;
    pub const ReqMmio: Self = Self::SysMapMmio;
    pub const ReqAcpiTable: Self = Self::SysAcpiTable;
    // IOMMU aliases used by the driver/manager vocabulary.
    pub const MapDma: Self = Self::IommuMap;
    pub const UnmapDma: Self = Self::IommuUnmap;
    pub const CreateIommuDomain: Self = Self::IommuDomainCreate;
    pub const DestroyIommuDomain: Self = Self::IommuDomainDestroy;
    pub const BindIommuRequester: Self = Self::IommuBind;

    #[inline]
    pub const fn category(self) -> u8 {
        ((self as u32) >> 8) as u8
    }

    #[inline]
    pub const fn from_u32(v: u32) -> Option<Self> {
        match v {
            0x0000 => Some(Self::None),
            0x0001 => Some(Self::SysLog),
            0x0002 => Some(Self::SysAlloc),
            0x0003 => Some(Self::SysFree),
            0x0004 => Some(Self::SysMapMmio),
            0x0005 => Some(Self::SysUnmapMmio),
            0x0006 => Some(Self::SysPagePhys),
            0x0007 => Some(Self::SysAllocDma),
            0x0008 => Some(Self::SysFreeDma),
            0x0009 => Some(Self::SysYield),
            0x000A => Some(Self::SysInfo),
            0x000B => Some(Self::SysShutdown),
            0x000C => Some(Self::SysAcpiTable),
            0x0100 => Some(Self::IpcCreate),
            0x0101 => Some(Self::IpcDestroy),
            0x0102 => Some(Self::IpcSend),
            0x0103 => Some(Self::IpcRecv),
            0x0104 => Some(Self::IpcReply),
            0x0105 => Some(Self::IpcShareMem),
            0x0106 => Some(Self::IpcGrant),
            0x0107 => Some(Self::IpcRevoke),
            0x0200 => Some(Self::DevRegister),
            0x0201 => Some(Self::DevUnregister),
            0x0202 => Some(Self::DevAttach),
            0x0203 => Some(Self::DevDetach),
            0x0204 => Some(Self::DevSuspend),
            0x0205 => Some(Self::DevResume),
            0x0206 => Some(Self::DevIrqBind),
            0x0207 => Some(Self::DevIrqUnbind),
            0x0208 => Some(Self::DevMmioGrant),
            0x0209 => Some(Self::DevMmioRevoke),
            0x020A => Some(Self::DevDmaGrant),
            0x020B => Some(Self::DevDmaRevoke),
            0x0210 => Some(Self::PciFind),
            0x0211 => Some(Self::PciRead),
            0x0212 => Some(Self::PciWrite),
            0x0300 => Some(Self::GfxFbInfo),
            0x0301 => Some(Self::GfxFbTakeover),
            0x0302 => Some(Self::GfxFbRelease),
            0x0303 => Some(Self::GfxModeSet),
            0x0304 => Some(Self::GfxModeGet),
            0x0305 => Some(Self::GfxSurfaceCreate),
            0x0306 => Some(Self::GfxSurfaceDestroy),
            0x0307 => Some(Self::GfxSurfaceCommit),
            0x0308 => Some(Self::GfxCmdSubmit),
            0x0309 => Some(Self::GfxCmdWait),
            0x0400 => Some(Self::AudPlay),
            0x0401 => Some(Self::AudStop),
            0x0402 => Some(Self::AudCapture),
            0x0403 => Some(Self::AudJackQuery),
            0x0404 => Some(Self::AudAmpSet),
            0x0405 => Some(Self::AudStreamCreate),
            0x0406 => Some(Self::AudStreamDestroy),
            0x0410 => Some(Self::AlsaPcmEnumerate),
            0x0411 => Some(Self::AlsaPcmInfo),
            0x0412 => Some(Self::AlsaPcmOpen),
            0x0413 => Some(Self::AlsaPcmClose),
            0x0414 => Some(Self::AlsaPcmState),
            0x0415 => Some(Self::AlsaPcmHwParams),
            0x0416 => Some(Self::AlsaPcmParams),
            0x0417 => Some(Self::AlsaPcmPrepare),
            0x0418 => Some(Self::AlsaPcmStart),
            0x0419 => Some(Self::AlsaPcmDrop),
            0x041A => Some(Self::AlsaPcmRecover),
            0x041B => Some(Self::AlsaPcmWrite),
            0x041C => Some(Self::AlsaPcmRead),
            0x041D => Some(Self::AlsaMixerEnumerate),
            0x041E => Some(Self::AlsaMixerRead),
            0x041F => Some(Self::AlsaMixerWrite),
            0x0420 => Some(Self::AlsaJackState),
            0x0421 => Some(Self::AlsaHwdepEnumerate),
            0x0500 => Some(Self::BlkRead),
            0x0501 => Some(Self::BlkWrite),
            0x0502 => Some(Self::BlkFlush),
            0x0503 => Some(Self::BlkInfo),
            0x0504 => Some(Self::BlkTrim),
            0x0600 => Some(Self::InputPoll),
            0x0601 => Some(Self::InputSubscribe),
            0x0602 => Some(Self::InputUnsubscribe),
            0x0700 => Some(Self::NetSend),
            0x0701 => Some(Self::NetRecv),
            0x0702 => Some(Self::NetBind),
            0x0703 => Some(Self::NetUnbind),
            0x0800 => Some(Self::CapQuery),
            0x0801 => Some(Self::CapGrant),
            0x0802 => Some(Self::CapRevoke),
            0x0803 => Some(Self::CapDelegate),
            0x0900 => Some(Self::GfxViseCreateCtx),
            0x0901 => Some(Self::GfxViseDestroyCtx),
            0x0902 => Some(Self::GfxViseSwapBuffers),
            0x0903 => Some(Self::GfxViseUploadTex),
            0x0904 => Some(Self::GfxViseDrawCall),
            0x0905 => Some(Self::GfxViseSetPipeline),
            0x0906 => Some(Self::GfxViseSetUniform),
            0x0A00 => Some(Self::IommuEnumerate),
            0x0A01 => Some(Self::IommuQueryController),
            0x0A02 => Some(Self::IommuDomainCreate),
            0x0A03 => Some(Self::IommuDomainDestroy),
            0x0A04 => Some(Self::IommuBind),
            0x0A05 => Some(Self::IommuUnbind),
            0x0A06 => Some(Self::IommuMap),
            0x0A07 => Some(Self::IommuUnmap),
            0x0A08 => Some(Self::IommuInvalidate),
            0x0A09 => Some(Self::IommuReservedRegions),
            0x0A0A => Some(Self::IommuFaultRead),
            _ => None,
        }
    }
}

/// Legacy name used by ds-ipc dispatcher.
pub type Opcode = DsCmd;