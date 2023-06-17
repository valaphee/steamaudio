use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=phonon");
    println!(
        "cargo:rustc-link-search=lib/{}",
        match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
            "android" => match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
                "x86" => "android-x86",
                "x86_64" => "android-x64",
                value => unimplemented!("Unsupported architecture: {}", value),
            }
            "linux" => match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
                "x86" => "linux-x86",
                "x86_64" => "linux-x64",
                value => unimplemented!("Unsupported architecture: {}", value),
            }
            "windows" => match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
                "x86" => "windows-x86",
                "x86_64" => "windows-x64",
                value => unimplemented!("Unsupported architecture: {}", value),
            }
            value => unimplemented!("Unsupported operating system: {}", value),
        }
    );
    println!("cargo:rerun-if-changed=build.rs");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::Builder::default()
        .header("include/phonon.h")
        .generate()
        .unwrap()
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}
