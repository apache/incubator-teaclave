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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
extern crate base64;
use ring::aead::*;
use std::convert::TryFrom;
use std::str;
use teaclave_types::{FunctionArguments, FunctionRuntime};

#[derive(Default)]
pub struct OnlineDecrypt;

#[derive(serde::Deserialize)]
struct OnlineDecryptArguments {
    key: String,
    nonce: String,
    encrypted_data: String,
}

impl TryFrom<FunctionArguments> for OnlineDecryptArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

fn decrypt(key: &[u8], nonce_data: &[u8], data: &mut Vec<u8>) {
    let key = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, &key).unwrap());
    let nonce = Nonce::try_assume_unique_for_key(&nonce_data[0..12]).unwrap();
    key.open_in_place(nonce, Aad::empty(), data).unwrap();
    data.truncate(data.len() - AES_256_GCM.tag_len());
}

fn decrypt_string_base64(key: &str, nonce_str: &str, encrypted: &str) -> String {
    let decoded_key = base64::decode(&key).unwrap();
    let nonce = base64::decode(&nonce_str).unwrap();
    let mut data_vec = base64::decode(&encrypted).unwrap();

    decrypt(&decoded_key, &nonce, &mut data_vec);

    let string = match str::from_utf8(&data_vec) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    string.to_string()
}

impl OnlineDecrypt {
    pub const NAME: &'static str = "builtin-online-decrypt";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        arguments: FunctionArguments,
        _runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let args = OnlineDecryptArguments::try_from(arguments)?;
        let key = args.key;
        let nonce = args.nonce;
        let encrypted_data = args.encrypted_data;
        let result = decrypt_string_base64(&key, &nonce, &encrypted_data);
        Ok(result)
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use serde_json::json;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_online_decrypt)
    }

    fn test_online_decrypt() {
        let args = FunctionArguments::from_json(json!({
            "key": "aqUdgZ0lJnuz9yiPkoDxM6ZcTcVVpd4KKLqzbHD88Lg=",
            "nonce": "AAECAwQFBgcICQoL",
            "encrypted_data": "CaZd8qSMMlBp8SjSXj2I4dQIuC9KkZ5DI/ATo1sWJw=="
        }))
        .unwrap();

        let input_files = StagedFiles::default();
        let output_files = StagedFiles::default();

        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));
        let function = OnlineDecrypt;

        let summary = function.run(args, runtime).unwrap();
        assert_eq!(summary, "Hello Teaclave!");
    }
}
