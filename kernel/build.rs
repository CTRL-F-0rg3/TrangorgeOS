use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn collect_c_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str());.
                if matches!(name, Some("linuxcom") | Some("wincom") | Some("aarch64") | Some("arm64") | Some("arm") | Some("risc-v") | Some("riscv")) {
                    continue;
                }
                collect_c_files(&path, out);
            } else if matches!(path.extension().and_then(|e| e.to_str()), Some("c") | Some("s")) {
                out.push(path);
            }
        }
    }
}

fn clang_target(arch: &str) -> &'static str {
    match arch {
        "x86_64" => "x86_64-unknown-none-elf",
        "riscv64" => "riscv64gc-unknown-none-elf",
        other => panic!(
            "cannot build the C bridge for unsupported target architecture: {other}"
        ),
    }
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let src_dir = Path::new(&manifest_dir).join("src");

    println!("cargo:rerun-if-changed=src");

    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "x86_64".to_string());

    match arch.as_str() {
        "x86_64" => {
            let llvm_target = clang_target("x86_64");
            let is_test = env::var("PROFILE").map(|p| p == "test").unwrap_or(false);

            let mut c_files = Vec::new();
            collect_c_files(&src_dir, &mut c_files);
            c_files.sort();

            let mut objs = Vec::new();
            for c_file in &c_files {
                let rel = c_file.strip_prefix(&src_dir).expect("strip prefix");
                let obj_name = rel.to_string_lossy().replace('/', "_") + ".o";
                let obj = Path::new(&out_dir).join(obj_name);

                let mut cmd = Command::new("clang");
                cmd.arg(format!("--target={}", llvm_target))
                    .args([
                        "-ffreestanding",
                        "-fno-builtin",
                        "-fno-stack-protector",
                        "-std=gnu11",
                    ]);
                if is_test {
                    cmd.arg("-fPIC").arg("-O1");
                } else {
                    cmd.args([
                        "-mno-red-zone",
                        "-mcmodel=kernel",
                        "-mno-sse",
                        "-mno-sse2",
                        "-mno-mmx",
                        "-mno-80387",
                        "-O2",
                    ]);
                }
                let status = cmd
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
        "riscv64" => {
            println!("cargo:rustc-link-arg=-T{}", manifest_dir + "/riscv64-link.ld");
        }

        other => panic!("build.rs has no C-bridge configuration for architecture: {other}"),
    }
}
