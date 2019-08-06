// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use cfg_if::cfg_if;
use std::env;

cfg_if! {
    if #[cfg(feature = "mesalock_sgx")]  {
        const LIB_T_PROTECTED_FS_NAME: &'static str = "sgx_tprotected_fs";
    } else {
        use std::path::PathBuf;
        use std::process::Command;
        const LIBPROTECTED_FS_NAME: &'static str = "protected_fs";
        const PROTECTED_FS_C_NAME: &'static str = "protected_fs_c";
    }
}

#[cfg(not(feature = "mesalock_sgx"))]
fn build_protected_fs_c() {
    Command::new("make")
        .arg("--version")
        .output()
        .expect("make not found");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_dir = manifest_dir.join(PROTECTED_FS_C_NAME);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let build_dir = out_dir.join(PROTECTED_FS_C_NAME);

    let output = Command::new("make")
        .current_dir(&source_dir)
        .env("CXXFLAGS", "")
        .env("PROTECTED_FS_OUT_DIR", &build_dir)
        .output()
        .expect("failed to compile protected_fs_c");
    if !output.status.success() {
        panic!("failed to compile protected_fs_c");
    }

    env::set_var("PROTECTED_FS_LIB_DIR", &build_dir);
    env::set_var("PROTECTED_FS_STATIC", "true");
}

cfg_if! {
    if #[cfg(feature = "mesalock_sgx")] {
        fn build() {
            let sdk_dir = env::var("SGX_SDK").unwrap_or("/opt/intel/sgxsdk".into());
            println!("cargo:rustc-link-search=native={}/lib64", sdk_dir);
            println!("cargo:rustc-link-lib=static={}", LIB_T_PROTECTED_FS_NAME);
        }
    } else {
        fn build() {
            build_protected_fs_c();

            if let Ok(lib_dir) = env::var("PROTECTED_FS_LIB_DIR") {
                println!("cargo:rustc-link-search=native={}", lib_dir);
                let mode = match env::var_os("PROTECTED_FS_STATIC") {
                    Some(_) => "static",
                    None => panic!("Not supported dylib."),
                };
                println!("cargo:rustc-link-lib={}={}", mode, LIBPROTECTED_FS_NAME);
            } else {
                panic!("Env var {} not set", "PROTECTED_FS_LIB_DIR");
            }
            println!("cargo:rustc-link-lib=crypto");
            println!("cargo:rustc-link-lib=stdc++");
        }
    }
}

fn main() {
    build();
}
