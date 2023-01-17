use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=phonon");
    println!("cargo:rustc-link-search=native=lib/{}", env::var("TARGET").unwrap());
    println!("cargo:rerun-if-changed=build.rs");

    let bindings = bindgen::Builder::default()
        .header("include/phonon.h")
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}
