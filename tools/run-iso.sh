#!/usr/bin/env bash
# TrangorgeOS — boot the demo ISO in QEMU (BIOS).
#
# Device layout mirrors kernel_Workspace/kernel-bin/run.sh:
#   * CD-ROM  : the ISO. QEMU puts -cdrom on the *secondary* IDE channel, which
#               the kernel's ATA-PIO driver never probes (it only touches the
#               primary channel 0x1F0), so the CD cannot confuse the FS test.
#   * hda     : a small writable disk used as the TFS root device. The kernel's
#               fs self-test reads sector 0 of the first ATA disk and demands an
#               0x55AA MBR signature, then formats the root device with TFS, so
#               the disk is created with that signature.
#
# Usage:
#   tools/run-iso.sh [options] [-- extra qemu args...]
#
# Options:
#   --iso=FILE            boot this ISO (default: newest dist/*.iso)
#   --headless            no window; serial log to file; auto-exit (smoke test)
#   --display=MODE        gtk (default) | sdl | none
#   --timeout=SECONDS     stop QEMU after N seconds (default: 120 in headless mode)
#   --serial-log=FILE     serial output file (default: dist/serial-<iso>.log)
#   --root-disk=FILE      writable TFS disk (default: dist/demo-root-disk.img)
#   --fresh-disk          re-create the root disk instead of reusing it
#   --snapshot            discard disk writes (QEMU -snapshot; parallel runs OK)
#   --no-snapshot         always persist writes to the root disk
#   --no-disk             boot without any ATA disk (fs test will report FAILED)
#   -h | --help           this text
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="$REPO_ROOT/dist"

ISO=""
HEADLESS=0
DISPLAY="gtk"
TIMEOUT=0
SERIAL_LOG=""
ROOT_DISK=""
FRESH_DISK=0
NO_DISK=0
SNAPSHOT=""
EXTRA_QEMU=()

usage() { sed -n '2,27p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; }
log()   { printf '[run-iso] %s\n' "$*" >&2; }
die()   { printf '[run-iso] ERROR: %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
    case "$1" in
        --iso=*)         ISO="${1#--iso=}" ;;
        --headless)      HEADLESS=1; DISPLAY="none" ;;
        --display=*)     DISPLAY="${1#--display=}" ;;
        --timeout=*)     TIMEOUT="${1#--timeout=}" ;;
        --serial-log=*)  SERIAL_LOG="${1#--serial-log=}" ;;
        --root-disk=*)   ROOT_DISK="${1#--root-disk=}" ;;
        --fresh-disk)    FRESH_DISK=1 ;;
        --snapshot)      SNAPSHOT=1 ;;
        --no-snapshot)   SNAPSHOT=0 ;;
        --no-disk)       NO_DISK=1 ;;
        -h|--help)       usage; exit 0 ;;
        --)              shift; EXTRA_QEMU=("$@"); break ;;
        *)               die "unknown option: $1 (try --help)" ;;
    esac
    shift
done

# Automated (headless) runs default to a throw-away overlay: the QEMU write lock
# on the root disk otherwise blocks a second instance, and a fresh TFS format on
# every boot makes the smoke test reproducible. Use --no-snapshot to persist.
if [ -z "$SNAPSHOT" ]; then
    if [ "$HEADLESS" = "1" ]; then SNAPSHOT=1; else SNAPSHOT=0; fi
fi

command -v qemu-system-x86_64 >/dev/null 2>&1 || \
    die "qemu-system-x86_64 not found (Debian/Ubuntu: sudo apt-get install qemu-system-x86)"

# --------------------------------------------------------------------------- #
# Locate the ISO
# --------------------------------------------------------------------------- #

if [ -z "$ISO" ]; then
    ISO="$(ls -1t "$DIST"/*.iso 2>/dev/null | head -n1 || true)"
fi

if [ -z "$ISO" ] || [ ! -f "$ISO" ]; then
    die "no ISO found - build one first:  tools/mkiso.sh"
fi

ISO="$(cd "$(dirname "$ISO")" && pwd)/$(basename "$ISO")"
ISO_NAME="$(basename "$ISO")"
log "ISO: $ISO"

# --------------------------------------------------------------------------- #
# Writable root disk (TFS)
# --------------------------------------------------------------------------- #

if [ "$NO_DISK" = "0" ]; then
    [ -n "$ROOT_DISK" ] || ROOT_DISK="$DIST/demo-root-disk.img"
    if [ "$FRESH_DISK" = "1" ] || [ ! -f "$ROOT_DISK" ]; then
        log "creating writable root disk: $ROOT_DISK"
        dd if=/dev/zero of="$ROOT_DISK" bs=1M count=8 status=none
        # MBR signature - required by kernel/src/fs/mod.rs self_test().
        printf '\x55\xaa' | dd of="$ROOT_DISK" bs=1 seek=510 conv=notrunc status=none
    else
        log "reusing writable root disk: $ROOT_DISK (--fresh-disk resets it)"
    fi
fi

# --------------------------------------------------------------------------- #
# Serial / display handling
# --------------------------------------------------------------------------- #

QEMU_ARGS=(
    -cdrom "$ISO"
    -device qemu-xhci
    -m 512M
    -smp 4
    -boot order=d
    -no-reboot
    -display "$DISPLAY"
)

if [ -n "$SERIAL_LOG" ] || [ "$HEADLESS" = "1" ]; then
    [ -n "$SERIAL_LOG" ] || SERIAL_LOG="$DIST/serial-${ISO_NAME%.iso}.log"
    mkdir -p "$(dirname "$SERIAL_LOG")"
    rm -f "$SERIAL_LOG"
    QEMU_ARGS+=( -serial "file:$SERIAL_LOG" )
    log "serial log: $SERIAL_LOG"
else
    QEMU_ARGS+=( -serial stdio )
fi

if [ "$NO_DISK" = "0" ]; then
    QEMU_ARGS+=( -drive "format=raw,file=$ROOT_DISK,if=ide,index=0" )
fi

if [ "$SNAPSHOT" = "1" ]; then
    QEMU_ARGS+=( -snapshot )
    log "disk writes are discarded (QEMU -snapshot)"
fi

if [ "$HEADLESS" = "1" ] && [ "$TIMEOUT" = "0" ]; then
    # A cold boot of the current alpha kernel takes well over a minute in QEMU
    # (the graphics/console init alone is slow), so do not be too eager here.
    TIMEOUT=120
fi

log "starting QEMU (display=$DISPLAY, timeout=${TIMEOUT:-none}s)"

if [ "$TIMEOUT" != "0" ] && command -v timeout >/dev/null 2>&1; then
    timeout --signal=TERM --kill-after=5 "$TIMEOUT" \
        qemu-system-x86_64 "${QEMU_ARGS[@]}" ${EXTRA_QEMU[@]+"${EXTRA_QEMU[@]}"} || true
elif [ "$TIMEOUT" != "0" ]; then
    qemu-system-x86_64 "${QEMU_ARGS[@]}" ${EXTRA_QEMU[@]+"${EXTRA_QEMU[@]}"} &
    qemu_pid=$!
    ( sleep "$TIMEOUT"; kill -TERM "$qemu_pid" 2>/dev/null || true ) &
    wait "$qemu_pid" 2>/dev/null || true
else
    exec qemu-system-x86_64 "${QEMU_ARGS[@]}" ${EXTRA_QEMU[@]+"${EXTRA_QEMU[@]}"}
fi

# --------------------------------------------------------------------------- #
# Smoke test (only meaningful when we captured serial output)
# --------------------------------------------------------------------------- #

if [ -z "$SERIAL_LOG" ] || [ ! -f "$SERIAL_LOG" ]; then
    exit 0
fi

echo
echo "=========== TrangorgeOS ISO smoke test ==========="
fails=0

check() { # check <description> <fixed-string pattern>
    if grep -qF -- "$2" "$SERIAL_LOG"; then
        printf '  [ OK ] %s\n' "$1"
    else
        printf '  [FAIL] %s  (expected: %s)\n' "$1" "$2"
        fails=$((fails + 1))
    fi
}

check "kernel banner reached"   'Welcome in my Galaxy!'
check "memory allocator online" '[mm] allocator initialized OK'
check "self-test suite ran"     'module test(s)...'

status_line="$(grep -o 'SYSTEM STATUS: .*' "$SERIAL_LOG" | tail -n1 || true)"
if [ -n "$status_line" ]; then
    echo "  -> $status_line"
else
    printf '  [FAIL] no SYSTEM STATUS line in the serial log\n'
    fails=$((fails + 1))
fi

if grep -q 'SYSTEM STATUS: [0-9]*/[0-9]* FAILED' "$SERIAL_LOG"; then
    echo "  [warn] some self-tests failed - see the serial log tail below"
fi

echo "================================================="
echo "serial log tail ($SERIAL_LOG):"
tail -n 30 "$SERIAL_LOG"
echo "================================================="

if [ "$fails" != "0" ]; then
    die "$fails smoke-test check(s) failed"
fi

log "smoke test PASSED"
