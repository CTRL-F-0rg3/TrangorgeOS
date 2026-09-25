# lib-odin — Odin bindings for the TrangorgeOS communication protocol

Thin Odin client for the Rust core in `../lib` (exported as `libtg_comm.a`).
It mirrors the public protocol and declares `foreign` procedures that call the
`tgcomm_*` C ABI. The capability table, ring and the authorization gate are
opaque — Odin never re-implements them.

## Files

```
tgcomm.odin   # constants, enums, TgComm_Msg (#assert size == 64), foreign procs
```

## Usage (Odin)

```odin
import "tgcomm"

msg: tgcomm.TgComm_Msg
tgcomm.build_request(&msg, tgcomm.Layer.Driverspace, tgcomm.Layer.Kernel,
                     tgcomm.opcode(tgcomm.Op_Class.Video, 1), /*cap=*/7)

table: tgcomm.TgComm_Cap_Table   // opaque storage, 16-byte aligned
tgcomm.tgcomm_cap_table_init(&table)
tgcomm.tgcomm_cap_insert(&table, 7, 42, u8(tgcomm.Object_Type.Device),
                         tgcomm.RIGHT_CALL)

if tgcomm.tgcomm_authorize(&table, &msg) == i32(tgcomm.Result.Forwarded) {
    // forwarded — the kernel may act
}
```

Link with `-ltg_comm` (the Rust core built as `libtg_comm.a`). The Odin
compiler was not available in the build environment used to author this
binding, so the declarations follow the current `foreign`/`---` syntax and
are not machine-compiled here.
