use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let is_sim = env::var("SGX_MODE").unwrap_or_else(|_| "HW".to_string());
    match is_sim.as_ref() {
        "HW" => {}
        _ => println!("cargo:rustc-cfg=sgx_sim"),
    }

    let out_dir = env::var("OUT_DIR").expect("$OUT_DIR not set. Please build with cargo");
    let dest_file = Path::new(&out_dir).join("build_config.rs");
    println!("cargo:rerun-if-changed=build.config.toml");
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
    if !c.status.success() {
        panic!("stdout: {:?}, stderr: {:?}", c.stdout, c.stderr);
    }
}
