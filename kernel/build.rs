use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn collect_c_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_c_files(&path, out);
            } else if matches!(path.extension().and_then(|e| e.to_str()), Some("c") | Some("s")) {
                out.push(path);
            }
        }
    }
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let src_dir = Path::new(&manifest_dir).join("src");

    println!("cargo:rerun-if-changed=src");

    let mut c_files = Vec::new();
    collect_c_files(&src_dir, &mut c_files);
    c_files.sort();

    let target = "x86_64-unknown-none-elf";

    let mut objs = Vec::new();
    for c_file in &c_files {
        let rel = c_file.strip_prefix(&src_dir).expect("strip prefix");
        let obj_name = rel.to_string_lossy().replace('/', "_") + ".o";
        let obj = Path::new(&out_dir).join(obj_name);

        let status = Command::new("clang")
            .arg(format!("--target={}", target))
            .args([
                "-ffreestanding",
                "-fno-builtin",
                "-fno-stack-protector",
                "-mno-red-zone",
                "-mcmodel=kernel",
                "-mno-sse",
                "-mno-sse2",
                "-mno-mmx",
                "-mno-80387",
                "-std=gnu11",
                "-O2",
            ])
            .arg("-I")
            .arg(&src_dir)
            .arg("-c")
            .arg(c_file)
            .arg("-o")
            .arg(&obj)
            .status()
            .unwrap_or_else(|e| panic!("failed to run clang for {}: {}", c_file.display(), e));

        if !status.success() {
            panic!("C compilation failed for {}", c_file.display());
        }
        objs.push(obj);
    }

    let lib = Path::new(&out_dir).join("libmm.a");
    let _ = fs::remove_file(&lib);

    let mut ar = Command::new("ar");
    ar.arg("rcs").arg(&lib);
    for o in &objs {
        ar.arg(o);
    }
    let status = ar.status().expect("failed to run ar");
    if !status.success() {
        panic!("ar failed");
    }

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=mm");
}
