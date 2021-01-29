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

use cfg_if::cfg_if;
use std::env;
use std::path::PathBuf;
use std::process::Command;

#[cfg(not(feature = "mesalock_sgx"))]
fn build_non_sgx_protected_fs_c_with_cmake() {
    let build_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("build");
    let target = std::env::var("TARGET").unwrap();
    let profile = std::env::var("PROFILE").unwrap();
    let build_type = match profile.as_str() {
        "debug" => "Debug",
        "release" => "Release",
        _ => panic!("Unsupported profile: {}", profile),
    };

    let script = PathBuf::from("protected_fs_c").join("build.sh");
    let target_dir = if target == "aarch64-apple-ios" {
        build_dir.join("target").join(build_type)
    } else {
        build_dir.join("target")
    };

    let status = Command::new("bash")
        .arg(&script)
        .arg("--build_dir")
        .arg(&build_dir)
        .arg("--mode")
        .arg("non_sgx")
        .arg("--target")
        .arg(&target)
        .arg("--build_type")
        .arg(&build_type)
        .status()
        .expect("bash command failed to start");
    assert!(status.success());

    println!("cargo:rustc-link-search=native={}", target_dir.display());
    println!("cargo:rustc-link-lib=static=tprotected_fs");
    println!("cargo:rustc-link-lib=static=uprotected_fs");
    if target != "aarch64-apple-ios" {
        println!("cargo:rustc-link-lib=crypto");
        println!("cargo:rustc-link-lib=stdc++");
    }
}

#[cfg(feature = "mesalock_sgx")]
fn build_sgx_protected_fs_c_with_cmake() {
    let sdk_dir = env::var("SGX_SDK").unwrap_or("/opt/intel/sgxsdk".into());
    let build_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("build");
    let script = PathBuf::from("protected_fs_c").join("build.sh");
    let target = std::env::var("TARGET").unwrap();
    let profile = std::env::var("PROFILE").unwrap();
    let build_type = match profile.as_str() {
        "debug" => "Debug",
        "release" => "Release",
        _ => panic!("Unsupported profile: {}", profile),
    };
    let target_dir = build_dir.join("target");

    let status = Command::new("bash")
        .env("SGX_SDK", &sdk_dir)
        .arg(&script)
        .arg("--build_dir")
        .arg(&build_dir)
        .arg("--mode")
        .arg("sgx")
        .arg("--target")
        .arg(&target)
        .arg("--build_type")
        .arg(&build_type)
        .status()
        .expect("bash command failed to start");
    assert!(status.success());

    println!("cargo:rustc-link-search=native={}", target_dir.display());
    println!("cargo:rustc-link-lib=static=tprotected_fs");
}

cfg_if! {
    if #[cfg(feature = "mesalock_sgx")] {
        fn build() {
            build_sgx_protected_fs_c_with_cmake();
        }
    } else {
        fn build() {
            build_non_sgx_protected_fs_c_with_cmake();
        }
    }
}

fn main() {
    build();
}
