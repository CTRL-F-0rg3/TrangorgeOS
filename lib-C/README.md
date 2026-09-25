# lib-C — C bindings for the TrangorgeOS communication protocol

Thin C client for the Rust core in `../lib`. It mirrors the **public**
protocol (message layout, layers, opcodes, rights, object types) in
`include/tgcomm.h`, but the capability table, the shared-memory ring and the
authorization gate stay **opaque**: C only calls the `tgcomm_*` C ABI exported
by `libtg_comm.a`, it never re-implements the logic.

## Layout

```
include/tgcomm.h   # protocol types + tgcomm_* declarations
src/tgcomm.c       # thin helper (tgcomm_ring_bytes -> Rust)
test/test.c        # smoke test (authorize, revoke, ring round-trip)
Makefile
```

## Build & test

```sh
make            # build libtgcomm_c.a (builds the Rust core first)
make test       # build + link + run the smoke test against libtg_comm.a
make clean
```

The test proves, from C, that:

* a missing capability is denied,
* READ rights do not authorize a CALL,
* a forbidden route (userspace → driverspace) is denied,
* revocation flips a forwarded request back to a deny,
* the ring round-trips a message byte-for-byte.
