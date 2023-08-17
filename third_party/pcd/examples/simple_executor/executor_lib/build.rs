fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let version = rustc_version::version().unwrap();
    println!("cargo:rustc-env=RUSTC_VERSION={}", version);
}
