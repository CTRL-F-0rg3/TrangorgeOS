// comlimine/build.rs
fn main() {
    // Kompiluj C
    cc::Build::new()
        .file("src/main.c")
        .include("src")
        .no_default_flags(true)
        .flag("-ffreestanding")
        .flag("-nostdlib")
        .flag("-mno-red-zone")
        .flag("-mno-mmx")
        .flag("-mno-sse")
        .flag("-mno-sse2")
        .compile("limine_entry");

    // Linker script
    println!("cargo:rustc-link-arg=-Tcomlimine/linker.ld");
    println!("cargo:rerun-if-changed=src/main.c");
    println!("cargo:rerun-if-changed=linker.ld");
}