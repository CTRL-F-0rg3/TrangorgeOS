#!/usr/bin/env bash
# Build a TrangorgeOS userspace binary against the real Rust standard library.
#
#   ./build.sh              # build tgs-init for the userspace target
#   ./build.sh --release    # optimised
#   ./build.sh --check      # type-check only, for CI
#   ./build.sh --test       # run tgs-hal's host tests (no target needed)
#
# The one thing this script exists to do is pass `-Zbuild-std`, because there is no
# prebuilt `std` for `x86_64-trangorge-uspace.json` and Cargo will not link one by
# itself. Everything else is here so that the flags in one place and in the
# committed script rather than in whatever a contributor's shell history had.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$HERE"

# rustup's cargo may not be on a non-login shell's PATH.
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"

TARGET_JSON="$HERE/targets/x86_64-trangorge-uspace.json"
LINKER_LD="$HERE/linker.ld"
PROFILE_ARGS=()
CHECK=0
RUN_TESTS=0

for arg in "$@"; do
    case "$arg" in
        --release) PROFILE_ARGS=(--release) ;;
        --check)   CHECK=1 ;;
        --test)    RUN_TESTS=1 ;;
        -h|--help) sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "build.sh: unknown option $arg" >&2; exit 2 ;;
    esac
done

if [[ ! -f "$TARGET_JSON" ]]; then
    echo "build.sh: missing target spec $TARGET_JSON" >&2
    exit 1
fi
if [[ ! -f "$LINKER_LD" ]]; then
    echo "build.sh: missing linker script $LINKER_LD" >&2
    exit 1
fi

# The host tests need no target and no `std`: `tgs-hal` is a plain `no_std` crate
# with a mockable syscall gate, and that is the whole point of it.
if [[ "$RUN_TESTS" == 1 ]]; then
    exec cargo test --manifest-path "$HERE/Cargo.toml" -p tgs-hal "$@"
fi

# `-Zbuild-std=std,panic_abort`:
#   std          the point of the exercise
#   panic_abort  the kernel has no unwinder and no `setjmp` support, so a panic
#                has to be a process exit rather than a stack unwind. `build-std`
#                does not pick this up from the profile.
STD_FLAGS=(
    # Cargo refuses a `.json` target spec without this, and says so with an error
    # that reads as if the problem were somewhere else entirely.
    -Zjson-target-spec
    -Zbuild-std=std,panic_abort
    # `std`'s own memcpy/memset come from compiler-builtins, which
    # `-Zbuild-std-features` brings in; the flag also satisfies the
    # "which `mem` implementation" question `std` asks on a bare target.
    -Zbuild-std-features=compiler-builtins-mem
    # `std` for an unknown target reaches for `_Unwind_Backtrace` when it prints a
    # backtrace. With `panic_abort` those paths are dead, but the symbols are
    # still referenced from the backtrace code, so they are provided rather than
    # left to fail at link time. See `README.md` under "known gaps".
)

# A bare ELF: no dynamic loader, no crt1.o, no libc.so. `-nostdlib` keeps the
# linker from looking for any of them, and the script is the only thing that
# supplies `_start`.
#
# These go through `RUSTFLAGS` rather than after a `--`. Cargo's argument
# parser for `check` and `build` does not reliably forward a trailing `--`
# argument list to rustc, and `RUSTFLAGS` is the supported way to say the same
# thing; it also means the flags reach every crate in the graph, which is what a
# link argument needs.
export RUSTFLAGS="\
-C link-arg=-nostdlib \
-C link-arg=-static \
-C link-arg=-znoexecstack \
-C link-arg=-T$LINKER_LD \
-C relocation-model=static \
-C link-arg=--gc-sections"

# `-Z` flags belong to *cargo* rather than rustc, so they go on the cargo command
# line and nowhere else. Cargo rejects one after a `--` outright.
PKG=(
    --manifest-path "$HERE/Cargo.toml"
    --target "$TARGET_JSON"
    -p tgs-init
    "${STD_FLAGS[@]}"
    "${PROFILE_ARGS[@]}"
)

if [[ "$CHECK" == 1 ]]; then
    exec cargo check "${PKG[@]}"
fi

cargo build "${PKG[@]}"

PROFILE=debug
[[ "${PROFILE_ARGS[*]:-}" == *release* ]] && PROFILE=release
OUT="$HERE/target/x86_64-trangorge-uspace/$PROFILE/tgs-init"

echo
echo "built $OUT"
if command -v file >/dev/null 2>&1; then
    file "$OUT"
fi
echo
echo "copy it into the image as /bin/init.elf and boot; see ../README.md"
