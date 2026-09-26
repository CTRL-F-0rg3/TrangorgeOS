//! The audio device-class framework (`ds-fw-audio`).
//!
//! A sibling of `ds-fw-iommu` / `ds-fw-gpu`: it holds only the **device-class
//! contract and the wire vocabulary**, never hardware register logic. The
//! hardware lives in `drivers/Alsa-TrangorgeOS` and the management logic in
//! `crates/ds-manager` - both depend on this crate only, which satisfies the
//! workspace rules on zero duplication and on `drivers/` never depending on
//! `crates/`.
//!
//! It is split into four pieces:
//!
//! * [`traits::AudioDevice`] - the contract a driver implements.
//! * [`service::AudioService`] - the server-side dispatcher: turns a `DsMsg`
//!   into an [`traits::AudioDevice`] call and enforces capability checks.
//! * [`client::AudioClient`] - the client: encodes calls as `DsCmd` IPC.
//! * [`codec`] - the wire encoding shared by both sides.
//!
//! ## Why this mirrors ALSA
//!
//! The vocabulary is deliberately ALSA's, not a fresh invention: a card owns
//! PCM endpoints, parameters are negotiated field by field, and a mixer element
//! is addressed by a name plus a coordinate. That is what lets `aplay` and
//! `amixer` be ported with their control flow intact instead of being
//! redesigned. The wire format (`#[repr(C)]` payloads plus opcodes) comes
//! entirely from `kapi-abi`; this crate adds no ABI surface of its own.

#![no_std]

pub mod client;
pub mod codec;
pub mod service;
pub mod traits;
pub mod types;

pub use client::AudioClient;
pub use codec::MAX_PAYLOAD_LEN;
pub use service::{required_capability, AudioService, Reply, Request};
pub use traits::AudioDevice;
pub use types::{
    CardId, DropMode, HwdepId, JackId, MixerId, PcmHandle, PcmId, Transfer,
};
