# `ustd` — a real Rust `std` for TrangorgeOS userspace

Every Rust crate in this repository is `no_std`: the kernel and its drivers build
with `-Zbuild-std=core,alloc,compiler_builtins`, and a userspace program gets
`Vec` and `String` and nothing else. `ustd` is what changes that. It builds
**upstream, unmodified `std`** for TrangorgeOS ring 3, so a program here is an
ordinary Rust program — `HashMap`, `format!`, `println!`, `std::fs`, `std::env`,
`std::time`, `std::thread` — with no `no_std` and no `unsafe`.

```rust
// init/src/main.rs — the whole program, no unsafe, no no_std
fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("{args:?} on {}", std::env::consts::OS);
}
```

## Why it is built this way

`std` is a compiled artifact, not a trait. There are two ways to get one:

| Approach | What it costs |
|----------|---------------|
| **Re-implement `std`** as a TrangorgeOS-flavored crate | Every program has to learn the new API; nothing written for Linux compiles; the `std` you get is as old as the day you wrote it. |
| **A sysroot** — a target spec, a platform layer and a C runtime, then compile real `std` | ~600 lines of glue. Everything on crates.io that uses `std` keeps working. |

This is the second one, and `ustd` is the second one's directory.

## Layers

```text
  upstream std  →  tgs-libc  →  tgs-hal  →  int 0x80  →  kernel
```

| Crate | What it is | Host-testable |
|-------|------------|---------------|
| [`tgs-hal`](hal) | The syscall ABI: the number table, the gate, errno decoding, the descriptor table, the `mmap` heap, the `linux_dirent64` codec. | **Yes — 58 tests, no target needed.** |
| [`tgs-libc`](libc) | The `#[no_mangle] extern "C"` symbols `std` links against, and the `#[global_allocator]` it allocates through. | No — it defines `malloc`, so it is behind a feature. |
| [`tgs-rt`](rt) | `_start` (the ELF entry point), the stack walk that finds `argc`/`argv`/`envp`/auxv, and the fallback when the kernel has not built one. | No — it defines the entry point. |
| [`tgs-init`](init) | A binary that uses the whole `std` surface, and reports what works. | It is the program. |

`hal` is the interesting one, and it is why this directory is worth having even
before the kernel catches up. Everything in it is written against a `Syscall`
trait rather than an `asm!` block, so the allocator's free lists, the
descriptor-number rules and the dirent layout are all exercised on a host against
a fake kernel that hands out *real* addresses. Those tests found twelve bugs while
this was being written, two of them unsound.

## Build

```sh
just uspace-std          # or: ustd/build.sh
just uspace-std-test     # the host tests — no target, no nightly build-std
just uspace-std-check    # type-check the userspace target
```

The result is `ustd/target/x86_64-trangorge-uspace/{debug,release}/tgs-init`, a
static ELF with no interpreter. Copy it into the image as `/bin/init.elf`.

## The `target_os` decision, stated plainly

`targets/x86_64-trangorge-uspace.json` says `"os": "linux"`.

That is a deliberate lie, and it is the price of not forking rustc. Upstream `std`
has no `trangorge` platform selection: without a `sys/pal/trangorge` module in the
`std` sources, a target it does not recognise falls through to
`sys/pal/unsupported`, and `std::fs`, `std::net` and the rest become stubs that
panic. Declaring the OS as `linux` makes `std` select its unix path and resolve
`libc` to the real crate, so the only thing left to supply is the *definitions*
of the functions `libc` declares — which is all `tgs-libc` is.

* **Buys:** every `std` API, working, from crates.io, today.
* **Costs:** the compiled artifact reports `target_os = "linux"`, so a dependency
  doing `cfg(target_os = "linux")` takes the Linux branch. For a TrangorgeOS
  program that is usually right; where it is not, it is a `cfg` away.
* **The real fix** is a `std` fork with a `sys/pal/trangorge` module, which would
  make this directory mostly unnecessary. That is the right long-term move and it
  is a bigger project than this one.

## What works, and what does not

| `std` | Status | Needs from the kernel |
|-------|--------|----------------------|
| `Vec`, `String`, `Box`, `format!`, `println!` | **works** | `mmap` |
| `HashMap`, `HashSet` | **works** | `getrandom` (falls back to a TSC-seeded SplitMix64, which is not cryptographic) |
| `std::env::args` | **works** | nothing — `_start` falls back to a synthetic `argv` |
| `std::env::vars` | **works** if the kernel pushes a process image; otherwise empty | the stack layout in `SYSCALLS.md` |
| `std::time::Instant`, `SystemTime` | **works** | `clock_gettime` |
| `std::io::stdin/stdout/stderr`, `Read`, `Write` | **works** | `read`/`write` |
| `std::fs::read_to_string`, `metadata`, `read_dir` | **works** | `openat`, `newfstatat`, `getdents64` |
| `std::process::id`, `exit` | **works** | `getpid`, `exit_group` |
| `std::thread::spawn` | **refuses, loudly** (`EAGAIN`) | `fs_base` and a `clone` that maps a stack |
| `std::net` | `ENOSYS` | the `0x3000` block |
| `set_var`, `unsetenv` | **no-op, documented** | a private growable copy of the environment block |

The two refusals are the interesting ones. `std::thread::spawn` fails with
`EAGAIN` rather than through a stub that reports success, because a second task
with no `fs_base` would corrupt the allocator's free lists — a crash inside
`malloc` is much harder to diagnose than an error at the `spawn`. And `setenv` is
a no-op that says so in its own documentation, because the environment block is
part of the stack image the kernel built and appending to it would overwrite the
auxv that follows it.

## Known gaps

* **The heap does not coalesce.** Freed blocks go onto one of nine free lists and
  nothing merges adjacent blocks, so a program that churns through many differently
  sized allocations eventually fragments and an allocation fails. The right place
  to fix that is the kernel's page allocator; see
  [`tgs-hal/src/heap.rs`](hal/src/heap.rs).
* **`_Unwind_Backtrace` is unresolved.** With `panic = "abort"` the paths that use
  it are dead, but `std`'s backtrace code still references the symbols. Expect a
  link error naming them on the first real `build.sh` run, and expect the fix to
  be stubs in `tgs-libc` — `panic = "abort"` means they are never called.
* **No `set_tid_address` clear-thread pointer**, so a `thread_local!` with a
  destructor is never run. Fine while there is one task, wrong the moment there
  are two.
* **The `0x2000` block is not implemented by the kernel yet.** `SYSCALLS.md` is the
  contract; `hal/src/sysnum.rs` is the same table in Rust, with a test that
  asserts it does not collide with any number the kernel already dispatches.

## Reading order

1. [`SYSCALLS.md`](SYSCALLS.md) — what the kernel has to do, including the two
   changes it needs before any of this runs.
2. [`hal/src/heap.rs`](hal/src/heap.rs) — the allocator, and why its header sits
   *below* the payload.
3. [`hal/src/dirent.rs`](hal/src/dirent.rs) — `std::fs::read_dir` parses these
   bytes itself, so the layout is the contract.
4. [`rt/src/lib.rs`](rt/src/lib.rs) — `_start`, and the stack walk.
