fn main() {
    let version = rustc_version::version().unwrap();
    println!("cargo:rustc-env=RUSTC_VERSION={}", version);

    // Link to static library if `modular` is not enabled.
    println!("cargo:rerun-if-changed=build.rs");
}
