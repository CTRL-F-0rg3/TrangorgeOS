# TrangorgeOS

# TrangorgeOS

> A modern, high-performance bare-metal operating system built ground-up on a custom **Separated Tri-Partition Architecture** (*Architektura Trójpodziału Rozdzielnego*).

---

## Overview & Philosophy

TrangorgeOS is built with a strict **from-scratch philosophy** (~35,000+ lines of code, actively expanding). It discards classic microkernel overhead and hybrid kernel bloat in favor of a strictly isolated, policy-enforced driver architecture. 

The project operates under an intensive development cycle aimed at producing a stable, bare-metal kernel fully functional on physical hardware.

---

## Visuals & Screenshots

| Kernel Loading & Memory Allocation Testing | Kernel Base Resolution | Kernel in 1080p |
|:---:|:---:|:---:|
| ![Kernel Loading](kernelloading.png) | ![Kernel Base Res](kernel_loader_base_res.png) | ![Kernel 1080p](kernelin1920x1080.png) |

---

## Architectural Model: Separated Tri-Partition Architecture

TrangorgeOS decouples driver functionality and system privilege into 4 distinct operational layers (3 main domains + userspace abstraction):


```

+-----------------------------------------------------------------+
|                         USERSPACE                               |
|   - Applications & High-Level Libraries                         |
+-----------------------------------------------------------------+
|                   USER DRIVER SPACE (UDS)                       |
|   - High-risk / Peripherals (Fault-isolated, strict boundary)   |
+-----------------------------------------------------------------+
|                     DRIVER SPACE (DS)                           |
|   * Dynamic Drivers   : GPU & complex hardware                  |
|   * Static Drivers    : Init & single-action hardware setup       |
|   * Library Drivers   : Inter-driver interface providers        |
+-----------------------------------------------------------------+
|                    KERNEL CORE / DRIVERS                        |
|   - Trusted Core Drivers (Network, USB stack base, Core MM)     |
|   - Fine-grained Kernel Policy Enforcement & Data Flow Control  |
+-----------------------------------------------------------------+

```

1. **Kernel Core & Trusted Drivers:** Holds only maximum-trust drivers (e.g., base network, USB stack core) to eliminate IPC latency for essential paths without compromising core stability. Exports explicit system interfaces.
2. **Driver Space (DS):** Modular driver execution environment with strict error margins:
   - **Dynamic Drivers:** Handle complex, stateful hardware (e.g., GPU control).
   - **Static Drivers:** Perform hardware initialization or non-exporting, single-purpose setup.
   - **Library Drivers:** Expose specialized interfaces (e.g., PCI, HDMI control) for other drivers to consume.
3. **User Driver Space (UDS):** High-level peripheral drivers isolated at a safe distance from the core to prevent system crashes on fault.
4. **Userspace & User Library Drivers:** Top-level space housing application runtime and user-level library interfaces.

---

## Current Subsystems & Refactoring Status

> **Notice:** The kernel is currently undergoing a major refactoring phase (modernization and safety audit) across multiple subsystems.

- **Memory Management (MM):** Fully custom memory management subsystem and dynamic allocator written in C (~9,000 LOC currently, expanding to ~16,000 LOC during refactoring to eliminate security/safety bugs).
- **Network & USB:** Network driver base operational; USB stack operating under a controlled bare-metal environment.
- **Input & Display:** Built-in BIOS PS/2 fallback support; active development on modern USB input. Kernel features an in-kernel text editor, serial UART output (COM1), and an automated internal system tester executing real-time sanity checks.
- **Multiprocessing & Interrupts:** SMP multi-core initialization active. APIC/IDT interrupt architecture and process scheduler are actively being overhauled.
- **Binary Support:** Native execution support for `ELF` and custom `.bin` binaries with custom IPC interfaces.

---

## Custom Versioning System

TrangorgeOS uses an in-house versioning scheme reflecting the precise state of development:

$$\text{Major} . \text{Changes} . \text{FixesPerChange} . \text{Iteration}\text{State}$$

*Example: `v0.182.7.1a`*
- **`0` (Major):** Pre-release major build.
- **`182` (Changes):** Total cumulative feature changes introduced.
- **`7` (Fixes):** Bugfixes applied for the current change.
- **`1` (Iteration):** Single target architecture fully active.
- **`a` (State):** Alpha phase (`a` = initial work $\rightarrow$ `b` = stabilization $\rightarrow$ `g` = pre-release gamma).

### Scale Milestones:
- **Alpha:** Pre-release scale (up to ~60,000 LOC), active architectural building.
- **Beta:** System runs stably on physical hardware.
- **Gamma:** Final stabilization phase prior to Release `v1.0`.

---

## Git Branch Model & Workflow

The repository relies on a strict multi-tier branch hierarchy:

| Branch | Purpose |
|:---|:---|
| `init` | Conceptual structure, architecture blueprints, no active codebase. |
| `new` | Experimental features and isolated proof-of-concept tests. |
| `unstable` | Primary active development branch. |
| `stabilizing` | Refactoring, bug-hunting, and code modernizing before release. |
| `stable` | Tested, incremental updates with guaranteed stability. |
| `main` | Production showcase branch. |

---

## Build

The project builds as two Cargo workspaces:

- **`kernel_Workspace/`** — the kernel and its bootable image (`kernel-bin`).
- **`kernel-drivers/`** — the exported hardware-driver libraries (`pcie`, `usb`, `audio`, …), extracted from the legacy kernel as standalone `no_std` crates. The network driver (`nic`) is intentionally **not** exported.

Build orchestration is handled with [Just](https://github.com/casey/just) (`justfile`):

```sh
just            # build everything (driver libraries -> kernel bootimage)
just drivers    # build the driver libraries only
just kernel     # build the kernel bootimage
just run        # build + run the kernel in QEMU
just check      # cargo check (no linking)
just iso        # build a bootable demo .iso (see "Demo ISO" below)
just iso-run    # boot the newest demo .iso in QEMU
```

> **Note:** the experimental `ctrlfile` build tool was **excluded from the repository** for now — it is not mature enough yet — and has been replaced by the `justfile` above.

### Demo ISO

A bootable, publishable demo image is produced from the exact same kernel build:

```sh
just iso        # -> dist/TrangorgeOS-<version>-x86_64-demo.iso
just iso-run    # boot the newest ISO in QEMU (window + serial on stdout)
just iso-test   # headless ISO smoke test (serial log + PASS/FAIL report)
```

The ISO boots via El Torito in *no-emulation* mode: **ISOLINUX** starts, and
**MEMDISK** then boots the unmodified `bootimage-kernel-bin.bin` from RAM. This
is currently the only supported path: `bootloader 0.9` produces a raw
512-byte-sector BIOS disk image, while the `comgrub` (multiboot2) and
`comlimine` (Limine) entry points are still stubs — they do not export the
`kernel_main` ABI the kernel expects (see `TrangorgeOS — TODO.md`, section 2.6).

Extra packages required (on top of the toolchain above):

```sh
sudo apt-get install -y xorriso syslinux-common    # Debian/Ubuntu
```

`tools/mkiso.sh` also writes the releases artefacts next to the ISO:
`*.iso.sha256` and `dist/RELEASE.txt` (version, git commit, kernel hash,
toolchain). The image carries a hybrid MBR, so the very same file can be `dd`-ed
to a USB stick and booted in legacy/CSM mode.

> **Current limitation:** legacy BIOS only — there is no UEFI hand-off yet.
> Full recipe, publication checklist and limitations: [`tools/iso/README.md`](tools/iso/README.md).
> Verified on 2026-09-25: the demo ISO boots in QEMU to the in-kernel terminal
> with `SYSTEM STATUS: 19/19 OK`.

---

## Project Roadmap


- [x] **Current Stage (`stabilizing` / Alpha v0.182.x):** Deep refactoring of MM allocator, scheduler modernization, bug localization.
- [ ] **Milestone 1 (~1.5 Months):** Complete memory allocator refactoring, lock down current subsystem rewrite, merge to `stable`.
- [ ] **Milestone 2 (~4 Months):** Mature active drivers, write dedicated GPU/display drivers, initiate multi-architecture porting.
- [ ] **Milestone 3 (~1 Year):** Complete full bare-metal testing on physical hardware, establish stable kernel runtime, expand userspace tooling.

---

<p align="center">
  <i>TrangorgeOS — Building bare-metal systems from scratch, one commit at a time.</i>
</p>
