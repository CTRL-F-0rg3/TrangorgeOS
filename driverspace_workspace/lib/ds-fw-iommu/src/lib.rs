//! The IOMMU device-class framework (`ds-fw-iommu`).
//!
//! A sibling of `ds-fw-gpu` / `ds-fw-audio`: it holds only the **device-class
//! contract and the wire vocabulary**, never hardware register logic. The
//! hardware lives in `drivers/iommu-driver` and the management logic in
//! `crates/ds-manager` - both depend on this crate only, which satisfies the
//! workspace rules on zero duplication and on `drivers/` never depending on
//! `crates/`.
//!
//! It is split into three pieces:
//!
//! * [`traits::IommuDevice`] - the contract a driver implements.
//! * [`service::IommuService`] - the server-side dispatcher: turns a `DsMsg`
//!   into a [`traits::IommuDevice`] call and enforces capability checks.
//! * [`client::IommuClient`] - the client: encodes calls as `DsCmd` IPC.
//!
//! The wire format (`#[repr(C)]` payloads plus opcodes) comes entirely from
//! `kapi-abi`; this crate adds no ABI surface of its own.

#![no_std]

pub mod client;
pub mod codec;
pub mod service;
pub mod traits;
pub mod types;

pub use client::IommuClient;
pub use codec::MAX_PAYLOAD_LEN;
pub use service::{IommuService, Reply, Request};
pub use traits::IommuDevice;
pub use types::{ControllerId, DomainId, IoRange, RequesterId};
