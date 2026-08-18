use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let odin_dir = manifest.join("odin");
    let lib_name = "audiodriver_odin";
    let lib_path = out.join(format!("lib{}.a", lib_name));

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", odin_dir.display());

    let odin = env::var("ODIN").unwrap_or_else(|_| "odin".to_string());

    let status = Command::new(&odin)
        .arg("build")
        .arg(odin_dir.to_str().unwrap())
        .arg("-build-mode:static-lib")
        .arg("-target:freestanding_x64")
        .arg("-no-crt")
        .arg("-o:speed")
        .arg(format!("-out:{}", lib_path.to_str().unwrap()))
        .status()
        .expect("nie znaleziono `odin` w PATH (ustaw ODIN=/sciezka/odin)");

    if !status.success() {
        panic!("odin build failed");
    }

    if !lib_path.exists() {
        panic!("odin nie wyprodukowal {}", lib_path.display());
    }

    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static={}", lib_name);
}