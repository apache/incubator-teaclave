use std::env;

fn main() {
    // Link to static library if `modular` is not enabled.
    println!("cargo:rerun-if-changed=build.rs");

    if env::var("USE_STATIC").is_ok_and(|var| var.to_lowercase() == "true" || var == "1") {
        let lib_path = env::var("STATIC_LIB_PATH").expect("must set `STATIC_LIB_PATH`");
        let lib_name = env::var("STATIC_LIB_NAME").expect("must set `STATIC_LIB_NAME`");

        println!("cargo:rustc-link-search={}", lib_path);
        println!("cargo:rustc-link-lib=static={}", lib_name);
    }
}
