#!/usr/bin/env bash
# TrangorgeOS — build a bootable driver image.
#
# The image carries every driver the machine *might* need, plus a manifest that
# says what each one claims to handle. At boot, `ds-detect` reads the manifest,
# scans the buses, and starts only the drivers that match hardware it actually
# finds. The image is therefore the same on every machine; what differs is the
# subset of drivers that comes up.
#
# Layout of the produced image:
#
#   <stage>/BOOT          raw kernel boot image (512-byte sectors)
#   <stage>/MANIFEST      TDXM driver manifest, read by ds-detect
#   <stage>/drivers/*     one file per driver, plus its sha256
#   <stage>/README.TXT    what the image contains, for `strings` and humans
#
# Why a manifest and not just the binaries: the loader has to answer "is there
# a driver for this (vendor, device)?" without loading anything, and "what may
# this driver be granted?" without trusting the driver itself. Both answers
# have to be available before the first driver runs, so they have to be data.
#
# Usage:
#   tools/mkdriverimg.sh [options]
#
# Options:
#   --no-build        reuse already-built driver binaries
#   --profile=P       cargo profile to build (default: debug)
#   --version=STRING  version recorded in the manifest and volume label
#   --out=FILE        output image (default: dist/TrangorgeOS-drivers.img)
#   --size=MB         image size in MiB (default: 16)
#   --list            print the manifest that would be built, then exit
#   -h | --help       this text
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WS="$REPO_ROOT/driverspace_workspace"
DIST="$REPO_ROOT/dist"
STAGE="$DIST/driverimg-root"
TARGET_DIR="$WS/target"

export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"

PROFILE="debug"
DO_BUILD=1
VERSION="${TRANGORGE_VERSION:-0.1.0}"
OUT_IMG=""
SIZE_MB=16
LIST_ONLY=0

log()  { printf '[mkdriverimg] %s\n' "$*" >&2; }
die()  { printf '[mkdriverimg] ERROR: %s\n' "$*" >&2; exit 1; }

usage() { sed -n '2,29p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; }

for arg in "$@"; do
    case "$arg" in
        --no-build)     DO_BUILD=0 ;;
        --profile=*)    PROFILE="${arg#--profile=}" ;;
        --version=*)    VERSION="${arg#--version=}" ;;
        --out=*)        OUT_IMG="${arg#--out=}" ;;
        --size=*)       SIZE_MB="${arg#--size=}" ;;
        --list)         LIST_ONLY=1 ;;
        -h|--help)      usage; exit 0 ;;
        *)              die "unknown option: $arg (try --help)" ;;
    esac
done

[ -n "$OUT_IMG" ] || OUT_IMG="$DIST/TrangorgeOS-drivers-$VERSION.img"

# --------------------------------------------------------------------------- #
# The catalogue: what the image will contain.
#
# This is the single source of truth for the build script and the runtime
# manifest. It is a plain text file so that adding a driver is a one-line
# change and the diff is reviewable. Format:
#
#   <driver-name> <driver-id> <match> <policy> [capabilities]
#
#   match       exact:VVVV:DDDD  - one (vendor, device) pair
#               class:CCCC       - every device of that class (CCSS or CC)
#   policy      ondemand | always | manual
#   capabilities  comma-separated CapId bit names, or `-` for none
#
# The driver ids match `crates/ds-registry/src/database.rs`; keeping them in
# step is what lets the manager bind a scanned device to a loaded driver.
# --------------------------------------------------------------------------- #
cat <<'CATALOGUE' > /tmp/tg-driver-catalog.$$
Alsa-TrangorgeOS  4  exact:8086:2415  ondemand  ALSA_ENUMERATE
Alsa-TrangorgeOS  4  exact:8086:2425  ondemand  ALSA_ENUMERATE
Alsa-TrangorgeOS  4  exact:1022:7807  ondemand  ALSA_ENUMERATE
NetTrangorgeOS    5  class:02         ondemand  -
IommuTrangorgeOS  6  class:08         always    IOMMU_ENUMERATE
VgaTrangorgeOS   10  class:03         ondemand  -
InputTrangorgeOS 7  class:09         ondemand  -
UsbTrangorgeOS   8  class:0C         ondemand  -
SataTrangorgeOS  9  class:01         ondemand  -
CATALOGUE
CATALOGUE_FILE=/tmp/tg-driver-catalog.$$

if [ "$LIST_ONLY" = "1" ]; then
    echo "Driver catalogue for TrangorgeOS $VERSION:"
    echo
    printf '%-20s %-4s %-18s %-9s %s\n' NAME ID MATCH POLICY CAPS
    while read -r name id match policy caps; do
        [ -n "$name" ] || continue
        printf '%-20s %-4s %-18s %-9s %s\n' "$name" "$id" "$match" "$policy" "$caps"
    done < "$CATALOGUE_FILE"
    echo
    echo "Start rule: ondemand entries run only when a scanned device matches."
    rm -f "$CATALOGUE_FILE"
    exit 0
fi

# --------------------------------------------------------------------------- #
# 1. Build the driver binaries.
# --------------------------------------------------------------------------- #
CARGO_FLAG=()
[ "$PROFILE" = "release" ] && CARGO_FLAG=(--release)

build_driver() {
    local name="$1" path="$2"
    log "building $name"
    ( cd "$WS" && cargo build "${CARGO_FLAG[@]}" -p "$path" --bins ) \
        || die "failed to build $name ($path)"
}

if [ "$DO_BUILD" = "1" ]; then
    build_driver "Alsa-TrangorgeOS" "alsa-trangorgeos"
    build_driver "IommuTrangorgeOS" "iommu-driver"
fi

# --------------------------------------------------------------------------- #
# 2. Stage the tree.
# --------------------------------------------------------------------------- #
log "staging into $STAGE"
rm -rf "$STAGE"
mkdir -p "$STAGE/drivers"

# Locate a built binary by *crate* name, whichever profile produced it.
#
# The catalogue names a driver, the Cargo manifest names a crate, and the two
# are not always the same (`Alsa-TrangorgeOS` the directory builds the
# `alsa-trangorgeos` binary). This function takes the binary name and tries
# both spellings, because a manifest entry that cannot find its binary is
# exactly the silent failure this image is supposed to avoid.
find_binary() {
    local name="$1"
    local lower
    lower="$(printf '%s' "$name" | tr '[:upper:]' '[:lower:]')"
    for dir in "$TARGET_DIR/$PROFILE" "$TARGET_DIR/debug" "$TARGET_DIR/release"; do
        [ -d "$dir" ] || continue
        for candidate in "$dir/$name" "$dir/$lower"; do
            [ -f "$candidate" ] && { printf '%s' "$candidate"; return 0; }
        done
    done
    return 1
}

# The boot image, when one has been built. It is optional: a driver image with
# no kernel is still a valid catalogue, and is what `--no-build` after a plain
# `cargo build` produces.
BOOT_SRC="$REPO_ROOT/kernel_Workspace/kernel-bin/bootimage-kernel-bin.bin"
if [ -f "$BOOT_SRC" ]; then
    cp "$BOOT_SRC" "$STAGE/BOOT"
    log "boot image: $(stat -c '%s' "$STAGE/BOOT") bytes"
else
    log "no kernel boot image found; building a catalogue-only image"
fi

# Copy each driver named in the catalogue.
declare -A COPIED=()
while read -r name id match policy caps; do
    [ -n "$name" ] || continue
    [ -n "${COPIED[$name]:-}" ] && continue

    if binary="$(find_binary "$name")"; then
        cp "$binary" "$STAGE/drivers/$name"
        ( cd "$STAGE/drivers" && sha256sum "$name" > "$name.sha256" )
        COPIED[$name]=1
        log "staged driver: $name ($(stat -c '%s' "$STAGE/drivers/$name") bytes)"
    else
        log "WARNING: no binary for $name; the manifest will list it anyway"
    fi
done < "$CATALOGUE_FILE"

# --------------------------------------------------------------------------- #
# 3. Generate the manifest.
#
# The format is a text rendering of `ds_detect::manifest::Manifest`: a header
# line, then one tab-separated line per entry, then a sentinel. It is text
# because the loader reads it before anything else is loaded, and a format that
# `cat` and `diff` understand is a format that can be reviewed in a pull
# request. `ds-detect` parses exactly this.
# --------------------------------------------------------------------------- #
MANIFEST="$STAGE/MANIFEST"
{
    echo "# TrangorgeOS driver manifest"
    echo "# TDXM 1  version=$VERSION  entries=$(grep -cve '^\s*$' "$CATALOGUE_FILE")"
    echo "# name<TAB>driver_id<TAB>match<TAB>policy<TAB>capabilities"
    while read -r name id match policy caps; do
        [ -n "$name" ] || continue
        printf '%s\t%s\t%s\t%s\t%s\n' "$name" "$id" "$match" "$policy" "$caps"
    done < "$CATALOGUE_FILE"
    echo "END"
} > "$MANIFEST"
log "manifest: $(grep -c $'\t' "$MANIFEST") entries"

# --------------------------------------------------------------------------- #
# 4. Human-readable description, for `strings` on the image and for anyone
#    debugging a boot that did not start what they expected.
# --------------------------------------------------------------------------- #
cat > "$STAGE/README.TXT" <<EOF
TrangorgeOS driver image
========================
Version   : $VERSION
Built     : $(date -u '+%Y-%m-%dT%H:%M:%SZ')
Profile   : $PROFILE

This image carries every driver the machine might need. At boot, ds-detect
reads MANIFEST, scans the buses, and starts only the drivers whose match rule
covers a device it actually found. Drivers for hardware that is absent are
carried but never loaded.

Catalogue
---------
$(while read -r name id match policy caps; do
    [ -n "$name" ] || continue
    printf '  %-20s id=%-3s %-18s %s\n' "$name" "$id" "$match" "$policy"
done < "$CATALOGUE_FILE")

Match rules
-----------
  exact:VVVV:DDDD  only that (vendor, device) pair
  class:CCCC       every device of that class; CCCC is CC or CCSS in hex
  policy=always    start regardless of what was found (bus controllers)
  policy=manual    carry it, never start it automatically

Contents
--------
  BOOT           kernel boot image, if one was built
  MANIFEST       the catalogue, read before any driver is loaded
  drivers/*      the driver binaries
  drivers/*.sha256  per-driver checksums
EOF

# --------------------------------------------------------------------------- #
# 5. Compose the image.
#
# A raw, MBR-partitioned disk image: the same 512-byte-sector artefact the BIOS
# and QEMU already boot, so the detection layer runs on the same path the rest
# of the system uses rather than on a special one.
# --------------------------------------------------------------------------- #
mkdir -p "$DIST"
SIZE_BYTES=$((SIZE_MB * 1024 * 1024))
log "composing ${SIZE_MB} MiB image at $OUT_IMG"

# Start from a copy of the boot image so the kernel and its partition table
# survive intact; fall back to a zeroed image when there is no kernel.
if [ -f "$STAGE/BOOT" ]; then
    BOOT_SIZE=$(stat -c '%s' "$STAGE/BOOT")
    if [ "$BOOT_SIZE" -gt "$SIZE_BYTES" ]; then
        SIZE_MB=$(( (BOOT_SIZE / 1024 / 1024) + 8 ))
        SIZE_BYTES=$((SIZE_MB * 1024 * 1024))
        log "boot image needs $BOOT_SIZE bytes; growing the image to ${SIZE_MB} MiB"
    fi
    dd if="$STAGE/BOOT" of="$OUT_IMG" bs=1M conv=notrunc status=none
else
    dd if=/dev/zero of="$OUT_IMG" bs=1M count="$SIZE_MB" status=none
fi

# Append the staged tree as a FAT16 partition. FAT16 rather than ext2 because
# the boot code reads it through the BIOS INT 13h extensions, which every BIOS
# implements for FAT but not for ext.
if ! command -v mkfs.fat >/dev/null 2>&1; then
    die "mkfs.fat not found (package dosfstools); cannot create the driver partition"
fi

# Build the partition image separately, then append it at a 1 MiB boundary.
#
# FAT16 has a hard minimum of 32 MiB - `mkfs.fat` refuses anything smaller -
# and the partition is two thirds of the image, so an image below 48 MiB would
# produce a partition that cannot be formatted. `MIN_IMAGE_MB` encodes that.
MIN_IMAGE_MB=48
PART_MIN_MB=32

if [ "$SIZE_MB" -lt "$MIN_IMAGE_MB" ]; then
    log "raising the image from ${SIZE_MB} to ${MIN_IMAGE_MB} MiB (FAT16 needs a 32 MiB partition)"
    SIZE_MB=$MIN_IMAGE_MB
    SIZE_BYTES=$((SIZE_MB * 1024 * 1024))
fi

# Two thirds of the image, rounded down to whole MiB.
PART_MB=$((SIZE_MB * 2 / 3))
[ "$PART_MB" -lt "$PART_MIN_MB" ] && PART_MB=$PART_MIN_MB

PART_IMG="$(mktemp -u /tmp/tg-driverpart.XXXXXX)"
trap 'rm -f "$PART_IMG" "$CATALOGUE_FILE"' EXIT

dd if=/dev/zero of="$PART_IMG" bs=1M count="$PART_MB" status=none
mkfs.fat -F 16 -n "TGDRIVERS" "$PART_IMG" >/dev/null 2>&1 \
    || die "mkfs.fat failed on the staging partition"

# Populate it: mcopy is part of mtools. This happens *after* the layout files
# are written, because they live in the partition too.
if ! command -v mcopy >/dev/null 2>&1; then
    die "mcopy not found (package mtools); cannot populate the driver partition"
fi

# Append the partition at the first 1 MiB boundary after the boot image, and
# record where it starts so the boot code can find it without a filesystem.
#
# The offset is a plain number in `PARTITION` next to the manifest, because the
# loader reads the catalogue before it has mounted anything.
BOOT_END=0
if [ -f "$STAGE/BOOT" ]; then
    BOOT_END=$(stat -c '%s' "$STAGE/BOOT")
fi
# Round up to the next 1 MiB, because a partition that did not start on a
# 1 MiB boundary would be truncated by the 2 MiB LBA alignment the BIOS uses.
PART_OFFSET=$(( (BOOT_END + 0x100000 - 1) / 0x100000 * 0x100000 ))

# Without a boot image there is nothing at sector 0, and a filesystem written
# there would be what the BIOS looks for when it reads the MBR - so the
# partition still starts at 1 MiB. `truncate` zero-fills the gap.
if [ "$PART_OFFSET" -eq 0 ]; then
    PART_OFFSET=$(( 0x100000 ))
    log "no boot image: placing the driver partition at 1 MiB"
fi
PART_BYTES=$(stat -c '%s' "$PART_IMG")

# --------------------------------------------------------------------------- #
# 6. Populate the partition *before* it is written into the image.
# --------------------------------------------------------------------------- #
# Order matters: `mcopy` mutates the partition image, so it has to run before
# the `dd` below. Copying it the other way round produces an image whose
# filesystem is empty - and an empty filesystem that still passes a naive
# "does the partition exist" check.
mcopy -s -i "$PART_IMG" "$STAGE"/* :: 2>/dev/null \
    || die "mcopy failed; the driver tree did not fit"

# Record the layout, in sectors, which is the unit INT 13h uses. The numbers are
# decimal because the boot code has no hex parser yet. It is written after the
# tree copy above and then copied in on its own, because it is the one file
# whose contents depend on where the partition ended up.
{
    echo "TRANGORGE_PARTITION"
    echo "offset_bytes=$PART_OFFSET"
    echo "offset_sectors=$((PART_OFFSET / 512))"
    echo "size_bytes=$PART_BYTES"
    echo "manifest_path=/MANIFEST"
} > "$STAGE/PARTITION"
mcopy -i "$PART_IMG" "$STAGE/PARTITION" :: 2>/dev/null \
    || die "could not write the partition layout into the partition"

# Grow the image if the partition does not fit, then place it.
NEEDED_BYTES=$((PART_OFFSET + PART_BYTES))
if [ "$NEEDED_BYTES" -gt "$SIZE_BYTES" ]; then
    SIZE_BYTES=$(( (NEEDED_BYTES + 0x100000 - 1) / 0x100000 * 0x100000 ))
    SIZE_MB=$((SIZE_BYTES / 1024 / 1024))
    log "driver partition needs $NEEDED_BYTES bytes; image grown to ${SIZE_MB} MiB"
fi

# The image is truncated only now, after every size is known: doing it earlier
# and then writing past the end would silently zero the tail.
truncate -s "$SIZE_BYTES" "$OUT_IMG"
dd if="$PART_IMG" of="$OUT_IMG" bs=512 seek=$((PART_OFFSET / 512)) conv=notrunc status=none

# --------------------------------------------------------------------------- #
# 7. Verify the artefacts actually landed.
# --------------------------------------------------------------------------- #
# A manifest that is on the host but not in the image is the failure mode this
# check exists to catch.
mtype -i "$PART_IMG" ::/MANIFEST >/dev/null 2>&1 \
    || die "MANIFEST is missing from the driver partition"
for name in "${!COPIED[@]}"; do
    mtype -i "$PART_IMG" "::/drivers/$name" >/dev/null 2>&1 \
        || die "driver $name is missing from the driver partition"
done
log "verified: manifest and ${#COPIED[@]} driver(s) present in the partition"

# Finally, read the partition back *out of the finished image*. Every check
# above looked at the staging file, which is not the same thing: the partition
# can be correct and still be missing from the image, or be at the wrong
# offset, and only this catches it.
CHECK_IMG="$(mktemp -u /tmp/tg-drivercheck.XXXXXX)"
if ! dd if="$OUT_IMG" of="$CHECK_IMG" bs=512 skip=$((PART_OFFSET / 512)) \
        count=$((PART_BYTES / 512)) status=none; then
    die "could not read the driver partition back out of the image"
fi
mtype -i "$CHECK_IMG" ::/MANIFEST >/dev/null 2>&1 \
    || die "the image does not contain a readable driver partition at offset $PART_OFFSET"
log "verified: partition readable from the image at offset $PART_OFFSET"
rm -f "$CHECK_IMG"

# The partition must be in the image, or the manifest is unreadable at boot.
if ! dd if="$OUT_IMG" bs=512 skip=$((PART_OFFSET / 512)) count=1 status=none \
        | cmp -s - <(dd if="$PART_IMG" bs=512 count=1 status=none); then
    die "the driver partition is not where PARTITION says it is"
fi
log "verified: partition present at offset $PART_OFFSET"

# --------------------------------------------------------------------------- #
# 7. Publication artefacts.
# --------------------------------------------------------------------------- #
( cd "$(dirname "$OUT_IMG")" && sha256sum "$(basename "$OUT_IMG")" > "$(basename "$OUT_IMG").sha256" )

log "image: $OUT_IMG ($(stat -c '%s' "$OUT_IMG") bytes)"
log "sha256: $(cut -d' ' -f1 < "$OUT_IMG.sha256")"
log "staging tree kept at $STAGE (inspect it with: cat $STAGE/MANIFEST)"

cat <<EOF

===============================================================
 TrangorgeOS driver image ready
   image    : $OUT_IMG
   manifest : $STAGE/MANIFEST ($(grep -c $'\t' "$STAGE/MANIFEST") entries)
   drivers  : ${#COPIED[@]} staged
   part     : offset $PART_OFFSET (sector $((PART_OFFSET / 512)))

 At boot ds-detect reads MANIFEST, scans the buses and starts only the
 drivers whose match rule covers hardware it found. A machine with
 one sound card starts one audio driver out of however many the
 image carries.
===============================================================
EOF
