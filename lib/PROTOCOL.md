# TrangorgeOS shared-memory communication protocol — specification

Version 1. This document is the normative description of the wire protocol
implemented by `lib/` (Rust) and mirrored by `lib-C`, `lib-odin`, `lib-ada`.

## 1. Constants

| Name | Value | Meaning |
|---|---|---|
| `COMM_MAGIC` | `0x5447434D` | `"TGCM"`, little-endian |
| `COMM_VERSION` | `1` | wire version |
| `COMM_MSG_SIZE` | `64` | bytes per message |
| `RING_CONTROL_SIZE` | `16` | bytes of ring header |
| `RING_SLOTS_DEFAULT` | `256` | default slot count |
| `CAP_TABLE_SIZE` | `512` | max capability entries |

## 2. Message layout (`CommMsg`, `#[repr(C)]`, 64 bytes)

| Offset | Size | Field | Type |
|---|---|---|---|
| 0 | 4 | `magic` | u32 |
| 4 | 1 | `version` | u8 |
| 5 | 1 | `kind` | u8 (`MsgKind`) |
| 6 | 1 | `layer` | u8 (`Layer`, source) |
| 7 | 1 | `target` | u8 (`Layer`, destination) |
| 8 | 4 | `opcode` | u32 |
| 12 | 4 | `cap` | u32 (`CapId`, `u32::MAX` = invalid) |
| 16 | 4 | `seq` | u32 |
| 20 | 4 | `status` | i32 |
| 24 | 32 | `a0..a3` | u64 × 4 |
| 56 | 8 | `reserved` | u8 × 8 |

## 3. Layers and routing

```
Kernel = 0, Driverspace = 1, Userspace = 2, Manager = 3, UserDriverSpace = 4
```

A request is only routed when `may_target(src, dst)` holds:

| src \ dst | Kernel | Driverspace | Userspace | Manager | UDS |
|---|---|---|---|---|---|
| Kernel | – | ✓ | ✓ | ✓ | ✓ |
| Driverspace | ✓ | – | – | ✓ | – |
| Userspace | ✓ | – | – | ✓ | – |
| Manager | ✓ | ✓ | ✓ | – | ✓ |
| UserDriverSpace | ✓ | – | – | ✓ | – |

## 4. Rights bitmask

`READ(0) WRITE(1) EXEC(2) MAP(3) GRANT(4) TRANSFER(5) SEND(6) RECV(7)
CALL(8) MANAGE(9)`. `contains(a,b)` = `(a & b) == b`.

## 5. Object types

`Memory=0 Endpoint=1 Notification=2 Channel=3 ShmemRegion=4 Device=5 Irq=6`.

## 6. Opcodes and requirements

`opcode = (class << 8) | op`. Classes: `Sys=0 Mem=1 Cap=2 Ipc=3 Shmem=4
Video=5 Audio=6 Input=7 Block=8 Net=9 Pci=10 Vgpu=11 Fs=12`.

`required(op)` returns the `(rights, obj_type)` a capability must satisfy;
an unknown/reserved opcode returns `None` and is **never** forwarded.

| Class | rights | obj_type |
|---|---|---|
| Sys | CALL | any |
| Mem create(1) | MANAGE | Memory |
| Mem map(3) | MAP\|WRITE | Memory |
| Mem grant(5) | GRANT\|MAP | Memory |
| Cap | MANAGE | any |
| Ipc send(1)/call(3) | SEND | Endpoint |
| Ipc recv(2)/reply(4) | RECV | Endpoint |
| Shmem create(1) | MANAGE | ShmemRegion |
| Shmem attach/detach | MAP | ShmemRegion |
| device classes | CALL | Device |

## 7. Ring (shared-memory SPSC)

```
[ RingControl: head u32 | tail u32 | capacity u32 | pad u32 ]   (16 bytes)
[ slot 0: CommMsg (64 bytes) ]
[ slot 1: CommMsg (64 bytes) ]
...
```

`head` = producer index, `tail` = consumer index, in message units. One slot
is always kept free to disambiguate full from empty.

## 8. Capability table

A fixed array of 512 entries; an entry is `{ obj_id u64, obj_type u8,
valid u8, pad u16, rights u32 }`. Absence (or `valid == 0`) ⇒ deny.

## 9. The authorization gate

```
authorize(table, msg):
  if msg.magic != MAGIC or msg.version != VERSION  -> Malformed
  if layers undecodable                            -> Malformed
  if !may_target(src, dst)                         -> RouteDenied
  if opcode unknown/reserved                        -> UnknownOp
  if cap absent in table                            -> NoCapability
  if !rights.contains(required_rights)              -> RightsInsufficient
  if required_type present and mismatch             -> TypeMismatch
  -> Forwarded
```

The kernel handler is invoked **only** on `Forwarded`. This is the invariant
proven in `lib-ada`.
