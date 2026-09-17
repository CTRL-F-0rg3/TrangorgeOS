fn main() {
    nasm_rs::compile_library("boot.asm", &["src/boot.asm"]).unwrap();

    // Ścieżka bezwzględna: `cargo:rustc-link-arg` przekazuje argument do linkera
    // w katalogu docelowym, więc relatywna ścieżka by się nie rozwiązała.
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo:rustc-link-arg=-T{}/linker.ld", manifest);
    println!("cargo:rustc-link-lib=static=boot");

    println!("cargo:rerun-if-changed=src/boot.asm");
    println!("cargo:rerun-if-changed=linker.ld");
}