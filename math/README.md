# TrangorgeOS Math Stack

A `#![no_std]` mathematical library stack for the kernel, drivers and the
graphics stack. Everything here is written from scratch, is `forbid(unsafe_code)`,
and is verified against the system's own error vocabulary.

## Layout

```
math/
└── crates/
    └── math-core/     # shared substrate (implemented)
        ├── bits.rs    #   float classification + field extraction
        ├── consts.rs  #   constants in f32 and f64
        ├── approx.rs  #   approximate comparison, lerp, remap
        ├── error.rs   #   errors mapped onto kapi_abi::Status
        └── feature.rs #   compile-time target capability detection
```

## Design rules

These hold for every crate in the stack, and are enforced rather than merely
documented:

1. **`#![no_std]` and `#![forbid(unsafe_code)]`**, the latter enforced at the
   workspace level through `[workspace.lints]`. Float bit manipulation uses the
   safe `to_bits` / `from_bits` pair, so every result is a pure function of its
   input with no platform escape hatch hiding a rounding decision.
2. **No hidden allocation.** Fixed-capacity arrays only. A math routine that
   needs memory takes a slice from the caller.
3. **Errors speak the system's language.** [`math_core::error::MathError`]
   converts into `kapi_abi::Status`, so a math failure travels through
   `ds-manager` and IPC like any other error.
4. **Target capabilities are decided at compile time**, not probed at boot -
   see `math_core::feature`.

## Building

```sh
cd math
cargo test  -p math-core
cargo check -p math-core --target x86_64-unknown-none
```

The kernel targets are `x86_64-unknown-none` and `riscv64gc-unknown-none-elf`
(see the repository `rust-toolchain.toml`).

## Roadmap

The requested set is 35 libraries. The order below is deliberate: each layer
depends only on the ones above it, and nothing below is started until the layer
above is green.

| # | Crate | Replaces | State |
|---|---|---|---|
| 1 | `math-core` | shared substrate | **done** - 25 tests |
| 2 | `math-fp` | `libm`, `micromath` | planned |
| 3 | `math-vector` | `glam`, `nalgebra`, `euclid` | planned |
| 4 | `math-fixed` | `fixed`, `fpdec` | planned |
| 5 | `math-bigint` | `ruint`, `ethnum`, `ibig` | planned |
| 6 | `math-complex` | `num-complex` | planned |
| 7 | `math-kdtree` | `kdtree` | planned |
| 8 | `math-splines` | `splines`, `keyframe` | planned |
| 9 | `math-dsp` | `realfft`, `microfft`, `biquad` | planned |
| 10 | `math-hash` | `crc32fast`, `xxhash-rust`, `siphasher` | planned |
| 11 | `math-rand` | `rand_core`, `fastrand` | planned |
| 12 | `math-crypto` | `curve25519-dalek`, `k256`, `p256`, `subtle` | planned, see warning |
| 13 | `math-geo` | `geo-types`, `spade` | planned |

### A note on the security-critical entries

`math-crypto` would cover elliptic curves and constant-time primitives. Writing
those from scratch is a genuinely bad idea: timing side channels and subtle
field-arithmetic bugs are exactly the failure modes that hand-written crypto
produces, and they do not show up in tests. The plan is to implement them as
clean, auditable arithmetic (RFC 6234 / FIPS 180-4 style, with constant-time
selection at the representation level) and to document the limits plainly - but
anything that actually **signs** something should still use a vetted crates.io
implementation.

## Accuracy policy

Each crate states its target ULP and its test tolerances. A routine that cannot
meet the stated bound is documented as such rather than quietly rounded up -
`math-core` already applies this: the `f32` Cody-Waite split constants were
removed because the single-precision remainder falls below one ULP and the split
buys nothing, and the documentation says so instead of pretending otherwise.
