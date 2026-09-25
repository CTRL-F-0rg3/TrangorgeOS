# `tg-comm` — strict shared-memory communication protocol for TrangorgeOS

The **single, independent** definition of how the layers of TrangorgeOS talk
to each other through shared memory, and of *what they are allowed to ask*.

This crate is **not a member of any Cargo workspace** (the `[workspace]` key
makes this directory its own isolated workspace). It is the "single source of
truth" for the protocol; the language bindings in `lib-C`, `lib-odin` and
`lib-ada` are thin clients that call into it.

## What it defines

| Concern | Where |
|---|---|
| Wire message (64-byte `CommMsg`) | `src/wire.rs` |
| Layers + routing matrix | `src/layer.rs` |
| Capability model (`CapId`, `ObjectType`, `CapEntry`, `CapTable`) | `src/caps.rs` |
| Rights bitmask | `src/rights.rs` |
| Opcodes + per-opcode requirements | `src/opcodes.rs` |
| Shared-memory SPSC ring | `src/ring.rs` |
| Shared-memory regions | `src/region.rs` |
| **The authorization gate** | `src/filter.rs` |
| Layer channel (wide API) | `src/channel.rs` |
| Driver manager facade | `src/manager.rs` |
| Stable C ABI | `src/ffi.rs` |

## Security model

Every request carries a capability handle. The kernel-side `authorize` gate
checks, in order:

1. structural validity (magic, version, layers),
2. routing policy (`Layer::may_target`),
3. that the opcode is known and non-reserved,
4. that the capability exists, carries the required rights, and refers to an
   object of the required type.

Only `Forwarded` requests reach the kernel. The full argument that this makes
unauthorized actions impossible is in `lib-ada` (SPARK) and in `PROTOCOL.md`.

## Build & test

```sh
cargo build           # no_std rlib + staticlib + cdylib
cargo test            # 11 tests proving "only authorized requests pass"
cargo build --release # -> target/release/libtg_comm.a (for C / Ada / Odin)
```

See `PROTOCOL.md` for the complete wire specification, and the sibling
folders `lib-C`, `lib-odin`, `lib-ada` for the cross-language bindings.
