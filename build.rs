use std::{env, path::PathBuf};

fn main() {
    let in_dir = PathBuf::from(env::var("STEAMAUDIO_DIR").expect("STEAMAUDIO_DIR is not defined"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindgen::Builder::default()
        .header(in_dir.join("include/phonon.h").to_str().unwrap())
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Failed to write bindings");

    println!("cargo:rustc-link-lib=phonon");
    println!(
        "cargo:rustc-link-search={}/lib/{}",
        in_dir.to_str().unwrap(),
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
