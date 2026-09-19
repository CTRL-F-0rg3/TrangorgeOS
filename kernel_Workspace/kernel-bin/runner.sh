#!/usr/bin/env bash
# Invoked by `cargo run` as the runner for the `x86_64-kernel` target.
#
# cargo passes the freshly compiled bare-metal ELF as the first argument.
# That ELF is not directly bootable (it needs the bootloader), so we drop it
# and hand control to build.sh (bootimage) + run.sh (QEMU).
#
# Usage:
#   cargo run                      # build + boot (graphical window, no timeout)
#   cargo run -- -device virtio-net-pci  # forward extra QEMU args
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# $1 = compiled kernel-bin ELF (not used here)
shift || true

"$HERE/build.sh"
exec "$HERE/run.sh" "$@"
