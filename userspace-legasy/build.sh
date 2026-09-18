#!/bin/sh
set -e
cd "$(dirname "$0")"

# Katalog docelowy dla obrazow userspace (mozna nadpisac pierwszym argumentem).
OUT="${1:-../iso-root/bin}"

cargo build --release

# To jest workspace, wiec artefakty leza w target/<cel>/release.
TARGET_DIR="target/x86_64-unknown-none/release"
if [ ! -d "$TARGET_DIR" ]; then
    TARGET_DIR="target/release"
fi

mkdir -p "$OUT"

for app in init shell demo terminal; do
    cp "$TARGET_DIR/$app" "$OUT/$app.elf"
done

echo "userspace deployed: init shell demo terminal -> $OUT"