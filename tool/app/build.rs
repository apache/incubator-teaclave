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

use std::env;
use std::path::PathBuf;

fn choose_sgx_dylib(is_sim: bool) {
    if is_sim {
        println!("cargo:rustc-link-lib=dylib=sgx_urts_sim");
        println!("cargo:rustc-link-lib=dylib=sgx_uae_service_sim");
    } else {
        println!("cargo:rustc-link-lib=dylib=sgx_urts");
        println!("cargo:rustc-link-lib=dylib=sgx_uae_service");
    }
}

fn main() {
    let sdk_dir = env::var("SGX_SDK").unwrap_or("/opt/intel/sgxsdk".into());
    println!("cargo:rustc-link-search=native={}/lib64", sdk_dir);

    let out_path = env::var_os("ENCLAVE_OUT_DIR").unwrap_or("out".into());
    let out_dir = &PathBuf::from(out_path);

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    if let Ok(edl_dir) = env::var("TEACLAVE_EDL_DIR") {
        println!("cargo:rerun-if-changed={}/Enclave_common.edl", edl_dir);
    }
    println!("cargo:rustc-link-lib=static=Enclave_common_u");

    let is_sim = match env::var("SGX_MODE") {
        Ok(ref v) if v == "SW" => true,
        Ok(ref v) if v == "HW" => false,
        Err(env::VarError::NotPresent) => false,
        _ => {
            panic!("Stop build process, wrong SGX_MODE env provided.");
        }
    };

    choose_sgx_dylib(is_sim);
}
