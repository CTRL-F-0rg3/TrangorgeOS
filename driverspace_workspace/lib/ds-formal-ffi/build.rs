// Build script dla Ada/SPARK FFI.
//
// Formalna część `ds-spark-core` jest budowana przez GNAT (`gprbuild`).
// GNAT jest zależnością opcjonalną: bez niego `cargo check` ma nadal działać
// (tylko bez natywnej biblioteki SPARK), zamiast wywalać cały build panicem.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Ścieżka do projektu Ada SPARK
    let spark_project_dir = PathBuf::from("../../formal/ds-spark-core");

    println!("cargo:rerun-if-changed=../../formal/ds-spark-core/src");

    let gprbuild_available = Command::new("gprbuild")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !gprbuild_available {
        println!(
            "cargo:warning=nie znaleziono `gprbuild` (zainstaluj GNAT/SPARK); \
             pomijam budowanie formalnej biblioteki ds-spark-core"
        );
        return;
    }

    // Wywołanie gprbuild w trybie bare metal
    let status = Command::new("gprbuild")
        .arg("-P")
        .arg("ds_spark_core.gpr")
        .arg("-p") // Twórz katalogi jeśli nie istnieją
        .arg("--target=x86_64-unknown-none") // Lub inny target bare metal
        .current_dir(&spark_project_dir)
        .status()
        .expect("nie udalo sie uruchomic `gprbuild`");

    if !status.success() {
        panic!("gprbuild failed. Check SPARK contracts and Ada syntax.");
    }

    // Wskazanie linkerowi Rusta, gdzie szukać wygenerowanej biblioteki .a
    let lib_dir = spark_project_dir.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // Linkowanie biblioteki statycznej (nazwa bez prefiksu 'lib' i rozszerzenia '.a')
    println!("cargo:rustc-link-lib=static=ds_spark_core");
}