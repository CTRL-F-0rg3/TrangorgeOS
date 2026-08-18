# TrangorgeOS — TODO

> **Purpose:** This file tracks the remaining work required to move TrangorgeOS from the current `unstable` development state toward a reliable, testable, and eventually stable system.
>
> **Status convention:** `TODO` means not started or not verified; `WIP` means partially implemented; `BLOCKED` means dependent on another task; `DONE` should only be used after the stated acceptance criteria have been verified.
>
> **Priority convention:** `P0` blocks reliable boot or basic correctness; `P1` is required for the next serious development milestone; `P2` improves robustness and maintainability; `P3` is ecosystem or long-term work.

## 0. Current snapshot

| Area | Current state | Priority |
|---|---|---|
| Kernel boot and x86_64 baseline | Present, but requires repeatable build and boot verification | P0 |
| Memory subsystem | Layered implementation exists; initialization order is defined in `mm_init()` | P0 |
| Heap allocator | Buddy path is active; slab implementation exists but `HEAP_USE_SLAB` is `0` | P1 |
| Driver space | Separate address space, shared rings, initialization parameters, and services are present | P0 |
| Driver-space ABI | Implemented enough for experiments, not yet documented as stable | P0 |
| Filesystem | FAT32/EXT4 and additional filesystem code are present; correctness and integration tests are needed | P1 |
| USB/PCI | Significant infrastructure exists; device coverage and hardware validation are incomplete | P1 |
| Networking | NIC/VirtIO structures exist; network-driver work is explicitly a project priority | P1 |
| ARM64 and RISC-V | Public target goals; feature parity and build/boot status must be established | P2 |
| Package manager | Kernel-side `ctrlinstall` modules exist; end-to-end package workflow needs completion | P2 |
| Toolchain and ecosystem | `triang-lang`, libraries, ISO tooling, and auxiliary tools exist at different maturity levels | P2/P3 |
| Release process | No published release; stable branch criteria are not yet formalized | P1 |

## 1. P0 — immediate correctness blockers

### 1.1 Repair and compile-check driver-space initialization

**Status:** TODO  
**Area:** `kernel/src/driverspaceinit/init/init.rs`

The visible implementation creates a `Driverspace` structure with an address space, two ring physical addresses, an initialization-parameter page, and a preparation flag. Later helper functions reference scratch-page state that is not visibly represented in the structure. Resolve this inconsistency rather than masking it with an unrelated workaround.

**Tasks:**

- Add or remove scratch-page state consistently in the `Driverspace` structure and initialization path.
- Ensure the scratch page is allocated, zeroed, mapped, and released exactly once.
- Ensure `scratch_view()` cannot return a pointer after the driver-space instance has been destroyed.
- Ensure every failure after partial allocation releases previously allocated frames and address-space mappings.
- Compile the kernel after the fix with the repository’s documented target.

**Acceptance criteria:** The driver-space initialization module compiles without undefined fields or unreachable top-level statements; all allocation failures clean up; a self-test completes successfully.

### 1.2 Establish a reproducible baseline build

**Status:** TODO  
**Area:** root Docker scripts, `kernel/`, `targets/`

Document and verify the one canonical build command. The repository contains Docker scripts, target configuration, boot components, and an ISO builder, but the expected artifact and exact run command must be explicit.

**Tasks:**

- Verify the required Rust toolchain and external binaries.
- Run the Docker build from a clean environment.
- Record the produced kernel, image, and ISO paths.
- Run the image in an emulator and capture serial output.
- Add a short “Build and run” section to the root `README.md`.
- Make the build fail on warnings or errors that affect the boot artifact, where practical.

**Acceptance criteria:** A new contributor can clone the repository, run the documented commands, produce the same boot artifact, and reach the kernel’s expected boot milestone.

### 1.3 Define the driver-space ABI versioning rules

**Status:** TODO  
**Area:** `kernel/src/driverspaceinit/abi/`, `driverspacelib/src/`

The protocol already contains `DsMsg`, ring structures, magic/version fields, and command identifiers. It needs a written compatibility contract before more drivers depend on it.

**Tasks:**

- Document the binary layout, field widths, alignment, endianness, and reserved fields of `DsMsg` and ring metadata.
- Define the meaning of every command and every argument.
- Define error/status codes and whether negative values are stable ABI values.
- Define how unknown commands, malformed messages, full rings, and stale IDs are handled.
- Define the compatibility policy for `DS_MAGIC`, `DS_VERSION`, and command additions.
- Add compile-time size/alignment assertions on both kernel and driver-space sides.
- Add a protocol version document and a small ABI conformance test.

**Acceptance criteria:** The same ABI document can be used to implement a driver without reading kernel internals, and the conformance test passes on both sides of the boundary.

### 1.4 Make driver-space isolation explicit and testable

**Status:** TODO  
**Area:** `kernel/src/driverspaceinit/`, `kernel/src/mm/protection/`, `kernel/src/mm/process/`

The kernel creates a separate address space and maps shared pages into it. The security and failure-isolation properties need tests rather than only implementation intent.

**Tasks:**

- Test that driver space cannot read arbitrary kernel virtual addresses.
- Test that driver-space mappings are limited to explicitly granted RAM and MMIO regions.
- Validate permissions for shared rings, initialization parameters, RAM grants, and MMIO grants.
- Reject overlapping, unaligned, out-of-range, and double-freed grants.
- Add a driver crash or invalid-command test and verify that kernel execution continues.
- Define whether and how a failed driver can be restarted or detached.

**Acceptance criteria:** Negative tests demonstrate that unauthorized mappings and malformed service requests are rejected without corrupting kernel state.

## 2. P1 — kernel and hardware reliability

### 2.1 Complete the allocator specification and test suite

**Status:** WIP  
**Area:** `kernel/src/mm/alloc/`

The active heap path uses buddy allocation. Slab code is present but disabled with `HEAP_USE_SLAB 0`. The allocator needs an explicit contract and stress tests before it is treated as a kernel foundation.

**Tasks:**

- Document physical-frame, virtual-mapping, heap, buddy, slab, DMA, and contiguous-memory responsibilities.
- Add tests for zero-size allocations, alignment, large allocations, `krealloc`, double free, invalid free, and out-of-memory behavior.
- Test buddy split and coalescing across every supported order.
- Test page mapping rollback when a physical-frame allocation fails partway through.
- Verify `heap_usable_size()` for active and invalid pointers.
- Decide whether slab should remain disabled, be enabled for small objects, or be redesigned.
- Add allocator statistics and leak checks to a repeatable debug test.
- Define which allocator functions are safe during interrupt context and early boot.

**Acceptance criteria:** The allocator passes deterministic unit/stress tests and its public contract describes ownership, alignment, blocking, context restrictions, and failure behavior.

### 2.2 Finish filesystem correctness and integration tests

**Status:** WIP  
**Area:** `kernel/src/fs/`

The tree contains MBR, disk, ATA PIO, block-driver, FAT32, EXT4, TFS, and filesystem initialization code. The project needs tests that verify data integrity across the complete stack.

**Tasks:**

- Define the supported FAT32 and EXT4 feature subset.
- Test mounting, directory creation, file creation, reads, writes, truncation, append, rename, delete, and remount persistence.
- Test unaligned reads/writes and operations spanning multiple blocks.
- Test corrupted superblocks, invalid MBR data, short reads, failed writes, and unexpected device removal.
- Define and test filesystem locking and concurrent access rules.
- Separate experimental TFS behavior from the stable filesystem API.
- Add image-based regression fixtures for each supported filesystem.

**Acceptance criteria:** Filesystem tests pass on generated disk images and data remains correct after unmount/remount cycles.

### 2.3 Stabilize PCI and USB discovery

**Status:** WIP  
**Area:** `kernel/src/drivers/`, `kernel/src/pci.rs`

USB core, EHCI/XHCI, HID, CDC, and mass-storage structures exist. The next step is reliable enumeration, error handling, and ownership of device resources.

**Tasks:**

- Document PCI configuration access and BAR discovery.
- Verify MMIO BAR validation and alignment.
- Complete USB device, configuration, interface, endpoint, and speed handling.
- Add enumeration tests with recorded or emulated descriptor sets.
- Test device removal, reset, stalled endpoints, transfer timeouts, and controller errors.
- Define which USB classes remain in kernel space and which move to driver space.
- Connect mass-storage discovery to the block-device registry and filesystem layer.

**Acceptance criteria:** A supported USB storage device and a supported HID device can be enumerated repeatedly without kernel instability.

### 2.4 Build a functional network-driver milestone

**Status:** WIP  
**Area:** `kernel/src/nic/`

The website identifies network-driver development as an important contribution area. The repository contains NIC abstractions, Ethernet, packets, protocols, and VirtIO queues, but a complete validated network path is still required.

**Tasks:**

- Define the minimum supported NIC and VirtIO feature set.
- Complete RX/TX queue setup, descriptor ownership, interrupt or polling behavior, and recovery.
- Add packet-buffer ownership and lifetime rules.
- Implement and test a minimal Ethernet/ARP/IPv4 path, or explicitly document the current scope.
- Add checksum, MTU, malformed-packet, and queue-exhaustion tests.
- Add an emulator-based network test with captured packets.
- Define how network drivers are exposed to driver space.

**Acceptance criteria:** The selected NIC can transmit and receive validated packets in an emulator or test harness without memory leaks or queue corruption.

### 2.5 Define and test audio driver integration

**Status:** WIP  
**Area:** `kernel/src/audio/`, `drivers/audiodriver/`, `driverspacelib/src/audio.rs`

Audio code spans kernel services, driver-space helpers, JACK-related bindings, and driver code. The ownership and trust boundary must be made explicit.

**Tasks:**

- Define which audio operations are kernel services and which belong to the audio driver.
- Document buffer ownership, physical-page grants, sample format, rate, channel count, and duration units.
- Test `AudioInfo`, `AudioPlay`, `AudioStop`, and amplifier controls with invalid buffers and lengths.
- Verify that audio buffers cannot be used after revocation.
- Decide whether JACK integration is a development dependency or part of the target runtime.
- Add a minimal deterministic tone test in an emulator or supported host environment.

**Acceptance criteria:** Audio requests use a documented ABI, invalid grants are rejected, and the driver can start/stop playback without corrupting shared memory.

### 2.6 Formalize bootloader and ISO output

**Status:** TODO  
**Area:** `comgrub/`, `comlimine/`, `iso-builder/`

The project contains GRUB and Limine-related boot components and a separate ISO builder. Their supported roles and artifact formats need to be clear.

**Tasks:**

- Choose the primary supported boot path for the next milestone.
- Document the difference between GRUB and Limine images.
- Verify linker scripts, kernel load addresses, entry points, memory-map handoff, and initrd placement.
- Add ISO-content validation before boot.
- Add emulator smoke tests for each supported boot path.
- Record serial and graphical output expected at each milestone.

**Acceptance criteria:** The canonical ISO boots through the documented bootloader and fails with a useful diagnostic if a required artifact is missing.

## 3. P1 — code quality and integration

### 3.1 Define the kernel ABI boundary

**Status:** TODO  
**Area:** `kernel/src/abi/`, `trangorgelibc/src/abi/`

The kernel ABI and client-side library must agree on types, error values, syscall numbers, pointer rules, and versioning.

**Tasks:**

- Inventory every public ABI type and function.
- Define integer widths, structure layout, alignment, ownership, and lifetime rules.
- Define pointer validation and user/kernel buffer-copy rules.
- Synchronize error handling between kernel and `trangorgelibc`.
- Add generated or checked bindings where possible.
- Add ABI compatibility tests.

**Acceptance criteria:** A client library can be rebuilt against the documented ABI and pass layout and behavior tests.

### 3.2 Define driver lifecycle and registration

**Status:** TODO  
**Area:** `driverspace/src/`, `driverspacelib/src/driver.rs`, `kernel/src/driverspaceinit/`

The project needs one lifecycle model for discovery, initialization, registration, attachment, running, detachment, shutdown, and failure recovery.

**Tasks:**

- Define driver states and allowed transitions.
- Define device identity, vendor/class fields, capabilities, and ownership.
- Complete driver registration and device attachment commands or remove unused commands from the public ABI.
- Define resource acquisition and release for IRQs, MMIO, DMA, and shared memory.
- Define shutdown ordering and timeout handling.
- Add a mock driver used in CI to exercise the complete lifecycle.

**Acceptance criteria:** A mock driver can register, attach to a device, request resources, handle a request, detach, and shut down cleanly.

### 3.3 Reduce duplicated or experimental paths

**Status:** TODO  
**Area:** whole repository

Several components appear to contain experimental or partially connected paths. These should be clearly labeled or removed before a stable branch is created.

**Tasks:**

- Mark unused driver initialization helpers and experimental commands.
- Remove stale generated output from source-controlled directories where appropriate.
- Document intentionally empty or placeholder modules.
- Run formatting and static checks for Rust, C, Zig, and assembly sources where tools are available.
- Remove IDE metadata from production source directories unless intentionally retained.
- Record known warnings and decide whether each is acceptable or must be fixed.

**Acceptance criteria:** The repository has a documented list of experimental components, no unexplained generated artifacts, and a clean baseline check command.

## 4. P2 — architecture portability

### 4.1 Separate architecture-independent and architecture-specific code

**Status:** TODO  
**Area:** `kernel/src/mm/arch/x86_64/`, CPU, paging, boot, targets

The current memory code visibly contains x86_64-specific implementations. Portability requires clear interfaces before ARM64 or RISC-V work expands.

**Tasks:**

- Define architecture-independent interfaces for page tables, TLB operations, interrupts, timers, CPU startup, and memory barriers.
- Move x86_64-only constants and operations behind architecture modules.
- Define target-specific linker scripts and boot parameters.
- Add architecture capability tables for page size, address width, cache behavior, and DMA constraints.
- Add compile-only CI checks for ARM64 and RISC-V targets.

**Acceptance criteria:** Architecture-independent kernel code compiles without importing x86_64 implementation details, and each announced target has a reproducible compile target.

### 4.2 ARM64 milestone

**Status:** BLOCKED by architecture abstraction work  

- Define the ARM64 target specification.
- Implement or stub the boot entry and exception vector.
- Implement page-table and TLB operations.
- Implement interrupt-controller integration.
- Port the memory bootstrap and serial diagnostics.
- Boot a minimal ARM64 image in an emulator.

**Acceptance criteria:** A documented ARM64 smoke test reaches kernel initialization and reports memory-subsystem status.

### 4.3 RISC-V milestone

**Status:** BLOCKED by architecture abstraction work  

- Define the RISC-V target specification.
- Implement boot entry, trap handling, paging mode, and timer setup.
- Port serial diagnostics and memory initialization.
- Boot a minimal RISC-V image in an emulator.

**Acceptance criteria:** A documented RISC-V smoke test reaches kernel initialization and reports memory-subsystem status.

## 5. P2 — package manager and user ecosystem

### 5.1 Complete `ctrlinstall`

**Status:** WIP  
**Area:** `kernel/src/ctrlinstall/`

The repository contains repository fetching, indexes, manifests, dependency resolution, transactions, diffs, and upgrades. The complete user-facing workflow still needs to be defined and tested.

**Tasks:**

- Define repository and package formats.
- Define signatures, trust roots, hashes, and rollback behavior.
- Complete dependency resolution and conflict diagnostics.
- Test interrupted transactions and power-loss recovery.
- Add install, remove, update, search, and list interfaces.
- Document how driver packages and kernel-compatible packages are separated.

**Acceptance criteria:** A package can be fetched, verified, installed, listed, upgraded, rolled back, and removed from a test repository.

### 5.2 Stabilize `trangorgelibc`

**Status:** TODO  

- Inventory implemented versus placeholder modules.
- Define syscall wrappers and error semantics.
- Add memory, I/O, synchronization, and ABI tests.
- Document which interfaces are safe for applications and which are experimental.

### 5.3 Advance `triang-lang`

**Status:** WIP  

- Document the language syntax and project format.
- Define the AST and IR stability policy.
- Add parser and semantic-analysis tests.
- Add deterministic code-generation tests for C and assembly.
- Define the x86_64 output ABI.
- Decide which language features are required for system tools.

## 6. P3 — tools, documentation, and community

### 6.1 Documentation structure

**Status:** WIP

- Move the architecture documents into a stable `docs/` directory.
- Add a contributor guide and development-environment guide.
- Add a boot-flow diagram and driver-space sequence diagram.
- Add per-subsystem pages for memory, filesystem, USB, network, ABI, and package management.
- Keep English as the reference language or define an explicit bilingual-document policy.
- Add a changelog policy for ABI and boot-image changes.

### 6.2 Code editor and developer tools

**Status:** TODO

- Define the dedicated editor’s scope and minimum viable feature set.
- Decide whether the editor is part of the OS image or a host-side development tool.
- Document project files, build integration, syntax support, and debugger/serial integration.
- Connect editor workflows to `triang-lang` and package-management tools.

### 6.3 Contribution and release process

**Status:** TODO

- Define branch policy for `main`, `unstable`, and the future stable branch.
- Define release criteria and a versioning scheme.
- Add CI for formatting, compilation, ABI tests, allocator tests, filesystem-image tests, and emulator boot tests.
- Add issue templates for kernel bugs, driver bugs, build failures, and documentation gaps.
- Document the AGPLv3 licensing requirements and contribution expectations.

## 7. Suggested milestone order

The recommended order is intentionally dependency-driven:

1. Repair and compile-check driver-space initialization.
2. Establish a reproducible x86_64 build, ISO, and emulator boot test.
3. Freeze and test the driver-space ABI at an experimental version.
4. Add isolation and invalid-request tests.
5. Finish allocator, filesystem, USB, and network regression suites.
6. Define driver lifecycle and complete a mock driver.
7. Stabilize the kernel ABI and `trangorgelibc`.
8. Define architecture abstractions and add compile-only ARM64/RISC-V targets.
9. Complete package-manager and custom-language workflows.
10. Define stable-branch and release criteria.

## 8. Definition of “ready for stable”

The kernel should not move to a stable branch merely because it boots once. A stable milestone should require a reproducible build, a documented boot artifact, passing allocator and filesystem regression tests, a versioned driver-space ABI, negative isolation tests, at least one validated storage path, one validated input path, one validated network path, and a known list of unsupported hardware.

The branch should also have a rollback or recovery story for failed package transactions and driver failures. Every public architecture claim should distinguish between “compiles,” “boots,” “has tested core services,” and “has feature parity.”

## 9. Source references

[1]: https://github.com/CTRL-F-0rg3/TrangorgeOS/tree/unstable "TrangorgeOS — unstable branch"

[2]: https://trangorgeos.website/ "Official TrangorgeOS website"

[3]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/driverspaceinit/init/init.rs "TrangorgeOS — driver-space preparation"

[4]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/driverspaceinit/init/service.rs "TrangorgeOS — driver-space services"

[5]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/mm/alloc/heap/heap.c "TrangorgeOS — heap buddy/slab routing"

[6]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/mm/alloc/api/alloc.c "TrangorgeOS — allocator API"

[7]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/mm/core/mm.c "TrangorgeOS — memory subsystem initialization"
