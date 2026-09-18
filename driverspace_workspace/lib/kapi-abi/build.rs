//! Build script dla `kapi-abi`.
//!
//! Crate jest w 100% `no_std` i nie generuje kodu — skrypt istnieje tylko po to,
//! żeby Cargo miało poprawny entry point (pusty `build.rs` powoduje błąd
//! "`main` function not found in crate `build_script_build`").
//!
//! Gdy pojawi się generowanie nagłówków C/Odin z typów ABI, kod trafi tutaj.

fn main() {
    println!("cargo:rerun-if-changed=src");
}

