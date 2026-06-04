use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.join("../..");
    let libyuv_source_dir = repo_root.join("external/libyuv/source");
    let libyuv_include_dir = repo_root.join("external/libyuv/include");

    if !libyuv_source_dir.exists() {
        panic!("external/libyuv/source not found; initialize the libyuv submodule");
    }

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .include(&libyuv_include_dir)
        .file(manifest_dir.join("cpp/libyuv_wrapper.cpp"))
        .flag_if_supported("-std=c++17");

    if env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("aarch64") {
        // GB8's GCC/binutils reject some libyuv AArch64 NEON inline assembly.
        // Prefer portable C fallbacks over failing to build the backend.
        build.define("LIBYUV_DISABLE_NEON", None);
    }

    let mut sources: Vec<_> = fs::read_dir(&libyuv_source_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cc"))
        .collect();
    sources.sort();

    for source in sources {
        build.file(source);
    }

    build.compile("pyvirtualcam_libyuv");

    println!("cargo:rerun-if-changed={}", libyuv_source_dir.display());
    println!("cargo:rerun-if-changed={}", libyuv_include_dir.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("cpp/libyuv_wrapper.cpp").display()
    );
}
