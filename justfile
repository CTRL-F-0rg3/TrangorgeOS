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
ds-libs:
    cargo build --manifest-path driverspace_workspace/Cargo.toml

# Build the Graphics workspace (protocols, server, API).
gfx-libs:
    cargo build --manifest-path gfx_workspace/Cargo.toml

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

# Build the bootable demo ISO -> dist/TrangorgeOS-<version>-x86_64-demo.iso
# Flags are forwarded to tools/mkiso.sh, e.g. `just iso --release --no-build`.
# Invoked through `bash` so the scripts do not need the executable bit.
iso *ARGS:
    bash ./tools/mkiso.sh {{ARGS}}

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
    cargo check --manifest-path gfx_workspace/Cargo.toml
    cargo check --manifest-path vise_lg_workspace/Cargo.toml

# Clean all build artifacts across all workspaces.
clean:
    cargo clean --manifest-path kernel-drivers/Cargo.toml
    cargo clean --manifest-path kernel_Workspace/Cargo.toml
    cargo clean --manifest-path driverspace_workspace/Cargo.toml
    cargo clean --manifest-path gfx_workspace/Cargo.toml
    cargo clean --manifest-path vise_lg_workspace/Cargo.toml
    @echo "[just] all workspaces cleaned"