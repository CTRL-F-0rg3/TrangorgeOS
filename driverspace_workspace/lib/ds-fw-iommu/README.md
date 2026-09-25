# `ds-fw-iommu` - IOMMU Device-Class Framework

The TrangorgeOS driver-space contract for IOMMU devices. This crate holds the
**contract and the wire vocabulary only** - never hardware register logic.

Hardware lives in `drivers/iommu-driver`; management logic lives in
`crates/ds-manager`. Both depend on this crate and nothing else, which is what
satisfies the workspace rule that `drivers/` must never depend on `crates/` and
the rule against duplicated code.

## Layout

| Module | Role |
|---|---|
| `traits` | `IommuDevice` - what a driver implements |
| `service` | `IommuService` - server-side dispatch **with capability enforcement** |
| `client` | `IommuClient` - client-side calls encoded as `DsCmd` IPC |
| `codec` | payload encode/decode, shared by both sides |
| `types` | `ControllerId`, `DomainId`, `RequesterId`, `IoRange` |

Every wire type is `#[repr(C)]` and comes from `kapi-abi`. This crate adds **no**
ABI surface of its own - opcodes and payload layouts live in one place.

## The three roles

```
   caller                 driver process
   -------                 --------------
   IommuClient  --DsCmd-->  IommuService --> IommuDevice
   (encodes)               (checks caps)   (hardware)
        ^                        |
        +------- reply payload --+
```

- A **driver** implements `IommuDevice` and wraps it in an `IommuService`.
- A **caller** (userspace service, or `ds-manager` acting on someone's behalf)
  uses `IommuClient` to issue requests.
- `IommuService` is the only place capabilities are checked.

## `IommuDevice`

```rust
pub trait IommuDevice {
    // discovery
    fn controller_count(&self) -> usize;
    fn controller_info(&self, index: usize, out: &mut IommuControllerInfo) -> bool;

    // domain lifecycle
    fn domain_create(&mut self, controller: ControllerId, hint: u32)
        -> Result<DomainId, DsError>;
    fn domain_destroy(&mut self, domain: DomainId) -> Result<(), DsError>;

    // binding
    fn bind(&mut self, request: &IommuBindPayload) -> Result<(), DsError>;
    fn unbind(&mut self, controller: ControllerId, requester: RequesterId)
        -> Result<(), DsError>;

    // mapping
    fn map(&mut self, request: &IommuMapPayload) -> Result<u64, DsError>;
    fn unmap(&mut self, request: &IommuUnmapPayload) -> Result<(), DsError>;
    fn invalidate(&mut self, request: &IommuInvalidatePayload)
        -> Result<IommuInvalidateScope, DsError>;

    // firmware reservations
    fn reserved_region_count(&self) -> usize;
    fn reserved_region(&self, index: usize, out: &mut IommuReservedRegionPayload) -> bool;

    // faults
    fn pending_faults(&self) -> usize;
    fn read_fault(&mut self, index: usize, out: &mut IommuFaultPayload) -> bool;
}
```

Properties that keep the trait usable as a framework boundary:

- **Object-safe.** No generics, no `Self` in return position.
- **No `kore_memory` types.** The framework must not inherit the hardware
  core's dependencies; everything crosses as plain integers.
- **Index-based getters return `bool`.** Out-of-range returns `false` and
  leaves `out` untouched - no sentinel values to guess at.
- **Every fallible method returns `kapi_abi::DsError`**, which is also the wire
  status, so no translation layer is needed.

### Domain ownership rule

Domains are allocated by the driver and named by `DomainId`. **Mapping requests
always carry a domain** - that is what keeps one device's IOVA space from being
shared with another's by accident. A caller wanting pass-through semantics
creates a domain and maps everything into it, rather than reaching for a
domain-less side path.


## `IommuService`

```rust
let mut service = IommuService::new(device);
service.grant(CapId::IOMMU_ENUMERATE);

let reply = service.dispatch(&Request::new(msg, &payload_bytes), &mut out);
```

- `dispatch` decodes the opcode, checks the required capability, decodes the
  payload, calls the device, and encodes the reply.
- **Capabilities are checked here and nowhere else.** A driver implementation
  should contain no permission logic, so there is exactly one place to audit.
- A reply with a payload writes it into `out` and reports the byte count in
  `Reply::arg2`. A too-small buffer yields `DsError::BufferTooSmall`, so the
  caller can always distinguish truncation from an empty reply.
- A non-IOMMU opcode is rejected with `DsError::InvalidMessage` - even if the
  caller holds every capability.

### Capability map

| Capability | Bit | Opcodes |
|---|---|---|
| `IOMMU_ENUMERATE` | `1 << 20` | `IommuEnumerate`, `IommuQueryController`, `IommuReservedRegions`, `IommuFaultRead` |
| `IOMMU_DOMAIN` | `1 << 21` | `IommuDomainCreate`, `IommuDomainDestroy` |
| `IOMMU_BIND` | `1 << 22` | `IommuBind`, `IommuUnbind` |
| `IOMMU_MAP` | `1 << 23` | `IommuMap`, `IommuUnmap`, `IommuInvalidate` |

`required_capability(cmd)` exposes the mapping, so a manager can implement
policy against it without duplicating the table.

## `IommuClient`

Mirrors `ds-ipc::ManagerClient`: it ships a request and reads the reply, and
holds no device state. Failures come back as `DsError` and are meant to be
propagated with `?`.

```rust
let client = IommuClient::new(service_handle);
let count = client.controller_count()?;
let domain = client.create_domain(ControllerId(0))?;
client.bind(ControllerId(0), RequesterId::new(0, 0x2a, 5, 3), domain)?;
```

Argument validation that can be done locally is done locally: an invalid
`DomainId`, a `RequesterId::NONE`, or a misaligned map size is rejected before
any IPC round trip.

## Payload passing

Structured payloads follow the convention already used by `LogPayload`: the
pointer goes in `DsMsg::arg0`, the length in `arg1`. `MAX_PAYLOAD_LEN` is the
worst-case encoded size of any IOMMU payload (48 bytes) and sizes the scratch
buffers on both sides.

The framework itself contains **no raw pointers**: the transport copies bytes
into a `Request` slice, and the service writes its reply into a caller-provided
`&mut [u8]`. That is what lets the tests feed plain byte slices.

## Types

| Type | Wire form | Notes |
|---|---|---|
| `ControllerId` | `u32` | index into `0..controller_count` |
| `DomainId` | `u32` | `DomainId::INVALID` = `u32::MAX`; `is_valid()` checks it |
| `RequesterId` | `u32` | `(segment << 16) \| bdf`; `NONE` = unowned; `Display` prints `ssss:bb:dd.f` |
| `IoRange` | `{ iova: u64, size: u64 }` | `is_valid`, `is_aligned`, `pages` |

The newtypes exist so a controller index cannot be passed where a domain id
belongs. `RequesterId`'s layout matches the driver's `PciDevice`, so conversion
between the two is free.

## Testing

17 unit tests, no kernel required:

- payload round-trips for **every** struct (`every_payload_round_trips`),
  including the padding fields that a naive codec silently misaligns;
- truncated payloads are rejected rather than read past the end;
- capability gating: mutation without the capability is `PermissionDenied`;
- a non-IOMMU opcode is rejected even with `CapId::KERNEL_ALL`;
- a too-small reply buffer is reported, not silently truncated;
- argument validation in the client that never reaches IPC.

```sh
cargo test -p ds-fw-iommu --lib
```

## See also

- [`../../drivers/iommu-TrangorgeOS/README.md`](../../drivers/iommu-TrangorgeOS/README.md) -
  the driver built on this framework.
- [`../kapi-abi/src/payloads/iommu.rs`](../kapi-abi/src/payloads/iommu.rs) -
  the payload definitions this codec encodes.
