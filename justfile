# TrangorgeOS — build orchestration (Just).
#
# Replaces the `ctrlfile` build tool, which was removed from the repo for now
# (see README.md). Run `just <recipe>` from the repository root, or `just` for
# the default `build` recipe.

set shell := ["bash", "-c"]

# Default: build the whole thing (driver libraries -> kernel bootimage).
build: drivers ds-libs gfx-libs vise-libs kernel
    @echo "[just] full build OK"

# Build the exported hardware-driver libraries (no_std, x86_64-unknown-none).
drivers:
    cargo build --manifest-path kernel-drivers/Cargo.toml

# Build the Driver Space workspace (ABI, IPC, manager, drivers).
#
# Three different build shapes live in this workspace, and one `cargo build`
# cannot express all three — that is why this is three recipes and not one line.
#
# 1. `lib/*` — host-side crates (tests, tooling). Built for the host.
# 2. `drivers/*` that are `#![no_std] #![no_main]` — real bare-metal drivers,
#    loaded by the Driver Space manager. They must be cross-compiled: linking one
#    against the host fails with `undefined symbol: main`, which is a missing
#    `--target` rather than a fault in the driver.
# 3. `drivers/*` that are ordinary `std` programs (tools, prototypes) — built
#    for the host, and they would *also* fail under `--target x86_64-unknown-none`
#    because there is no `std` for a bare-metal target.
ds-libs: ds-host ds-drivers ds-tools

# (1) The ABI/IPC/manager crates.
ds-host:
    cargo build --manifest-path driverspace_workspace/Cargo.toml -p ds-detect -p ds-manager -p ds-registry

# (2) The bare-metal drivers, cross-compiled. Only the crates that are actually
# `#![no_std] #![no_main]` belong here; adding a `std` one makes this recipe fail
# with `can't find crate for 'std'`.
#
# The package names come from each `Cargo.toml` and are not always the directory
# names (`IntelGpu-TrangorgeOS` -> `intel-gpu-trangorgeos`).
ds-drivers:
    cargo build --manifest-path driverspace_workspace/Cargo.toml \
        -p vgpu -p alsa-trangorgeos \
        --target x86_64-unknown-none

# (3) The host-side driver tools: prototypes and harness programs, not firmware.
ds-tools:
    cargo build --manifest-path driverspace_workspace/Cargo.toml \
        -p audiodriver -p intel-gpu-trangorgeos -p amd-gpu-trangorgeos \
        -p netcam_driver -p wacomgraphic_driver

# Build the Graphics workspace.
#
# Split for the same reason as `ds-libs`: `gfx-server` is `#![no_std] #![no_main]`
# firmware, while the protocol and API crates are host-side. One `cargo build`
# links the server against the host and fails on `undefined symbol: main`.
gfx-libs: gfx-host gfx-server

gfx-host:
    cargo build --manifest-path gfx_protocol_workspace/Cargo.toml -p gfx-protocol -p gfx-api

gfx-server:
    cargo build --manifest-path gfx_protocol_workspace/Cargo.toml -p gfx-server --target x86_64-unknown-none

# Build the Vise LG language workspace (compiler, runtime, IR).
vise-libs:
    cargo build --manifest-path vise_lg_workspace/Cargo.toml

# Build the kernel and its bootable image (depends on the driver libraries).
kernel: drivers
    cd kernel_Workspace/kernel-bin && cargo bootimage

# Run the kernel in QEMU (graphical window, serial on stdout). Also builds the
# userspace workspace so the whole system compiles together.
run: kernel uspace-libs
    cd kernel_Workspace/kernel-bin && ./run.sh

# Build the userspace workspace (nested system + graphical-window demo).
uspace-libs:
    cargo build -p demo-gfx --manifest-path userspace_worspace/Cargo.toml

# Run the userspace graphical-window demo -> writes demo_window.ppm.
demouserspace: uspace-libs
    cargo run -p demo-gfx --manifest-path userspace_worspace/Cargo.toml

# Run the userspace all-de desktop environment demo -> writes allde_frame.ppm.
allde:
    cargo run --manifest-path allde/Cargo.toml

# Build the userspace standard library (ustd) — a real Rust `std` for ring 3.
# The syscall layer, the C ABI and `_start` come from ustd/; the `std` itself is
# upstream, built by the script because there is no prebuilt one for the target.
uspace-std:
    bash ./ustd/build.sh

# Type-check the userspace target without linking, for CI.
uspace-std-check:
    bash ./ustd/build.sh --check

# The host tests for tgs-hal: the allocator, the descriptor table and the
# dirent codec, all driven against a mock kernel. Needs no target and no
# `-Zbuild-std`, so it is fast enough for a pre-commit hook.
uspace-std-test:
    cargo test --manifest-path ustd/Cargo.toml -p tgs-hal

# Build the bootable demo ISO -> dist/TrangorgeOS-<version>-x86_64-demo.iso
# Flags are forwarded to tools/mkiso.sh, e.g. `just iso --release --no-build`.
# Invoked through `bash` so the scripts do not need the executable bit.
iso *ARGS:
    bash ./tools/mkiso.sh {{ARGS}}

# Build the driver image: every driver plus a manifest saying what each claims
# to handle, so boot can start only the ones the machine actually needs.
# Flags are forwarded to tools/mkdriverimg.sh.
driverimg *ARGS:
    bash ./tools/mkdriverimg.sh {{ARGS}}

# Print the driver catalogue the image would carry, without building it.
driverimg-list:
    bash ./tools/mkdriverimg.sh --list

# Run the detection layer's tests (scan, match, plan, calibrate).
detect-test:
    cargo test --manifest-path driverspace_workspace/Cargo.toml -p ds-detect

# Boot the newest demo ISO in QEMU (graphical window, serial on stdout).
iso-run *ARGS: iso
    bash ./tools/run-iso.sh {{ARGS}}

# Automated ISO smoke test: no window, serial log captured, PASS/FAIL report.
iso-test: iso
    bash ./tools/run-iso.sh --headless

# Verify the ISO build dependencies (xorriso, syslinux, cargo-bootimage, ...).
iso-deps:
    bash ./tools/mkiso.sh --check-deps

# Remove all ISO build artefacts (dist/).
iso-clean:
    rm -rf dist

# Quick check without linking (verifies all workspaces).
check:
    cargo check --manifest-path kernel-drivers/Cargo.toml
    cargo check --manifest-path kernel_Workspace/Cargo.toml
    cargo check --manifest-path driverspace_workspace/Cargo.toml
    cargo check --manifest-path gfx_protocol_workspace/Cargo.toml
    cargo check --manifest-path vise_lg_workspace/Cargo.toml

# Clean all build artifacts across all workspaces.
clean:
    cargo clean --manifest-path kernel-drivers/Cargo.toml
    cargo clean --manifest-path kernel_Workspace/Cargo.toml
    cargo clean --manifest-path driverspace_workspace/Cargo.toml
    cargo clean --manifest-path gfx_protocol_workspace/Cargo.toml
    cargo clean --manifest-path vise_lg_workspace/Cargo.toml
    @echo "[just] all workspaces cleaned"