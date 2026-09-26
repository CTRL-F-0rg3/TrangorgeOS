//! The audio client: encodes framework calls as `DsCmd` IPC.
//!
//! Modelled on `ds-ipc::ManagerClient`: it only ships a request out and reads
//! the reply back, and holds no device state. Callers get a [`DsError`] they can
//! propagate directly.
//!
//! ## Payload lifetime
//!
//! Requests with a payload put the address of a stack struct in `arg0` and its
//! length in `arg1`. That relies on the synchronous semantics of
//! `kapi_syscall::sys_ipc_call`: the kernel has already read the payload by the
//! time the reply arrives.

use kapi_abi::{
    DsCmd, DsError, Handle,
    payloads::audio::{
        HwParamsQuery, HwParamsResult, JackState, MixerSelem, MixerValue, PcmInfo, PcmOpenPayload,
        PcmParams, PcmState, PcmStream,
    },
    wire::{Decoder, Encoder},
};
use kapi_syscall::sys_ipc_call;

use crate::{codec, types::*, MAX_PAYLOAD_LEN};

/// Handle of the audio service endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioClient {
    service: Handle,
}

impl AudioClient {
    #[inline]
    pub const fn new(service: Handle) -> Self {
        Self { service }
    }

    #[inline]
    pub const fn endpoint(&self) -> Handle {
        self.service
    }

    // ── enumeration ─────────────────────────────────────────────────────────────────────────────────────

    /// Number of PCM endpoints the service exposes.
    pub fn pcm_count(&self) -> Result<u32, DsError> {
        let reply = sys_ipc_call(DsCmd::AlsaPcmEnumerate, u32::MAX as u64, 0, 0);
        check(reply.status).map(|_| reply.arg0 as u32)
    }

    /// Read the descriptor of PCM endpoint `pcm`.
    pub fn pcm_info(&self, pcm: PcmId) -> Result<PcmInfo, DsError> {
        let reply = sys_ipc_call(DsCmd::AlsaPcmInfo, pcm.index() as u64, 0, 0);
        check(reply.status)?;
        let buf = [0u8; MAX_PAYLOAD_LEN];
        let mut decoder = Decoder::new(&buf[..payload_len(&reply)]);
        codec::decode_pcm_info(&mut decoder).ok_or(DsError::BufferTooSmall)
    }

    /// Current state of an open PCM.
    pub fn state(&self, pcm: PcmHandle) -> Result<PcmState, DsError> {
        check_handle(pcm)?;
        let reply = sys_ipc_call(DsCmd::AlsaPcmState, pcm.raw() as u64, 0, 0);
        check(reply.status).map(|_| PcmState::from_u32(reply.arg0 as u32))
    }

    /// The parameters currently negotiated for `pcm`.
    pub fn params(&self, pcm: PcmHandle) -> Result<PcmParams, DsError> {
        check_handle(pcm)?;
        let reply = sys_ipc_call(DsCmd::AlsaPcmParams, pcm.raw() as u64, 0, 0);
        check(reply.status)?;
        let buf = [0u8; MAX_PAYLOAD_LEN];
        let mut decoder = Decoder::new(&buf[..payload_len(&reply)]);
        codec::decode_params(&mut decoder).ok_or(DsError::BufferTooSmall)
    }

    /// Read one hardware-parameter field.
    pub fn hw_params_get(&self, pcm: PcmHandle, field: u32) -> Result<HwParamsResult, DsError> {
        self.hw_params(pcm, field, false, 0)
    }

    /// Narrow one hardware-parameter field to `value`.
    pub fn hw_params_set(
        &self,
        pcm: PcmHandle,
        field: u32,
        value: u64,
    ) -> Result<HwParamsResult, DsError> {
        self.hw_params(pcm, field, true, value)
    }

    /// Shared body of [`Self::hw_params_get`] and [`Self::hw_params_set`].
    fn hw_params(
        &self,
        pcm: PcmHandle,
        field: u32,
        set: bool,
        value: u64,
    ) -> Result<HwParamsResult, DsError> {
        check_handle(pcm)?;
        if field >= HwParamsQuery::FIELD_COUNT {
            return Err(DsError::InvalidMessage);
        }
        let mut payload = [0u8; MAX_PAYLOAD_LEN];
        let len = {
            let mut encoder = Encoder::new(&mut payload);
            let ok = encoder.write_u32(pcm.raw())
                && encoder.write_u32(field)
                && encoder.write_u32(set as u32)
                && encoder.write_u64(value);
            if !ok {
                return Err(DsError::BufferTooSmall);
            }
            encoder.pos()
        };
        let reply = sys_ipc_call(
            DsCmd::AlsaPcmHwParams,
            payload.as_ptr() as u64,
            len as u64,
            0,
        );
        check(reply.status)?;
        let mut decoder = Decoder::new(&payload[..payload_len(&reply)]);
        codec::decode_hw_params_result(&mut decoder).ok_or(DsError::BufferTooSmall)
    }

    // ── stream lifecycle ───────────────────────────────────────────────────────────────────────────────

    /// Open a PCM endpoint.
    pub fn open(
        &self,
        card: CardId,
        device: PcmId,
        stream: PcmStream,
        access: u32,
    ) -> Result<PcmHandle, DsError> {
        let request = PcmOpenPayload {
            card: card.index(),
            device: device.index(),
            subdevice: 0,
            stream,
            access,
            flags: 0,
        };
        let mut payload = [0u8; MAX_PAYLOAD_LEN];
        let len = {
            let mut encoder = Encoder::new(&mut payload);
            if !codec::encode_open(&mut encoder, &request) {
                return Err(DsError::BufferTooSmall);
            }
            encoder.pos()
        };
        let reply =
            sys_ipc_call(DsCmd::AlsaPcmOpen, payload.as_ptr() as u64, len as u64, 0);
        check(reply.status)?;
        let handle = PcmHandle(reply.arg0 as u32);
        if !handle.is_valid() {
            return Err(DsError::DeviceFault);
        }
        Ok(handle)
    }

    /// Close a PCM, releasing its buffer.
    pub fn close(&self, pcm: PcmHandle) -> Result<(), DsError> {
        check_handle(pcm)?;
        check(sys_ipc_call(DsCmd::AlsaPcmClose, pcm.raw() as u64, 0, 0).status)
    }

    /// Move a stream to `Prepared`.
    pub fn prepare(&self, pcm: PcmHandle) -> Result<(), DsError> {
        check_handle(pcm)?;
        check(sys_ipc_call(DsCmd::AlsaPcmPrepare, pcm.raw() as u64, 0, 0).status)
    }

    /// Move a `Prepared` stream to `Running`.
    pub fn start(&self, pcm: PcmHandle) -> Result<(), DsError> {
        check_handle(pcm)?;
        check(sys_ipc_call(DsCmd::AlsaPcmStart, pcm.raw() as u64, 0, 0).status)
    }

    /// Stop a stream, optionally playing out the buffer first.
    pub fn drop_stream(&self, pcm: PcmHandle, mode: DropMode) -> Result<(), DsError> {
        check_handle(pcm)?;
        check(sys_ipc_call(DsCmd::AlsaPcmDrop, pcm.raw() as u64, mode as u64, 0).status)
    }

    /// `Prepare` followed by `Start`, the usual xrun recovery.
    pub fn recover(&self, pcm: PcmHandle) -> Result<(), DsError> {
        check_handle(pcm)?;
        check(sys_ipc_call(DsCmd::AlsaPcmRecover, pcm.raw() as u64, 0, 0).status)
    }

    /// Write `frames` frames of interleaved samples.
    pub fn write(&self, pcm: PcmHandle, data: &[u8], frames: u32) -> Result<Transfer, DsError> {
        check_handle(pcm)?;
        if frames == 0 {
            return Err(DsError::InvalidMessage);
        }
        let reply = sys_ipc_call(
            DsCmd::AlsaPcmWrite,
            pcm.raw() as u64,
            frames as u64,
            data.as_ptr() as u64,
        );
        check(reply.status).map(|_| Transfer::from_frames(reply.arg0 as u32, frames))
    }

    /// Read `frames` frames of interleaved samples.
    pub fn read(&self, pcm: PcmHandle, frames: u32) -> Result<Transfer, DsError> {
        check_handle(pcm)?;
        if frames == 0 {
            return Err(DsError::InvalidMessage);
        }
        let reply = sys_ipc_call(DsCmd::AlsaPcmRead, pcm.raw() as u64, frames as u64, 0);
        check(reply.status).map(|_| Transfer::from_frames(reply.arg0 as u32, frames))
    }

    // ── mixer ──────────────────────────────────────────────────────────────────────────────────────────

    /// Read the descriptor of mixer element `mixer`.
    pub fn mixer_info(&self, mixer: MixerId) -> Result<MixerSelem, DsError> {
        let reply = sys_ipc_call(DsCmd::AlsaMixerEnumerate, mixer.index() as u64, 0, 0);
        check(reply.status)?;
        let buf = [0u8; MAX_PAYLOAD_LEN];
        let mut decoder = Decoder::new(&buf[..payload_len(&reply)]);
        codec::decode_mixer_selem(&mut decoder).ok_or(DsError::BufferTooSmall)
    }

    /// Read the current value of mixer element `mixer`.
    pub fn mixer_read(&self, mixer: MixerId) -> Result<MixerValue, DsError> {
        let reply = sys_ipc_call(DsCmd::AlsaMixerRead, mixer.index() as u64, 0, 0);
        check(reply.status)?;
        let buf = [0u8; MAX_PAYLOAD_LEN];
        let mut decoder = Decoder::new(&buf[..payload_len(&reply)]);
        codec::decode_mixer_value(&mut decoder).ok_or(DsError::BufferTooSmall)
    }

    /// Write a value to mixer element `mixer`, returning what was stored.
    pub fn mixer_write(&self, value: &MixerValue) -> Result<MixerValue, DsError> {
        let mut payload = [0u8; MAX_PAYLOAD_LEN];
        let len = {
            let mut encoder = Encoder::new(&mut payload);
            if !codec::encode_mixer_value(&mut encoder, value) {
                return Err(DsError::BufferTooSmall);
            }
            encoder.pos()
        };
        let reply =
            sys_ipc_call(DsCmd::AlsaMixerWrite, payload.as_ptr() as u64, len as u64, 0);
        check(reply.status)?;
        let buf = [0u8; MAX_PAYLOAD_LEN];
        let mut decoder = Decoder::new(&buf[..payload_len(&reply)]);
        codec::decode_mixer_value(&mut decoder).ok_or(DsError::BufferTooSmall)
    }

    // ── jack ───────────────────────────────────────────────────────────────────────────────────────────

    /// Read the state of jack `jack`.
    pub fn jack_state(&self, jack: JackId) -> Result<JackState, DsError> {
        let reply = sys_ipc_call(DsCmd::AlsaJackState, jack.index() as u64, 0, 0);
        check(reply.status)?;
        let buf = [0u8; MAX_PAYLOAD_LEN];
        let mut decoder = Decoder::new(&buf[..payload_len(&reply)]);
        codec::decode_jack_state(&mut decoder).ok_or(DsError::BufferTooSmall)
    }
}

/// `Ok` when the reply status is 0, otherwise translated into a [`DsError`].
#[inline]
fn check(status: i32) -> Result<(), DsError> {
    if status == 0 {
        Ok(())
    } else {
        Err(DsError::from_u32(status as u32))
    }
}

/// Extract the payload length from a reply, clamped to the local buffer size.
#[inline]
fn payload_len(reply: &kapi_abi::DsMsg) -> usize {
    let len = reply.arg2 as usize;
    if len > MAX_PAYLOAD_LEN {
        MAX_PAYLOAD_LEN
    } else {
        len
    }
}
