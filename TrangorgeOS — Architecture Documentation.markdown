# TrangorgeOS — Architecture Documentation

> **Document status:** Working draft 0.2  
> **Scope:** System architecture, kernel layout, memory and allocator design, driver-space model, repository map, and current gaps  
> **Sources:** The supplied project tree, the `unstable` branch, and the official project website [1] [2]

## 1. Project overview

TrangorgeOS is an operating system built from scratch around a **hybrid kernel**. The project currently targets **x86_64**, with ARM64 and RISC-V listed as target architectures for the wider system [2]. The public repository describes the project as a hybrid-kernel operating system [1].

The project is intended for environments where control, low resource usage, predictable deployment, and process isolation are important. The official website lists kiosks and self-service terminals, databases and servers, fiscal/POS systems, and large-scale network infrastructure as target use cases [2].

The repository is intentionally split into a kernel, driver-space components, standalone drivers, libraries, boot components, build tools, and a custom-language toolchain. This document explains the responsibility of each area and records what is implemented, what is experimental, and what remains to be specified.

## 2. Current state

According to the project author, the kernel is substantially implemented, the filesystem works but still has issues, and development is moving toward driver space, libraries outside the kernel, and final kernel polishing. The supplied project description estimates approximately 22,000 lines in the kernel and approximately 8,000 lines related to the allocator.

The public `unstable` branch contains the main project components and is still actively changing. The repository page reports Rust as the dominant language, followed by Zig and a small remainder of other languages [1]. The website describes Rust as the kernel language and Ada/SPARK, C, and a custom language as part of the planned wider ecosystem [2]. Planned technologies should therefore not be documented as already-active kernel components without source-level confirmation.

| Area | Verified or stated status | Documentation note |
|---|---|---|
| Kernel model | Hybrid kernel | Publicly stated by the repository and website. |
| Target architectures | x86_64, ARM64, RISC-V | The target list is public; implementation maturity must be tracked per architecture. |
| Kernel language | Rust, with low-level and supporting components | GitHub currently reports Rust as dominant; the website lists other ecosystem languages as planned. |
| Filesystems | FAT32 and EXT4 are publicly listed; TFS is present in the tree | Native filesystem status and compatibility guarantees require further specification. |
| Allocator | Buddy allocator active; slab implementation present but disabled by default | `HEAP_USE_SLAB` is currently `0` in `heap.c` [5]. |
| Driver model | Kernel space, driver space, user space | Explicitly defined by the official website and partially implemented in source [2] [3]. |

## 3. High-level architecture

```text
+----------------------------------------------------------------+
| Applications, tools, and user-facing libraries                |
| trangorgelibc, triang-lang, ISO tools, auxiliary utilities     |
+------------------------------+---------------------------------+
| User space                   | User-facing APIs and clients   |
| peripheral drivers and apps  | syscall, ABI, I/O              |
+------------------------------+---------------------------------+
| Driver space                 | driverspacelib, runtime, ABI  |
| isolated driver environment  | shared rings, services         |
+------------------------------+---------------------------------+
| Kernel space                 | CPU, memory, FS, USB, network, |
| critical mechanisms          | PCI, interrupts, VGA          |
+----------------------------------------------------------------+
| Hardware and firmware       | x86_64 today; ARM64/RISC-V    |
+----------------------------------------------------------------+
```

The official website describes the project as having a three-way driver split [2]. The source code shows that driver space is more than a directory name: the kernel creates a separate address space, allocates two communication rings, prepares initialization parameters, and maps the shared objects into driver space [3].

## 4. The three driver spaces

### 4.1 Kernel space

Kernel-space drivers are the critical components that require maximum performance or privileged access. The website lists the network driver, allocator, memory manager, filesystem, USB driver, and VGA text mode among the components intended to remain in the kernel [2]. The kernel also contains PCI and USB infrastructure, interrupt handling, DMA-related facilities, block-device support, and the low-level glue required by other components.

Kernel space should provide stable primitives rather than contain every device-specific implementation. Its responsibilities include memory management, address-space management, interrupts, DMA and MMIO access, device discovery, block I/O services, and protection boundaries.

### 4.2 Driver space

Driver space is the isolated environment for drivers that should not be able to compromise the kernel when they fail. The kernel-side implementation prepares a separate `AddressSpace`, two shared ring buffers named `k2d` and `d2k`, and a `DsInitParams` page [3]. The rings are used for kernel-to-driver and driver-to-kernel messages.

The driver-space service layer currently handles commands related to logging, page allocation and freeing, MMIO mapping, device enumeration, block reads and writes, audio operations, and physical-page lookup [4]. The implementation therefore already contains a concrete kernel-to-driver service boundary, although the full ABI and lifecycle policy still need stabilization.

### 4.3 User space

User space contains applications, libraries, tools, and peripheral drivers that do not need direct access to the kernel’s most privileged mechanisms. `trangorgelibc` provides the visible foundation for client-side ABI, error types, syscall-related interfaces, I/O, memory, and synchronization. The planned package manager, code editor, compatibility layers, and custom-language ecosystem belong above the kernel boundary [2].

## 5. Driver-space communication

Driver-space communication uses two ring buffers. A message contains an identifier, command, flags, three arguments, and a status value. The command set includes initialization and lifecycle commands as well as services such as `Log`, `AllocPages`, `FreePages`, `MapMmio`, `GetDeviceCount`, `BlockRead`, `BlockWrite`, `AudioInfo`, and `PagePhys` [3] [4].

```text
Driver or user-facing component
            |
            v
     driverspacelib API
            |
            v
     d2k / k2d ring messages
            |
            v
 Kernel-side service dispatcher
            |
            +--> physical memory and page mapping
            +--> PCI / USB / device registry
            +--> MMIO and DMA permissions
            +--> block and audio services
```

The intended rule is that a driver should not duplicate kernel mechanisms, while the kernel should not need to know every device-specific detail. Driver space should implement device logic through a controlled interface, and the kernel should provide the privileged operations needed to make that logic useful.

## 6. Kernel subsystem map

| Subsystem | Main location | Responsibility |
|---|---|---|
| Entry and diagnostics | `kernel/src/main.rs`, `serial.rs`, `vga_buffer.rs`, `testing.rs` | Boot-time entry, diagnostics, test helpers, and early output. |
| ABI | `kernel/src/abi` | Allocation, memory, filesystem, and public API contracts. |
| CPU and interrupts | `cpu/`, `gdt.rs`, `interrupts.rs` | ACPI, LAPIC, GDT, trampolines, CPU state, and interrupt entry. |
| Memory | `mm/`, `allocator/` | Physical and virtual memory, paging, heap, cache, protection, DMA, and address spaces. |
| Filesystem | `fs/` | MBR, disk access, ATA PIO, block drivers, FAT32, EXT4, TFS, and initialization. |
| Hardware drivers | `drivers/`, `pci.rs` | PCI, USB, HID, mass storage, host controllers, and glue code. |
| Graphics and terminal | `gfx/`, `terminal/`, `vga_buffer.rs` | Framebuffer, VGA, fonts, console, terminal, and graphics support. |
| Networking | `nic/` | Ethernet, packets, protocols, device abstractions, and VirtIO networking. |
| Installation and updates | `ctrlinstall/` | Repositories, indexes, manifests, dependency resolution, transactions, and upgrades. |
| Driver-space initialization | `driverspaceinit/` | Address-space setup, shared memory, initialization, communication, and services. |

The memory subsystem bootstrap order is explicitly visible in `mm_init()`: `arch_memory_init` → `paging_init` → `pmm_init` → `vmm_init` → `page_init` → `mapping_init` → `heap_init` → `cache_init` → `paging_subsystem_init` → `isolation_init` → `aspace_subsystem_init` [7].

## 7. Memory and allocator architecture

The memory subsystem is divided into physical memory, virtual memory, heap allocation, caching, DMA/contiguous memory, protection, and process address spaces.

```text
Rust and C allocation APIs
            |
            v
Allocation API and FFI bridge
            |
            +--> physical memory: bitmap, frames, PMM
            +--> virtual memory: VMM, mapping, pages, paging, TLB
            +--> heap: buddy allocator, optional slab allocator
            +--> special memory: contiguous allocation and DMA
            +--> cache: object cache and per-CPU support
            +--> protection: guard, isolation, permissions
            +--> process spaces: address_space and mmap
```

### 7.1. Initialization

`mm_init()` validates boot memory-map parameters and initializes the memory stack in a deliberate order. Physical memory and virtual memory are prepared before the heap; the heap is initialized before cache and isolation subsystems; address-space support is initialized last [7]. This order should be treated as an architectural contract because driverspace and device mappings depend on it.

### 7.2. Active heap path

The active heap path uses the buddy allocator. `heap.c` reserves a 256 MiB buddy region and maps physical frames into it as needed. `HEAP_USE_SLAB` is set to `0`, so slab code exists but is not selected by the default build. If slab is enabled later, it is intended for small allocations; otherwise allocations are routed through buddy [5].

The buddy implementation tracks allocation orders, free lists, mapped pages, used/free statistics, and buddy coalescing on release. The public allocation API includes `kmalloc`, `kzalloc`, `kcalloc`, `krealloc`, aligned allocation, `kfree`, page allocation, and allocator diagnostics [5] [6].

### 7.3. Required allocator documentation

The next allocator specification should define ownership, alignment, blocking behavior, interrupt-context safety, out-of-memory behavior, DMA constraints, and the relationship between physical frames, virtual mappings, buddy blocks, and slab objects.

## 8. Repository map

| Directory | Role |
|---|---|
| `kernel/` | Main kernel and its memory, filesystem, CPU, networking, ABI, and hardware layers. |
| `drivers/` | Packages for concrete device drivers such as GPU, audio, camera, and Wacom. |
| `driverspace/` | Driver-space executable or runtime and its public API. |
| `driverspacelib/` | Shared driver-space ABI, runtime, logging, memory, ring, audio, and block helpers. |
| `trangorgelibc/` | Client-side library and syscall/ABI-facing facilities. |
| `comgrub/` | GRUB boot component. |
| `comlimine/` | Limine boot component. |
| `iso-builder/` | ISO-image construction tool. |
| `triang-lang/` | Lexer, parser, AST, semantic analysis, IR, and C/assembly code generation. |
| `mp4_to_bmp/` | Auxiliary media-conversion tool. |
| `targets/` | Build-target configuration. |
| Docker files and scripts | Reproducible build and run environment. |

## 9. Build and boot components

The repository contains both `comgrub` and `comlimine`, indicating two boot-component paths. The project also contains an ISO builder, target configuration for bare-metal x86_64, and Docker scripts for repeatable build and execution. These components should be documented together with the exact artifact names, bootloader assumptions, image layout, and emulator or hardware test commands.

## 10. Current risks and open questions

The driver-space mechanism is present, but its long-term ABI and lifecycle policy are not yet fully specified. The project should document how driver images are loaded, how they are stopped or restarted, how a failed driver is isolated, and which commands are stable across versions.

The public source also requires a careful compile check around `kernel/src/driverspaceinit/init/init.rs`: the visible code references scratch-page state in helper functions, while the displayed `Driverspace` structure does not visibly contain the corresponding field. This should be resolved before treating the driver-space initialization path as stable.

Architecture support should be tracked per target. Listing ARM64 and RISC-V on the website does not by itself prove feature parity with x86_64. Each architecture needs a target definition, boot path, page-table implementation, interrupt model, linker configuration, and CI/build validation.

## 11. Summary

TrangorgeOS has a clear architectural direction: a hybrid kernel with a three-way driver split, a layered memory subsystem, a dedicated driver-space communication boundary, and an ecosystem of libraries, tools, and a custom language. The most concrete implementation boundary today is the kernel-to-driver-space channel built from a separate address space, shared rings, initialization parameters, and kernel-mediated services.

The highest-value next steps are to stabilize the driver-space ABI, make the allocator contract explicit, repair and test the driver-space initialization path, establish repeatable boot/build tests, and track architecture parity rather than only architecture names.

## References

[1]: https://github.com/CTRL-F-0rg3/TrangorgeOS/tree/unstable "TrangorgeOS — unstable branch"

[2]: https://trangorgeos.website/ "Official TrangorgeOS website"

[3]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/driverspaceinit/init/init.rs "TrangorgeOS — driver-space preparation"

[4]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/driverspaceinit/init/service.rs "TrangorgeOS — driver-space services"

[5]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/mm/alloc/heap/heap.c "TrangorgeOS — heap buddy/slab routing"

[6]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/mm/alloc/api/alloc.c "TrangorgeOS — allocator API"

[7]: https://github.com/CTRL-F-0rg3/TrangorgeOS/blob/unstable/kernel/src/mm/core/mm.c "TrangorgeOS — memory subsystem initialization"
