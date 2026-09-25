# uspace — TrangorgeOS userspace (nested system + graphical environment)

Userspace is a **nested system**: it has its own capability-based permissions
(consistent with the kernel's), connects to the kernel only for *executive
actions*, talks to the driver space over IPC, and runs the graphical stack.

## Layering

```
apps  (shell, terminal, ...)
userdrivers (input, gpu, ... specific user-space drivers)
  │  IPC (uspace::ipc)          — app <-> userdriver <-> driverspace
  │  executive (uspace::kernel) — framebuffer / process actions from kernel
  ▼
uspace::caps   nested capability permissions (kernel-consistent)
  ▼
tg-comm shared-memory protocol  →  driverspace / kernel
```

## Modules

| module | role |
|---|---|
| `caps` | nested capability model: `Session` spawns children with **derived** (subset) capabilities; a child can never exceed its parent's rights (monotonicity — mirrors the `lib-ada` capability proof) |
| `ipc`  | `IpcEndpoint` over `tg_comm`; a `Router` (used on host tests; the kernel routes on the real system) |
| `kernel` | `KernelClient` — executive actions (`acquire_framebuffer`) over the shared ring |
| `gfx`  | `compositor` (z-ordered surfaces), `render` (rect/fill/8x8 text), `app` (demo window + terminal) |

## Permissions (kernel-consistent)

Userspace uses the *same* capability primitives as the kernel
(`tg_comm::CapTable`, `CapId`, `Rights`, `ObjectType`). The session receives a
capability table from the kernel and derives subsets for apps/userdrivers:

```rust
let child = session.spawn_derived(&parent, &[
    (CapId(2), CapEntry::new(20, ObjectType::Memory, Rights::READ)),  // ok: subset
]);
// WRITE would be denied: the parent does not hold WRITE over that object.
```

This is the property proven in `lib-ada`: revocation/derivation can only ever
shrink the authorization set, so the nested user space cannot escalate into a
kernel-damaging capability.

## Graphical environment

`gfx::app::draw_demo_window` composites a window (with a small terminal) into
the kernel-provided framebuffer buffer:

1. fill the background,
2. create a window surface via the compositor,
3. draw border + title bar + terminal text into the surface,
4. `composite()` the surface into the framebuffer.

The software renderer stands in for the GPU/Vise-LG pipeline on this reference
implementation; the surface/command model matches the driver-space
`gfx-protocol` (`gfx_protocol_workspace`).

## Run the tests

```sh
cd uspace && cargo test
```

The host tests exercise the nested-capability monotonicity, the IPC router and
the demo window renderer end-to-end.
