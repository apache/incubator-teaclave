use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("$OUT_DIR not set. Please build with cargo");
    let dest_file = Path::new(&out_dir).join("build_config.rs");
    println!("cargo:rerun-if-changed=config_gen/main.rs");
    let c = Command::new("cargo")
        .args(&[
            "run",
            "--manifest-path",
            "config_gen/Cargo.toml",
            "build.config.toml",
            &dest_file.to_string_lossy(),
        ])
        .output()
        .expect("Cannot generate build_config.rs");
    assert!(c.status.success());
}
