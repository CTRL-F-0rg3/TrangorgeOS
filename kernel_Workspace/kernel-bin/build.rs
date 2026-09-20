// kernel_Workspace/kernel-bin/build.rs
//
// NOTE: `src/boot.asm` and `linker.ld` are deliberately NOT wired into the
// build. The `_start` of the bootimage comes from `bootloader::entry_point!`
// (src/main.rs) and the linker script is supplied by the `bootloader` crate
// itself; assembling the old asm entry as well would define the entry symbols
// twice and passing `-T linker.ld` would override the bootloader layout
// (`bootimage` then bails out with a build failure).
//
// The files are kept as reference for a future multiboot/long-mode bootstrap
// (for the day we drop bootloader 0.9).

fn main() {
    println!("cargo:rerun-if-changed=src/boot.asm");
    println!("cargo:rerun-if-changed=linker.ld");
}
