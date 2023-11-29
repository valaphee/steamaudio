use std::{env, path::PathBuf};

#[tokio::main]
async fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let zip_file = reqwest::get("https://github.com/ValveSoftware/steam-audio/releases/download/v4.4.1/steamaudio_4.4.1.zip").await.unwrap().bytes().await.unwrap().to_vec();
    zip_extract::extract(Cursor::new(&zip_file), &out_dir, true).unwrap();

    bindgen::Builder::default()
        .header(out_dir.join("include/phonon.h").to_str().unwrap())
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("bindings.rs"))
        .unwrap();

    println!("cargo:rustc-link-lib=phonon");
    println!(
        "cargo:rustc-link-search={}/lib/{}",
        out_dir.to_str().unwrap(),
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
