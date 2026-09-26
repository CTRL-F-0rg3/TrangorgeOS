# Alsa-TrangorgeOS

An ALSA-compatible audio stack for TrangorgeOS: the ALSA object model and the
`aplay` / `amixer` tools, ported to Rust and wired into the driver-space
architecture.

## What it is, and what it is not

It is **not** a kernel driver. It runs as a driver-space process, reaches
hardware only through resources `ds-manager` grants, and serves requests over
the same `kapi-abi` IPC the rest of the system speaks. There is no `/dev/snd`
here: the "device files" are IPC endpoints.

What it *is*, deliberately, is a faithful port of ALSA's concepts, so the
mental model survives the port:

| ALSA (C)                          | Here (Rust)                          |
|-----------------------------------|--------------------------------------|
| `snd_card`                        | [`core::Card`](src/core.rs)         |
| `snd_pcm`                         | [`core::Pcm`](src/core.rs)          |
| `snd_pcm_hw_params`               | [`core::HwParams`](src/core.rs)     |
| `snd_pcm_state`                   | [`core::Stream`](src/core.rs)       |
| `snd_mixer_selem`                 | [`core::MixerElem`](src/core.rs)    |
| `snd_ctl_elem_id`                 | `kapi_abi::payloads::audio::MixerSelem` |
| `snd_pcm_sw_params` / open        | `AudioDevice::open`                 |
| `snd_pcm_name_parse`              | [`tools::DeviceName`](src/tools.rs) |

## Layering

```text
  aplay / amixer            userspace tools (src/bin/)
         │  kapi-abi DsCmd
  ds-fw-audio::AudioClient  typed IPC client
  ds-fw-audio::AudioService  capability gating, dispatch
         │
  AlsaDriver                handle table, validation
  core                      ALSA object model
  ac97                      register map
         │  Platform trait
  ds-manager                MMIO / DMA / PCI grants
```

Each layer is testable on its own, which is why the seams exist at all. The
driver carries no IPC code, so the whole ALSA model is exercised against a
mock `Platform` with no kernel present.

## Layout

| Path | Contents |
|------|----------|
| `src/core.rs`       | the ALSA object model: cards, PCMs, hw-params, mixer elements, stream state |
| `src/ac97.rs`       | the AC97 register map, ported from the legacy Odin driver |
| `src/driver.rs`     | `AudioDevice` implementation and the `Platform` trait |
| `src/host.rs`       | `SyscallPlatform`, the production `Platform` over IPC |
| `src/tools.rs`      | `aplay` / `amixer` argument parsing and arithmetic |
| `src/main.rs`       | the service loop `ds-manager` calls |
| `src/bin/`          | the ported tools |

## Capability model

Two bits, and the split is deliberate:

* `CapId::ALSA_ENUMERATE` - list PCMs, mixer elements, jacks and hwdep, and
  query hardware parameters. Read-only.
* `CapId::ALSA_STREAM` - open, prepare, start, transfer, drop, and write mixer
  values.

The driver is granted **only** `ALSA_ENUMERATE` at start-up. Playing audio is a
decision for `ds-manager`, not something the driver grants itself - a driver
that hands itself full rights at boot makes the capability system decorative.

## Building

```sh
cargo test -p alsa-trangorgeos --lib      # the model, driver and tool parsing
cargo check -p alsa-trangorgeos --bins    # the three binaries
```

The binaries are `no_std` / `no_main` and do not link on a host: like every
driver-space binary in this workspace they need the target system and its
linker script, so `cargo build` is expected to fail at the link step on a
development host. `cargo check` is the meaningful host-side verification.

## Ported from

The vocabulary and behaviour follow [ALSA](https://www.alsa-project.org/):
`alsa-lib` for the model, `alsa-utils` for the tools. The AC97 register map was
ported from this repository's own legacy `drivers/audiodriver` (Odin), and the
offsets are asserted against it in `ac97.rs`'s tests.
