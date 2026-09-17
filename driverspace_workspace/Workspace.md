Oto profesjonalna dokumentacja architektury, przygotowana w języku angielskim, zgodnie z Twoimi wytycznymi.

***

# TrangorgeOS Driver Space Architecture and Workspace Documentation

## 1. Executive Summary
This document outlines the architectural redesign of the Driver Space subsystem within TrangorgeOS. The objective is to transition from a monolithic implementation (`driverspacelib` and `driverspace`) to a highly modular, strictly layered workspace. This restructuring enforces clear boundaries between the kernel interface, driver management daemons, and individual hardware drivers, ensuring maintainability, preventing code duplication, and establishing a robust foundation for the new Kernel API.

## 2. Workspace Topology
The `driverspace_workspace` is divided into three primary pillars. This strict separation of concerns dictates the dependency graph and ensures that hardware-specific logic remains isolated from system management and core abstractions.

```text
driverspace_workspace/
├── Cargo.toml                 # Workspace root configuration
├── lib/                       # Core libraries, frameworks, and the new Kernel API
├── crates/                    # Driver space management daemons and services
└── drivers/                   # Individual hardware driver implementations
```

## 3. Component Breakdown

### 3.1. `lib/` (Core Libraries and Kernel API)
The `lib/` directory serves as the foundational layer of the driver space. It contains the new Kernel API, infrastructure utilities, and device-class frameworks. **No hardware-specific logic resides here.**

*   **Kernel API (`kapi-*`)**
    *   `kapi-abi`: The single source of truth for all data structures, IPC message formats, opcodes, and error codes. All structures are strictly `#[repr(C)]` to ensure binary compatibility across different languages (Rust, C, Odin).
    *   `kapi-syscall`: Low-level, architecture-specific wrappers for kernel transitions (e.g., SVC calls, interrupt handling).
*   **Infrastructure (`ds-*`)**
    *   `ds-ipc`: Inter-process communication primitives, channel management, and message routing logic.
    *   `ds-mem`: Memory management utilities, including custom allocators for `no_std` environments, DMA mapping, and shared memory handling.
    *   `ds-log`: Standardized logging infrastructure that routes driver logs to the central manager or kernel console.
*   **Device Frameworks (`ds-fw-*`)**
    *   `ds-fw-block`, `ds-fw-audio`, `ds-fw-gpu`, `ds-fw-input`: High-level abstractions for specific device classes. These frameworks provide traits (e.g., `BlockDevice`) and handle protocol boilerplate, allowing driver authors to focus solely on hardware register manipulation and state machines.

### 3.2. `crates/` (Driver Space Management)
The `crates/` directory contains the privileged user-space (or kernel-space) daemons responsible for orchestrating the driver ecosystem. These components manage the lifecycle of drivers but do not interact directly with hardware.

*   `ds-manager`: The central daemon responsible for device enumeration, resource allocation (IRQs, MMIO regions, memory pages), and driver instantiation. It listens for hardware events from the kernel and dispatches them to the appropriate drivers.
*   `ds-registry`: A database service that maintains the mapping of hardware device identifiers (e.g., PCI Vendor/Device IDs) to their corresponding driver modules.

### 3.3. `drivers/` (Hardware Drivers)
The `drivers/` directory contains the actual implementations for specific hardware peripherals (e.g., `amdgpu_driver`, `audiodriver`, `wacomgraphic_driver`). 

*   Each driver is an independent crate (or an external project for C/Odin implementations).
*   Drivers implement the traits defined in the `ds-fw-*` frameworks.
*   Drivers communicate with the `ds-manager` and the kernel exclusively through the APIs provided in `lib/`.

## 4. The New Kernel API (`kapi`)
The legacy C headers (`libs/*.h`) and the old `abi.rs` are deprecated. The new `kapi` (Kernel API) is designed specifically for TrangorgeOS's architecture. 

Key characteristics of the new API:
1.  **Explicit Message Passing:** All communication between the kernel, the manager, and the drivers is handled via strictly typed IPC messages defined in `kapi-abi`.
2.  **Capability-Based Security:** Resource sharing (like memory or IRQs) is handled through explicit capability transfers, ensuring drivers cannot access memory or hardware they are not authorized to use.
3.  **Language Agnostic:** The ABI is designed to be easily consumable by Rust, C, and Odin, facilitating a multi-language driver ecosystem.

## 5. Architectural Rules and Dependency Graph
To maintain the integrity of the workspace, the following strict rules apply:

1.  **Unidirectional Dependencies:** 
    *   `drivers/` may only depend on `lib/`.
    *   `crates/` may only depend on `lib/`.
    *   **Crucial Rule:** `drivers/` must **never** depend on `crates/`. Drivers must not have compile-time knowledge of the manager's internal implementation.
2.  **No Code Duplication:** If a utility, protocol parser, or hardware abstraction is needed by more than one driver, it must be extracted and placed into the appropriate module in `lib/` (either in `ds-*` infrastructure or `ds-fw-*` frameworks).
3.  **Strict `no_std` Compliance:** All crates within `lib/` and `crates/` must be `#![no_std]`. Standard library features are prohibited to ensure compatibility with the bare-metal and constrained environments of the OS.

## 6. Cross-Language Interoperability (FFI)
TrangorgeOS utilizes multiple languages (Rust, C, Odin, Nim). The `lib/kapi-abi` crate acts as the central boundary. 
*   Rust structures must use `#[repr(C)]`.
*   A build script (`build.rs`) utilizing `cbindgen` should be implemented in `kapi-abi` to automatically generate the corresponding C and Odin headers. This ensures that any modification to the Rust ABI is instantly reflected in the headers used by C and Odin drivers, preventing binary incompatibilities.

## 7. Summary
This modular workspace architecture transforms the TrangorgeOS driver space from a tightly coupled monolith into a scalable, secure, and maintainable subsystem. By enforcing strict dependency rules, centralizing the ABI in `lib/kapi-abi`, and separating management logic (`crates/`) from hardware logic (`drivers/`), the system is now prepared for robust multi-language driver development and secure resource isolation.