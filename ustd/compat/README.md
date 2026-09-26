# Linux ABI compatibility for TrangorgeOS userspace

`ustd` gives userspace a real Rust `std`. This directory goes one step further: it
runs **unmodified Linux binaries** in ring 3 — the same `ld-linux-x86-64.so.2`
programs people already have, not just programs recompiled for TrangorgeOS.

```text
   /bin/ls  (PIE, PT_INTERP = /lib64/ld-linux-x86-64.so.2)
        │
        ▼
   tgs-loader ── our ld.so: maps segments, resolves symbols, relocates
        │
        ├── tgs-elf    ── ELF64 parsing: Ehdr, Phdr, DYNAMIC, SYMTAB, RELA
        ├── tgs-libc   ── the glibc ABI (already exists in ustd/libc)
        ├── tgs-vdso   ── the vDSO, so glibc stops using syscalls for clocks
        │
        ▼
   int 0x80 → kernel
```

## What has to be true, in order

The order is not a preference. Each step is a prerequisite for the next, and
skipping one produces a failure that looks like the one after it.

### 1. The ELF itself is not the hard part

A PIE executable and a shared object are the same format, and `tgs-elf` parses
both. The parsing is fully unit-tested against `/bin/ls` and the host's real
`libc.so.6` — real stripped binaries, not fixtures.

What *is* hard is the four things around the format:

| Problem | Why it bites | Where it lands |
|---------|--------------|----------------|
| **Relocation** | A PIE has no absolute addresses until the loader writes them. A missed `R_X86_64_RELATIVE` and the program jumps into page zero. | `tgs-loader` |
| **Threads** | glibc's `pthread_create` is `clone` plus a `fs_base` write. With no `fs_base` in the kernel, any threaded program dies. | kernel + `tgs-hal` |
| **The vDSO** | glibc reads the clock through it and *validates* it against `AT_SYSINFO_EHDR` and `AT_PAGESZ`. A wrong answer is a `SIGSEGV` during `main`'s first line. | `tgs-vdso` |
| **Signals and unwinding** | glibc's own code uses `setjmp`/`longjmap` and `_Unwind_*` before `main` is entered. A missing symbol is a link failure, not a runtime one. | `tgs-libc` |

### 2. glibc's start-up path, in order

```
_start                        (us: tgs-rt)
  → read auxv, set up TLS      (us: AT_PHDR, AT_PAGESZ, fs_base)
  → __libc_start_main         (us: tgs-libc)
      → __libc_init_secure
      → _dl_relocate_static_pie / _dl_start
          → tgs-loader: map PT_LOAD, walk DT_NEEDED, relocate
      → __libc_setup_tls
      → __libc_init_first
      → __libc_csu_init       (DT_INIT, DT_INIT_ARRAY, DT_PREINIT_ARRAY)
      → main(argc, argv, envp)
```

Every one of those symbols has to *exist*, and several of them have to do real
work. A stub that returns 0 gets past the linker and then fails mysteriously
inside glibc's initialisation, which is the worst possible failure mode.

### 3. The vDSO contract

glibc does not call `clock_gettime` through a syscall if a valid vDSO is present,
and it decides "valid" by checking that `AT_SYSINFO_EHDR` points at a `PT_DYNAMIC`
whose `DT_HASH` and symbol table it can read, and that `AT_PAGESZ` matches the
mapping the vDSO landed on.

The vDSO is a shared object built with a linker script that places it in its own
page, exporting:

| Symbol | Kind | Notes |
|--------|------|-------|
| `__vdso_clock_gettime` | real | calls the kernel through `__kernel_vsyscall` |
| `__vdso_gettimeofday` | real | same |
| `__vdso_time` | real | same |
| `__vdso_getcpu` | real | reads `RDPGSR`/GS on x86_64 |
| `__vdso_clock_getres` | real | constant |
| `__vdso_gettimeofday` etc. | aliases | glibc looks these up by name |

`__kernel_vsyscall` is the mechanism, and it is a *syscall*: there is no
`syscall` instruction on x86_64 that bypasses the gate, so it is
`int 0x80` with a flag that says "do not consult the vDSO". That flag is the
whole reason the vDSO needs one: without it the vDSO's own `clock_gettime` would
recurse into the loader's copy.

## What exists today

| Piece | State |
|-------|-------|
| `tgs-elf` — ELF64 parsing | **done**, host-tested against `/bin/ls` and `libc.so.6` |
| `tgs-loader` — segment mapping, `DT_*` walk, symbol resolution, relocation | not started |
| The `0x2000` syscall block (`SYSCALLS.md`) | numbers reserved, kernel unimplemented |
| `tgs-libc` — the glibc ABI, static part | `malloc` family, `stdin`/`stdout`, `open`/`read`/`write`, `clock_gettime`, `getrandom`, `exit_group`, `__errno_location`, `__cxa_*` |
| `tgs-rt` — `_start`, auxv | exists, but does not yet build a process image |
| Threads | **blocked on the kernel**: no `fs_base` |
| vDSO | not started |
| `dlopen`/`dlsym` | not started |

### `-z relr`: the thing that would have bitten us

The first version of the relocation test asserted the textbook layout: every
`DT_JMPREL` entry is an `R_X86_64_JUMP_SLOT`. **It failed on this host's
`/bin/ls`**, which has no `DT_JMPREL` at all. The real tag list is:

```
Needed=0x595 Init=0x3000 Fini=0x1cb94 InitArray=0x26d10 GnuHash=0x3c8
StrTab=0x10c8 SymTab=0x468 Rela=0x18d8 RelaSz=0xb70 Flags=0x8 Flags1=0x8000001
VerNeed=0x1808 VerSym=0x16fe Relr=0x58 RelrSz=0x2448 RelrEnt=0x8
```

Two things follow. The binary is linked **`-z now`** (`DF_1_PIE` in `DT_FLAGS_1`),
so there is no lazy PLT binding at all. And its `R_X86_64_RELATIVE` relocations
are **run-length encoded in `DT_RELR`**, which arrived in glibc 2.36.

A loader written against the textbook description would parse this binary without
complaint, find no PLT relocations, relocate nothing, and crash on the first
access to a global — with nothing in the parser to point at the cause. So the
`RELR` tags are named in `Tag` rather than left as `Other`, and a decoder exists
in `reloc.rs`.

**The decoder is not finished, and its documentation says so.** Only two RELR
properties are confirmed by tests: the base is folded into every emitted address,
and an all-zero bitmap is not mistaken for an address.

Getting there took two failed versions of the test, both worth recording. The
first assumed the textbook `DT_JMPREL` layout and failed, because a current
`/bin/ls` is linked `-z now` and has none. The second asserted a hardcoded
address list for the first RELR group — written from memory rather than read out
of the diagnostic — and failed too, revealing that the first entry of a real
table does **not** carry bit 63, contrary to the specification's wording. The
fabricated constants were deleted rather than nudged until they passed. Whether
a whole table decodes correctly is still unknown, because `DT_RELRSZ` runs past
the end of `.rela.dyn` on this host; dump a real table with the `#[ignore]`d
diagnostic before relying on `relr_decode`.

## The honest cost

This is a large project, and the ordering above is the reason: it is not one
linear task but six, each of which can fail independently. The first two are
here. The rest are written up — in each crate's module docs and in
`LINUX_ABI.md` — rather than half-implemented, because a half-implemented loader
is indistinguishable from a broken one at the only moment it matters.

**The single biggest blocker is not userspace at all**: the kernel has to program
`fs_base` on a ring-3 context switch and implement `clone`. Everything about
threads — `pthread_create`, `thread_local!` destructors, `std::thread` in a
loaded binary, and glibc's own locking — needs those two, and no amount of
userspace code substitutes for them.

__APPEND_ABI__
