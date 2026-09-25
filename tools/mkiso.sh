#!/usr/bin/env bash
# TrangorgeOS — build a bootable demo .iso image (BIOS/El Torito).
#
# Pipeline:
#   1. build the bootable kernel image   (cargo bootimage -> bootimage-kernel-bin.bin)
#   2. stage an ISO9660 tree             (dist/iso-root/)
#   3. wrap it into an El Torito ISO     (xorriso; ISOLINUX + MEMDISK)
#   4. checksums + release manifest      (dist/*.sha256, dist/RELEASE.txt)
#
# Why ISOLINUX + MEMDISK?
#   The only boot path that works today is the bootloader 0.9 BIOS image built by
#   `cargo bootimage` (see kernel_Workspace/kernel-bin/run.sh).  That artefact is
#   a raw disk image driven through the BIOS with 512-byte sectors, so it cannot
#   be started by "no emulation" El Torito directly: the BIOS would read the CD
#   (and hand the drive over) in 2048-byte sectors.
#   ISOLINUX is a no-emulation El Torito bootloader that installs its own INT 13h
#   handlers, and MEMDISK then boots our *unmodified* raw image from RAM - i.e.
#   byte-for-byte the artefact QEMU already boots today.
#
#   `--boot=hdemul` is provided as a syslinux-free fallback (El Torito hard disk
#   emulation); it depends on BIOS behaviour, see tools/iso/README.md.
#
# Usage:
#   tools/mkiso.sh [options]
#
# Options:
#   --release             build the kernel with `cargo bootimage --release`
#   --no-build            reuse an already built bootimage
#   --boot=MODE           memdisk (default) | hdemul
#   --version=STRING      version used in the artefact name / volume label
#   --out=FILE            output ISO (default: dist/TrangorgeOS-<version>-x86_64-demo.iso)
#   --synthesize-mbr      hdemul only: add a synthetic partition entry to the image
#   --check-deps          only verify build dependencies, then exit
#   -h | --help           this text
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WS="$REPO_ROOT/kernel_Workspace"
BIN_CRATE="$WS/kernel-bin"
DIST="$REPO_ROOT/dist"
STAGE="$DIST/iso-root"

# rustup/cargo home (a bare non-login shell may not have ~/.cargo/bin on PATH).
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"

PROFILE="debug"
DO_BUILD=1
BOOT_MODE="memdisk"
SYNTH_MBR=0
CHECK_DEPS=0
VERSION="${TRANGORGE_VERSION:-}"
OUT_ISO=""

# Custom target: `cargo bootimage` links the image with a JSON target spec that
# has no prebuilt sysroot, so build-std must be passed explicitly (same flags as
# kernel-bin/build.sh).
STD_FLAGS="-Zbuild-std=core,alloc,compiler_builtins -Zbuild-std-features=compiler-builtins-mem"

log()  { printf '[mkiso] %s\n' "$*" >&2; }
die()  { printf '[mkiso] ERROR: %s\n' "$*" >&2; exit 1; }
warn() { printf '[mkiso] WARNING: %s\n' "$*" >&2; }

usage() { sed -n '2,34p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; }

for arg in "$@"; do
    case "$arg" in
        --release)       PROFILE="release" ;;
        --no-build)      DO_BUILD=0 ;;
        --boot=*)        BOOT_MODE="${arg#--boot=}" ;;
        --version=*)     VERSION="${arg#--version=}" ;;
        --out=*)         OUT_ISO="${arg#--out=}" ;;
        --synthesize-mbr) SYNTH_MBR=1 ;;
        --check-deps)    CHECK_DEPS=1 ;;
        -h|--help)       usage; exit 0 ;;
        *)               die "unknown option: $arg (try --help)" ;;
    esac
done

case "$BOOT_MODE" in
    memdisk|hdemul) ;;
    *) die "--boot must be 'memdisk' or 'hdemul' (got '$BOOT_MODE')" ;;
esac


# --------------------------------------------------------------------------- #
# 0. Toolchain / dependency discovery
# --------------------------------------------------------------------------- #

# xorriso ships both the native binary and the `xorrisofs` wrapper. The wrapper
# already implies `-as mkisofs`, so only the native binary must be told about it.
XORRISO=""
XORRISO_EMUL=0          # 1 => prepend "-as mkisofs"
if command -v xorriso >/dev/null 2>&1; then
    XORRISO="xorriso"
    XORRISO_EMUL=1
elif command -v xorrisofs >/dev/null 2>&1; then
    XORRISO="xorrisofs"
fi

# ISOLINUX/MEMDISK assets. SYSLINUX_DIR overrides the search (handy for a
# syslinux source tree, e.g. SYSLINUX_DIR=~/syslinux-6.03/... ).
ISOLINUX_BIN=""
LDLINUX_C32=""
MEMDISK_BIN=""
ISOHDPFX_BIN=""

find_asset() { # find_asset <basename> [dirs...]
    local name="$1"; shift
    local dir
    for dir in "$@"; do
        if [ -f "$dir/$name" ]; then printf '%s\n' "$dir/$name"; return 0; fi
    done
    return 1
}

# syslinux asset locations. Arch/Manjaro use /usr/lib/syslinux/bios, Debian/Ubuntu
# use /usr/lib/syslinux/modules/bios (plus /usr/lib/ISOLINUX for isolinux.bin).
SYSLINUX_DIR="${SYSLINUX_DIR:-}"
SYS_DIRS=(
    /usr/lib/ISOLINUX
    /usr/lib/syslinux
    /usr/lib/syslinux/bios
    /usr/lib/syslinux/modules/bios
    /usr/share/syslinux
    /usr/lib64/syslinux
    /usr/lib64/syslinux/bios
    /usr/lib64/syslinux/modules/bios
    /usr/share/syslinux/bios
)
if [ -n "$SYSLINUX_DIR" ]; then
    SYS_DIRS=("$SYSLINUX_DIR" "${SYS_DIRS[@]}")
fi

ISOLINUX_BIN="$(find_asset isolinux.bin "${SYS_DIRS[@]}" || true)"
LDLINUX_C32="$(find_asset ldlinux.c32 "${SYS_DIRS[@]}" || true)"
MEMDISK_BIN="$(find_asset memdisk "${SYS_DIRS[@]}" || true)"
ISOHDPFX_BIN="$(find_asset isohdpfx.bin "${SYS_DIRS[@]}" || true)"

check_deps() {
    local missing=0

    for c in cargo rustup dd; do
        if ! command -v "$c" >/dev/null 2>&1; then
            warn "missing command: $c"
            missing=1
        fi
    done

    if [ -z "$XORRISO" ]; then
        warn "missing command: xorriso (provides xorrisofs)"
        missing=1
    fi

    if [ "$BOOT_MODE" = "memdisk" ]; then
        [ -n "$ISOLINUX_BIN" ] || { warn "missing isolinux.bin (syslinux-common)"; missing=1; }
        [ -n "$LDLINUX_C32" ]  || { warn "missing ldlinux.c32 (syslinux-common)"; missing=1; }
        [ -n "$MEMDISK_BIN" ]  || { warn "missing memdisk (syslinux-common)"; missing=1; }
        if [ -z "$ISOHDPFX_BIN" ]; then
            warn "missing isohdpfx.bin -> ISO will not get a hybrid MBR (USB boot)"
        fi
    fi

    if [ "$DO_BUILD" = "1" ] && ! command -v cargo-bootimage >/dev/null 2>&1; then
        warn "cargo-bootimage not installed yet (build.sh installs it on demand)"
    fi

    if [ "$SYNTH_MBR" = "1" ] && ! command -v python3 >/dev/null 2>&1; then
        warn "missing command: python3 (required by --synthesize-mbr)"
        missing=1
    fi

    if [ "$missing" != "0" ]; then
        cat >&2 <<'EOF'
[mkiso] Install the missing packages, then re-run:

  Debian/Ubuntu : sudo apt-get install -y xorriso syslinux-common
  Fedora/RHEL   : sudo dnf install -y xorriso syslinux
  Arch          : sudo pacman -S --needed libisoburn syslinux

  (syslinux-common provides isolinux.bin, ldlinux.c32, memdisk, isohdpfx.bin.
   If it is not packaged on your distro, set SYSLINUX_DIR to a syslinux tree.)
EOF
        return 1
    fi

    log "dependencies OK (xorriso: $XORRISO)"
    if [ -n "$ISOLINUX_BIN" ]; then log "isolinux.bin : $ISOLINUX_BIN"; fi
    if [ -n "$MEMDISK_BIN" ];  then log "memdisk      : $MEMDISK_BIN"; fi
    if [ -n "$ISOHDPFX_BIN" ]; then log "isohdpfx.bin : $ISOHDPFX_BIN"; fi
    return 0
}

# --------------------------------------------------------------------------- #
# 1. Version string
# --------------------------------------------------------------------------- #

detect_version() {
    if [ -n "$VERSION" ]; then printf '%s\n' "$VERSION"; return; fi
    if [ -f "$REPO_ROOT/VERSION" ]; then
        tr -d '[:space:]' < "$REPO_ROOT/VERSION"; return
    fi
    if command -v git >/dev/null 2>&1 && git -C "$REPO_ROOT" rev-parse --git-dir >/dev/null 2>&1; then
        local tag
        tag="$(git -C "$REPO_ROOT" describe --tags --abbrev=0 2>/dev/null || true)"
        if [ -n "$tag" ]; then printf '%s\n' "${tag#v}"; return; fi
        printf 'dev-%s\n' "$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
        return
    fi
    printf 'unversioned\n'
}

if ! check_deps; then exit 2; fi
if [ "$CHECK_DEPS" = "1" ]; then exit 0; fi

VERSION="$(detect_version)"
SUFFIX="-demo"          # keep the artefact clearly non-production
ARTIFACT="TrangorgeOS-${VERSION}-x86_64${SUFFIX}.iso"
[ -n "$OUT_ISO" ] || OUT_ISO="$DIST/$ARTIFACT"
IMAGE="$WS/target/x86_64-kernel/$PROFILE/bootimage-kernel-bin.bin"

log "version  : $VERSION"
log "profile  : $PROFILE"
log "boot mode: $BOOT_MODE"
log "image    : $IMAGE"
log "output   : $OUT_ISO"

# --------------------------------------------------------------------------- #
# 2. Build the bootable kernel image
# --------------------------------------------------------------------------- #

if [ "$DO_BUILD" = "1" ]; then
    if [ "$PROFILE" = "debug" ]; then
        # Known-good path: installs cargo-bootimage if needed and builds the
        # JSON-target image with build-std.
        log "building bootimage (debug) via kernel-bin/build.sh"
        if [ -x "$BIN_CRATE/build.sh" ]; then
            ( cd "$BIN_CRATE" && ./build.sh )
        else
            ( cd "$BIN_CRATE" && bash ./build.sh )
        fi
    else
        log "building bootimage (release)"
        if ! command -v cargo-bootimage >/dev/null 2>&1; then
            log "installing cargo-bootimage 0.10 (matches bootloader 0.9)"
            cargo install bootimage --version "^0.10" --locked
        fi
        ( cd "$BIN_CRATE" && cargo bootimage --release $STD_FLAGS )
    fi
else
    log "skipping build (--no-build)"
fi

[ -f "$IMAGE" ] || die "bootimage not found: $IMAGE (run without --no-build first)"

# Sanity check: a BIOS boot sector must end with the 0x55AA signature, otherwise
# neither MEMDISK nor a BIOS (nor QEMU) will start it.
MBR_SIG="$(dd if="$IMAGE" bs=1 skip=510 count=2 status=none | od -An -tx1 | tr -d '\n' | tr -s ' ')"
case "$MBR_SIG" in
    *"55 aa"*) : ;;
    *) die "$IMAGE does not look like a BIOS boot image (offset 510 = '$MBR_SIG', expected '55 aa')" ;;
esac

IMAGE_SIZE="$(stat -c '%s' "$IMAGE" 2>/dev/null || echo 0)"
log "bootimage: $IMAGE ($((IMAGE_SIZE / 1024)) KiB)"

if [ "$BOOT_MODE" = "hdemul" ] && [ "$IMAGE_SIZE" -lt 4194304 ]; then
    warn "image is smaller than 4 MiB; hard disk emulation expects a disk-sized image"
fi

# --------------------------------------------------------------------------- #
# 3. Stage the ISO9660 tree
# --------------------------------------------------------------------------- #

log "staging $STAGE"
rm -rf "$STAGE"
mkdir -p "$STAGE/boot" "$STAGE/isolinux"

cp "$IMAGE" "$STAGE/boot/trangorge.img"

if [ "$SYNTH_MBR" = "1" ]; then
    [ "$BOOT_MODE" = "hdemul" ] || warn "--synthesize-mbr is only meaningful with --boot=hdemul"
    # Experimental: write one partition entry (LBA 0, whole image, type 0x0C) into
    # the MBR area of the staged copy so that BIOS hard disk emulation has a
    # partition table to derive the geometry from. The bytes at 446..509 are
    # padding in the bootloader 0.9 boot sector, so the boot code is untouched.
    python3 - "$STAGE/boot/trangorge.img" <<'PY'
import os, struct, sys
path = sys.argv[1]
size = os.path.getsize(path)
sectors = size // 512
entry = bytearray(16)
entry[0] = 0x80                      # bootable
entry[1:4] = bytes((0x00, 0x02, 0x00))  # CHS start (cosmetic)
entry[4] = 0x0C                      # FAT32 LBA type
entry[5:8] = bytes((0xFE, 0xFF, 0xFF))
entry[8:12] = struct.pack('<I', 1)   # start LBA
entry[12:16] = struct.pack('<I', min(sectors - 1, 0xFFFFFFFF))
with open(path, 'r+b') as fh:
    fh.seek(446)
    fh.write(entry)
PY
fi

cp "$REPO_ROOT/tools/iso/isolinux.cfg" "$STAGE/isolinux/isolinux.cfg"
sed -i "s/@VERSION@/$VERSION/g" "$STAGE/isolinux/isolinux.cfg"
sed "s/@VERSION@/$VERSION/g" "$REPO_ROOT/tools/iso/README.txt" > "$STAGE/README.TXT"

if [ "$BOOT_MODE" = "memdisk" ]; then
    cp "$ISOLINUX_BIN" "$STAGE/isolinux/isolinux.bin"
    cp "$LDLINUX_C32"  "$STAGE/isolinux/ldlinux.c32"
    cp "$MEMDISK_BIN"  "$STAGE/isolinux/memdisk"
fi

printf '%s\n' "$VERSION" > "$STAGE/VERSION.TXT"

# --------------------------------------------------------------------------- #
# 4. Build the El Torito ISO
# --------------------------------------------------------------------------- #

mkdir -p "$DIST"
rm -f "$OUT_ISO"

log "creating $OUT_ISO with $XORRISO"

# `xorriso` needs the explicit mkisofs-emulation switch, `xorrisofs` already has it.
xorriso_base=()
if [ "$XORRISO_EMUL" = "1" ]; then
    xorriso_base=( -as mkisofs )
fi

if [ "$BOOT_MODE" = "memdisk" ]; then
    # Standard syslinux recipe (see syslinux/doc/isolinux.txt) + a hybrid MBR so
    # the same file can also be written to a USB stick with `dd`.
    xorriso_args=(
        ${xorriso_base[@]+"${xorriso_base[@]}"}
        -iso-level 2                # 31-char plain ISO9660 names: ldlinux.c32 etc.
        -R -J -joliet-long
        -V "TRANGORGEOS"
        -A "TrangorgeOS $VERSION"
        -b isolinux/isolinux.bin
        -c isolinux/boot.cat
        -no-emul-boot
        -boot-load-size 4
        -boot-info-table
    )
    if [ -n "$ISOHDPFX_BIN" ]; then
        xorriso_args+=( -isohybrid-mbr "$ISOHDPFX_BIN" )
    fi
    xorriso_args+=( -o "$OUT_ISO" "$STAGE" )
    "$XORRISO" "${xorriso_args[@]}"
else
    # Fallback: no syslinux needed, but the BIOS must support El Torito hard disk
    # emulation and should find a partition table in the boot image.
    "$XORRISO" ${xorriso_base[@]+"${xorriso_base[@]}"} \
        -iso-level 2 -R -J -joliet-long \
        -V "TRANGORGEOS" \
        -A "TrangorgeOS $VERSION" \
        -b boot/trangorge.img \
        -c boot/boot.cat \
        -hard-disk-boot \
        -o "$OUT_ISO" "$STAGE"
fi

[ -f "$OUT_ISO" ] || die "xorriso did not produce $OUT_ISO"

ISO_SIZE="$(stat -c '%s' "$OUT_ISO")"
log "ISO size: $((ISO_SIZE / 1024 / 1024)) MiB"

# --------------------------------------------------------------------------- #
# 5. Publication artefacts: checksum + release manifest
# --------------------------------------------------------------------------- #

( cd "$(dirname "$OUT_ISO")" && sha256sum "$(basename "$OUT_ISO")" > "$(basename "$OUT_ISO").sha256" )

GIT_BRANCH="unknown"; GIT_COMMIT="unknown"
if command -v git >/dev/null 2>&1 && git -C "$REPO_ROOT" rev-parse --git-dir >/dev/null 2>&1; then
    GIT_BRANCH="$(git -C "$REPO_ROOT" rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)"
    GIT_COMMIT="$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo unknown)"
fi

TOOLCHAIN="$(rustc --version 2>/dev/null || echo 'rustc not on PATH')"
BUILD_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
BOOTIMAGE_SHA="$(sha256sum "$IMAGE" | cut -d' ' -f1)"

cat > "$DIST/RELEASE.txt" <<EOF
TrangorgeOS — demo ISO release manifest
=======================================

Artifact      : $(basename "$OUT_ISO")
Version       : $VERSION
Architecture  : x86_64
Boot path     : bootloader 0.9 (BIOS) + El Torito ($BOOT_MODE)
Firmware      : LEGACY BIOS ONLY (no UEFI support yet)
ISO size      : $ISO_SIZE bytes
ISO sha256    : $(cut -d' ' -f1 < "$OUT_ISO.sha256")
Kernel image  : bootimage-kernel-bin.bin ($IMAGE_SIZE bytes)
Kernel sha256 : $BOOTIMAGE_SHA
Profile       : $PROFILE
Built (UTC)   : $BUILD_DATE
Toolchain     : $TOOLCHAIN
Git branch    : $GIT_BRANCH
Git commit    : $GIT_COMMIT

This is an ALPHA pre-release demo image:
  * boots to the in-kernel text terminal (VGA/framebuffer + serial COM1)
  * runs the built-in self-test suite
  * expects a writable ATA disk for the TFS file-system test (see below)

Running the ISO in QEMU (mirrors kernel-bin/run.sh):

  tools/run-iso.sh                 # graphical window + serial on stdout
  tools/run-iso.sh --headless      # serial only, exits after the boot log

Writing the ISO to a USB stick (it carries a hybrid MBR):

  sudo dd if=$(basename "$OUT_ISO") of=/dev/sdX bs=4M status=progress oflag=sync

Burning to a CD/DVD: use any ISO9660/El Torito capable tool (UDF mode off).
EOF

log "release manifest: $DIST/RELEASE.txt"
log "checksum        : $OUT_ISO.sha256"
echo
echo "==============================================================="
echo " TrangorgeOS demo ISO ready:"
echo "   $(basename "$OUT_ISO")"
echo "   sha256: $(cut -d' ' -f1 < "$OUT_ISO.sha256")"
echo " boot it with: tools/run-iso.sh"
echo "==============================================================="
