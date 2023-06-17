use std::{env, path::PathBuf};

fn main() {
    bindgen::Builder::default()
        .header("include/phonon.h")
        .generate()
        .unwrap()
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .unwrap();

    println!("cargo:rustc-link-lib=phonon");
    println!(
        "cargo:rustc-link-search=lib/{}",
        match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
            "android" => match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
                "x86" => "android-x86",
                "x86_64" => "android-x64",
                value => unimplemented!("Unsupported architecture: {}", value),
            },
            "linux" => match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
                "x86" => "linux-x86",
                "x86_64" => "linux-x64",
                value => unimplemented!("Unsupported architecture: {}", value),
            },
            "windows" => match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
                "x86" => "windows-x86",
                "x86_64" => "windows-x64",
                value => unimplemented!("Unsupported architecture: {}", value),
            },
            value => unimplemented!("Unsupported operating system: {}", value),
        }
    );
}
