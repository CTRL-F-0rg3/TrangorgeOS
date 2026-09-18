// Build script dla sterownika audio.
//
// Część logiki sterownika jest napisana w Odinie (`src/odin`) i kompilowana do
// statycznej biblioteki, którą linkujemy do crate'a. Kompilator `odin` jest
// zależnością opcjonalną: bez niego `cargo check` ma nadal działać (tylko bez
// natywnej części Odin), zamiast wywalać cały build panicem.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let odin_dir = manifest.join("src/odin");
    let lib_name = "audiodriver_odin";
    let lib_path = out.join(format!("lib{}.a", lib_name));

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", odin_dir.display());

    let odin = env::var("ODIN").unwrap_or_else(|_| "odin".to_string());

    let odin_available = Command::new(&odin)
        .arg("version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !odin_available {
        println!(
            "cargo:warning=nie znaleziono kompilatora `odin` (ustaw ODIN=/sciezka/odin); \
             pomijam budowanie natywnej czesci sterownika audio"
        );
        return;
    }

    let status = Command::new(&odin)
        .arg("build")
        .arg(odin_dir.to_str().unwrap())
        .arg("-build-mode:static-lib")
        .arg("-target:freestanding_x64")
        .arg("-no-crt")
        .arg("-o:speed")
        .arg(format!("-out:{}", lib_path.to_str().unwrap()))
        .status()
        .expect("nie udalo sie uruchomic `odin`");

    if !status.success() {
        panic!("odin build failed");
    }

    if !lib_path.exists() {
        panic!("odin nie wyprodukowal {}", lib_path.display());
    }

    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static={}", lib_name);
}