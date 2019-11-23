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

#[cfg(not(feature = "mesalock_sgx"))]
include!("../../common/prost_build_generator.rs");

#[cfg(not(feature = "mesalock_sgx"))]
fn main() {
    let _ = env_logger::init();

    let src = PathBuf::from("src");
    let _ = std::fs::create_dir("src/prost_generated");
    let output = PathBuf::from("src/prost_generated");

    let includes = &[src.clone()];
    let mut config = get_default_config();
    config.out_dir(output.clone());
    let base64_field = [
        "AeadConfig.key",
        "AeadConfig.nonce",
        "AeadConfig.ad",
        "ProtectedFsConfig.key",
    ];
    for field_name in base64_field.iter() {
        config.field_attribute(field_name, "#[serde(with = \"crate::base64_coder\")]");
    }
    config
        .compile_protos(&[src.join("kms.proto")], includes)
        .unwrap();
}

#[cfg(feature = "mesalock_sgx")]
fn main() {}
