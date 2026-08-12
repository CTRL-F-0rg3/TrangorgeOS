#!/bin/bash
echo "🚀 Uruchamianie TrangorgeOS w QEMU (przez Dockera)..."
docker run --rm -it \
    --device /dev/kvm \
    -v "$(pwd)":/build \
    trangorgeos-build \
    cargo run