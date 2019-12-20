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
    let proto_files = ["task_external.proto", "task_internal.proto"];;
    let src = PathBuf::from("src").join("proto");
    let mut to_compiled_protos = Vec::new();
    for proto in proto_files.iter() {
        let path = src.join(proto);
        println!("cargo:rerun-if-changed={}", path.to_string_lossy());
        to_compiled_protos.push(path);
    }
    let output = PathBuf::from("src").join("prost_generated");
    if !output.exists() {
        std::fs::create_dir(&output).expect("failed to create prost_generated dir");
    }
    let includes = &[src];
    let mut config = get_default_config();
    config.extern_path(".kms_proto", "kms_proto::proto");
    config.extern_path(".cred_proto", "authentication_proto::proto");
    config.extern_path(".data_common", "tdfs_common_proto::data_common_proto");
    config.out_dir(output);
    config
        .compile_protos(&to_compiled_protos, includes)
        .unwrap();
}

#[cfg(feature = "mesalock_sgx")]
fn main() {}
