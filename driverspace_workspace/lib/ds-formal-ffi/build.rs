use std::process::Command;
use std::path::PathBuf;

fn main() {
    // Ścieżka do projektu Ada SPARK
    let spark_project_dir = PathBuf::from("../../formal/ds-spark-core");
    
    println!("cargo:rerun-if-changed=../../formal/ds-spark-core/src");

    // Wywołanie gprbuild w trybie bare metal
    let status = Command::new("gprbuild")
        .arg("-P")
        .arg("ds_spark_core.gpr")
        .arg("-p") // Twórz katalogi jeśli nie istnieją
        .arg("--target=x86_64-unknown-none") // Lub inny target bare metal
        .current_dir(&spark_project_dir)
        .status()
        .expect("Failed to execute gprbuild. Is GNAT installed?");

    if !status.success() {
        panic!("gprbuild failed. Check SPARK contracts and Ada syntax.");
    }

    // Wskazanie linkerowi Rusta, gdzie szukać wygenerowanej biblioteki .a
    let lib_dir = spark_project_dir.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    
    // Linkowanie biblioteki statycznej (nazwa bez prefiksu 'lib' i rozszerzenia '.a')
    println!("cargo:rustc-link-lib=static=ds_spark_core");
}