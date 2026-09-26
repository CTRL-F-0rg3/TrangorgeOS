//! The video device-class framework (`ds-fw-video`).
//!
//! A sibling of `ds-fw-audio` / `ds-fw-gpu`: it holds only the **device-class
//! contract and the wire vocabulary**, never hardware register logic. The
//! hardware lives in `drivers/Video-TrangorgeOS` and the management logic in
//! `crates/ds-manager` - both depend on this crate only, which satisfies the
//! workspace rules on zero duplication and on `drivers/` never depending on
//! `crates/`.
//!
//! It is split into five pieces:
//!
//! * [`traits::VideoEngine`] - the contract a driver implements.
//! * [`sink`] - **the interface between decoded frames and whoever wants
//!   them**: a graphics server, an NPU, a capture writer. See below.
//! * [`service::VideoService`] - the server-side dispatcher, which turns a
//!   `DsMsg` into a [`traits::VideoEngine`] call and enforces capability checks.
//! * [`client::VideoClient`] - the client: encodes calls as `DsCmd` IPC.
//! * [`codec`] - the wire encoding shared by both sides.
//!
//! ## The `sink` module is the important one
//!
//! The obvious way to wire video to the rest of the system is to have the video
//! driver call into the graphics server, and to have the NPU call into the
//! video driver. Both are wrong for the same reason: they make an optional
//! component mandatory, and they make a cycle.
//!
//! So the dependency points one way, through [`sink::FrameSink`]. The video
//! driver *pushes*; a consumer *pulls*. A machine with no graphics server and no
//! NPU still decodes, into a ring nothing reads, which is a working capture
//! pipeline. A machine with both gets both, and neither knows the other exists.
//!
//! The interface is three methods, and every one of them is something a consumer
//! genuinely needs. It is deliberately not richer: a video stack whose sink
//! interface grows a field per consumer ends up able to serve none of them.
//!
//! ## Why this mirrors ALSA
//!
//! The stream vocabulary is ALSA's, for the same reason `ds-fw-audio` uses it:
//! negotiated limits, an explicit state machine, and a handle-based open/close
//! lifecycle. Ported tools and ported capture tools then read side by side with
//! the original C, and the parts that are *not* like ALSA - frames, surfaces,
//! sinks - are named for what they are rather than forced into a PCM analogy.

#![no_std]

// `Box` is used to store a `dyn FrameSink` in the registry, and the `no_std`
// prelude does not include it. `alloc` rather than `std`: a driver-space
// framework has no business assuming a hosted standard library, and the
// *driver* that owns the registry supplies the allocator anyway.
extern crate alloc;

pub mod client;
pub mod codec;
pub mod service;
pub mod sink;
pub mod traits;
pub mod types;

pub use client::VideoClient;
pub use codec::MAX_PAYLOAD_LEN;
pub use service::{required_capability, Reply, Request, VideoService};
pub use sink::{FrameSink, SinkError, SinkId, SinkRegistry, MAX_SINKS};
pub use traits::VideoEngine;
pub use types::{BufferId, EngineId, StreamHandle, MAX_POOL_SLOTS};
