
fn main() {
    nasm_rs::compile_library("boot.asm", &["src/boot.asm"]).unwrap();

    println!("cargo:rustc-link-arg=-Tcomgrub/linker.ld");
    println!("cargo:rustc-link-lib=static=boot");

    println!("cargo:rerun-if-changed=src/boot.asm");
    println!("cargo:rerun-if-changed=linker.ld");
}