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

#[cfg(not(feature = "mesalock_sgx"))]
include!("../../common/prost_build_generator.rs");

#[cfg(not(feature = "mesalock_sgx"))]
fn main() {
    println!("cargo:rerun-if-changed=src/cred.proto");
    let src = PathBuf::from("src");
    let output = src.join("prost_generated");
    if !output.exists() {
        std::fs::create_dir(&output).expect("failed to create prost_generated dir");
    }
    let includes = &[src.clone()];
    let mut config = get_default_config();
    config.out_dir(output);
    config
        .compile_protos(&[src.join("cred.proto")], includes)
        .unwrap();
}

#[cfg(feature = "mesalock_sgx")]
fn main() {}
