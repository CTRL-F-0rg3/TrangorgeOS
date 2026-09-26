# TODO.md: Driver Space Migration and Implementation Guide

## Overview
This document outlines the pending tasks, architectural constraints, and execution steps for migrating and implementing hardware drivers within the newly structured `driverspace_workspace`. 

The legacy monolithic implementations (`driverspacelib` and the old `driverspace` daemon) are deprecated. All driver development must now occur in the `drivers/` directory of the new workspace, strictly adhering to the modular architecture.

## 1. Prerequisites
Before beginning driver implementation, the foundational libraries located in `lib/` must be fully implemented and stabilized. Drivers cannot be compiled or tested without the following components:

- [ ] **`lib/kapi-abi`**: Finalize all IPC message structures, opcodes, and error codes. Ensure `cbindgen` is generating accurate C/Odin headers.
- [ ] **`lib/kapi-syscall`**: Implement architecture-specific SVC/syscall wrappers.
- [ ] **`lib/ds-ipc`**: Stabilize the channel, port, and shared memory abstractions.
- [ ] **`lib/ds-mem`**: Finalize the `no_std` allocators (Buddy/Slab) and DMA mapping utilities.
- [ ] **`lib/ds-fw-*`**: Define the core traits for device classes (`BlockDevice`, `AudioDevice`, `DisplayDevice`, `InputDevice`).

## 2. Driver Implementation Checklist
The following drivers require migration from the legacy codebase or complete reimplementation using the new framework traits.




### 2.3. Input and HID
- [ ] **`wacomgraphic_driver`**
  - [ ] Implement Wacom-specific HID report parsing.
  - [ ] Implement raw-to-logical data conversion for pen pressure and tilt (X/Y).
  - [ ] Implement the `InputDevice` trait from `ds-fw-input`.

### 2.4. Networking and Multimedia
- [ ] **`netcam_driver`**
  - [ ] Implement USB Video Class (UVC) protocol parsing.
  - [ ] Implement isochronous USB transfer handling.
  - [ ] Implement device control interfaces (brightness, contrast, zoom).

## 3. Strict Architectural Rules for Contributors
All contributors must adhere to the following rules when writing driver code. Violations will result in rejected pull requests.

1. **Unidirectional Dependencies**: 
   - Drivers (`drivers/`) may **only** depend on libraries in `lib/`. 
   - Drivers must **never** depend on the management daemons in `crates/`. Communication with the manager must happen exclusively via IPC (`ds-ipc`).
2. **No Standard Library**: 
   - All Rust driver code must be `#![no_std]`. The use of the Rust standard library is strictly prohibited.
3. **Zero Code Duplication**: 
   - If a utility, protocol parser, or hardware abstraction is needed by more than one driver, it must be extracted and placed into the appropriate module in `lib/ds-fw-*` or `lib/ds-*`. Do not copy-paste code between drivers.
4. **Strict FFI Boundaries**: 
   - If a driver requires C or Odin code, it must be isolated in a dedicated subdirectory (e.g., `c/` or `odin/`). 
   - Compilation of foreign code must be handled via `build.rs` using the `cc` crate or equivalent tooling. 
   - Rust wrappers for foreign functions must be isolated in an `ffi.rs` file.
5. **Capability and Resource Management**: 
   - Drivers must not attempt to access hardware resources (MMIO, IRQs, DMA) directly via hardcoded addresses. All resources must be requested from and granted by the `ds-manager` via the new Kernel API (`kapi-abi`).

## 4. Execution Steps for a New Driver
When starting work on a driver, follow this exact sequence:

1. **Initialize the Crate**: Create the directory under `drivers/<driver_name>/` and generate a new `Cargo.toml` with `#![no_std]` configured.
2. **Update Workspace**: Add the new crate path to the `members` array in the root `driverspace_workspace/Cargo.toml`.
3. **Define Dependencies**: Add only the necessary `lib/` crates to the driver's `Cargo.toml` (e.g., `kapi-abi`, `ds-ipc`, `ds-fw-gpu`).
4. **Implement Entry Point**: Create `main.rs` (or `lib.rs`), initialize the driver runtime, and establish the IPC connection to the `ds-manager`.
5. **Implement the Trait**: Implement the required trait from the corresponding `ds-fw-*` framework.
6. **Hardware Logic**: Implement the actual hardware register manipulation and state machines.

## 5. Current Status
- [ ] Workspace structure created and validated.
- [ ] Legacy `driverspacelib` code analyzed for extraction.
- [ ] `lib/kapi-abi` definitions finalized.
- [ ] First driver (`amdgpu_driver` or `audiodriver`) migrated to the new workspace.

*For questions regarding the architecture or the new Kernel API, refer to the main Architecture Documentation or consult the core maintainers.*