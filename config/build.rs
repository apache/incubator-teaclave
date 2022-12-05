// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#[cfg(feature = "build_config")]
fn main() {
    use std::env;
    use std::path::Path;
    use std::process::Command;
    use std::str;

    let is_sim = env::var("SGX_MODE").unwrap_or_else(|_| "HW".to_string());
    match is_sim.as_ref() {
        "HW" => {}
        _ => println!("cargo:rustc-cfg=sgx_sim"),
    }

    let out_dir = env::var("OUT_DIR").expect("$OUT_DIR not set. Please build with cargo");
    let dest_file = Path::new(&out_dir).join("build_config.rs");
    println!("cargo:rerun-if-changed=config_gen/main.rs");
    println!("cargo:rerun-if-changed=config_gen/templates/config.j2");
    println!("cargo:rerun-if-changed=build.config.toml");
    println!("cargo:rerun-if-changed=build.rs");
    let target_dir = Path::new(&env::var("TEACLAVE_SYMLINKS").expect("TEACLAVE_SYMLINKS"))
        .join("teaclave_build/target/config_gen");
    let unix_toml_dir = env::var("MT_SGXAPP_TOML_DIR").expect("MT_SGXAPP_TOML_DIR");
    // Use CARGO_ENCODED_RUSTFLAGS to override RUSTFLAGS which makes the run fail.
    let c = Command::new("cargo")
        .env("CARGO_ENCODED_RUSTFLAGS", "")
        .current_dir(&unix_toml_dir)
        .args([
            "run",
            "--target-dir",
            &target_dir.to_string_lossy(),
            "--manifest-path",
            "config/config_gen/Cargo.toml",
            "--",
            "-t",
            "config/build.config.toml",
            "-o",
            &dest_file.to_string_lossy(),
        ])
        .output()
        .expect("Cannot generate build_config.rs");
    if !c.status.success() {
        panic!(
            "stdout: {:?}, stderr: {:?}",
            str::from_utf8(&c.stderr).unwrap(),
            str::from_utf8(&c.stderr).unwrap()
        );
    }
}

#[cfg(not(feature = "build_config"))]
fn main() {}
