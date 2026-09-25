# ISO builder for TrangorgeOS demo images.
#
# Everything the ISO needs on top of the Rust toolchain: xorriso (El Torito) and
# syslinux-common (isolinux.bin, ldlinux.c32, memdisk, isohdpfx.bin).
#
# Build:
#   docker build -t trangorgeos-iso -f tools/iso/Dockerfile .
# Run (from the repository root):
#   docker run --rm -v "$PWD:/src" -w /src trangorgeos-iso tools/mkiso.sh
#   docker run --rm -v "$PWD:/src" -w /src trangorgeos-iso tools/mkiso.sh --release
#
# The result lands in ./dist/ on the host.

FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    build-essential \
    clang \
    lld \
    llvm \
    xorriso \
    syslinux-common \
    nasm \
    git \
    curl \
    ca-certificates \
    python3 \
    && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none

# Matches the toolchain in rust-toolchain.toml / kernel/rust-toolchain.toml.
# `rustup show` resolves the pinned nightly and installs the required components
# (rust-src, llvm-tools-preview), which the bootimage build needs for -Zbuild-std.
WORKDIR /src
COPY rust-toolchain.toml .
RUN rustup toolchain install nightly-2026-08-01 \
    --component rust-src,llvm-tools-preview \
    --profile minimal \
    && cargo install bootimage --version "^0.10" --locked

CMD ["tools/mkiso.sh"]
