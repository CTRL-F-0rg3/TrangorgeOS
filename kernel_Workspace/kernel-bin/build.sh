#!/usr/bin/env bash
# Build a bootable disk image for TrangorgeOS (kernel-bin) and optionally run it in QEMU.
#
# Usage:
#   ./build.sh          # build target/x86_64-kernel/debug/bootimage-kernel-bin.bin
#   ./build.sh run      # build + run in QEMU
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WS="$(cd "$HERE/.." && pwd)"

# rustup / cargo home (VS Code's shell may not have ~/.cargo/bin on PATH)
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"

if ! command -v cargo-bootimage >/dev/null 2>&1; then
    echo "[build] installing cargo-bootimage 0.10 (matches bootloader 0.9)..." >&2
    cargo install bootimage --version "^0.10" --locked
fi

cd "$HERE"

# -Zbuild-std must be passed explicitly: the workspace builds for the custom
# JSON target kernel/x86_64-kernel.json which has no prebuilt sysroot.
# compiler-builtins-mem provides memset/memcpy/memcmp for the no_std target.
STD_FLAGS="-Zbuild-std=core,alloc,compiler_builtins -Zbuild-std-features=compiler-builtins-mem"

echo "[build] cargo bootimage $STD_FLAGS" >&2
cargo bootimage $STD_FLAGS

IMAGE="$WS/target/x86_64-kernel/debug/bootimage-kernel-bin.bin"
echo "[build] image: $IMAGE" >&2

if [ "${1:-}" = "run" ]; then
    shift
    exec "$HERE/run.sh" "${@}"
fi