#!/bin/sh
set -e
cd "$(dirname "$0")"

cargo build --release

for app in init shell demo; do
    cp $app/target/x86_64-unknown-none/release/$app \
       ../iso-root/bin/$app.elf
done

echo "userspace deployed: init shell demo"