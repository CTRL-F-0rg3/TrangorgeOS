#!/bin/sh
set -e
cd "$(dirname "$0")"

cargo build --release

# deploy do obrazu FS / initrd
cp init/target/x86_64-unknown-none/release/tr-init \
   ../iso-root/bin/init.elf

echo "init.elf deployed"