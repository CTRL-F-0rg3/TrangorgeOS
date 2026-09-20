# TrangorgeOS — build orchestration (Just).
#
# Replaces the `ctrlfile` build tool, which was removed from the repo for now
# (see README.md). Run `just <recipe>` from the repository root, or `just` for
# the default `build` recipe.

set shell := ["bash", "-c"]

# Default: build the whole thing (driver libraries -> kernel bootimage).
build: kernel
    @echo "[just] build OK"

# Build the exported hardware-driver libraries (no_std, x86_64-unknown-none).
drivers:
    cargo build --manifest-path kernel-drivers/Cargo.toml

# Build the kernel and its bootable image (depends on the driver libraries).
kernel: drivers
    cd kernel_Workspace/kernel-bin && cargo bootimage

# Run the kernel in QEMU (graphical window, serial on stdout).
run: kernel
    cd kernel_Workspace/kernel-bin && ./run.sh

# Quick check without linking.
check:
    cargo check --manifest-path kernel-drivers/Cargo.toml
    cargo check --manifest-path kernel_Workspace/Cargo.toml -p kernel-bin
