#!/usr/bin/env bash
# Run the freshly built TrangorgeOS bootimage in QEMU.
#
# Opens a graphical window (VGA console) and mirrors the serial log to stdout.
# Runs until the window is closed (no timeout).
#
# Usage:
#   ./run.sh [extra qemu args...]
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WS="$(cd "$HERE/.." && pwd)"

IMAGE="$WS/target/x86_64-kernel/debug/bootimage-kernel-bin.bin"
DATA_IMG="$HERE/data.img"

if [ ! -f "$IMAGE" ]; then
    echo "bootimage not found: $IMAGE" >&2
    echo "run ./build.sh first" >&2
    exit 1
fi

# TFS data disk (second IDE drive). The kernel's fs self-test uses it as the
# writable root device and formats it with TFS on first use.
if [ ! -f "$DATA_IMG" ]; then
    echo "[run] creating blank data disk: $DATA_IMG" >&2
    dd if=/dev/zero of="$DATA_IMG" bs=1M count=8 status=none
fi

exec qemu-system-x86_64 \
    -drive "format=raw,file=$IMAGE" \
    -drive "format=raw,file=$DATA_IMG,if=ide,index=1" \
    -device qemu-xhci \
    -m 512M \
    -smp 4 \
    -serial stdio \
    -display gtk \
    -no-reboot \
    "$@"

