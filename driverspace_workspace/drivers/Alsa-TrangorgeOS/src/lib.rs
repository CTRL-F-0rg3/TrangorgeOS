//! `Alsa-TrangorgeOS` - the ALSA-compatible audio stack for TrangorgeOS.
//!
//! This is a Rust port of the ALSA model (`alsa-lib`) plus the two user-space
//! tools that matter most (`alsa-utils`: `aplay` and `amixer`), wired into the
//! driver-space architecture instead of sitting in the kernel.
//!
//! ## What this is, and what it is not
//!
//! It is **not** a kernel driver. It runs as a driver-space process, reaches
//! hardware only through resources `ds-manager` grants, and serves requests
//! over the same `kapi-abi` IPC the rest of the system speaks. That is why
//! there is no `/dev/snd` here: the "device files" are IPC endpoints.
//!
//! What it *is*, deliberately, is a faithful port of ALSA's concepts, so the
//! mental model survives the port:
//!
//! | ALSA (C)                      | Here (Rust)                          |
//! |-------------------------------|--------------------------------------|
//! | `snd_card`                    | [`core::Card`]                       |
//! | `snd_pcm`                     | [`core::Pcm`]                        |
//! | `snd_pcm_hw_params`           | [`core::HwParams`]                   |
//! | `snd_pcm_state`               | [`core::Stream`]                     |
//! | `snd_mixer_selem`             | [`core::MixerElem`]                  |
//! | `snd_ctl_elem_id`             | `kapi_abi::payloads::audio::MixerSelem` |
//! | `snd_pcm_sw_params` (open)    | `AudioDevice::open`                  |
//!
//! ## Layering
//!
//! ```text
//!   aplay / amixer            (userspace tools, IPC clients)
//!          │  kapi-abi DsCmd
//!   ds-fw-audio::AudioClient  (typed IPC client)
//!   ds-fw-audio::AudioService (capability gating, dispatch)
//!          │
//!   AlsaDriver                (handle table, validation)
//!   core                      (ALSA object model)
//!   ac97                      (register map)
//!          │  Platform trait
//!   ds-manager                 (MMIO / DMA / PCI grants)
//! ```
//!
//! Each layer is testable on its own, which is why the seams exist at all.

#![no_std]

pub mod ac97;
pub mod core;
pub mod driver;
pub mod host;
pub mod tools;

/// The crate version, as a string suitable for `--version` output.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The driver name, as it appears in logs and in the device registry.
pub const DRIVER_NAME: &str = "Alsa-TrangorgeOS";
