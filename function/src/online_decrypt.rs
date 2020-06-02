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
use anyhow::{anyhow, bail, Result};
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
    algorithm: String,
}

impl TryFrom<FunctionArguments> for OnlineDecryptArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

fn decrypt(
    key: &[u8],
    nonce_data: &[u8],
    data: &mut Vec<u8>,
    alg: &'static ring::aead::Algorithm,
) -> anyhow::Result<()> {
    let key =
        LessSafeKey::new(UnboundKey::new(&alg, &key).map_err(|_| anyhow!("decryption error"))?);
    let nonce = Nonce::try_assume_unique_for_key(&nonce_data[0..12])
        .map_err(|_| anyhow!("decryption error"))?;
    key.open_in_place(nonce, Aad::empty(), data)
        .map_err(|_| anyhow!("decryption error"))?;
    data.truncate(data.len() - alg.tag_len());
    Ok(())
}

fn decrypt_string_base64(
    key: &str,
    nonce_str: &str,
    encrypted: &str,
    alg: &'static ring::aead::Algorithm,
) -> anyhow::Result<String> {
    let decoded_key = base64::decode(&key)?;
    let nonce = base64::decode(&nonce_str)?;
    let mut data_vec = base64::decode(&encrypted)?;

    decrypt(&decoded_key, &nonce, &mut data_vec, &alg).map_err(|_| anyhow!("decryption error"))?;
    let string = str::from_utf8(&data_vec).map_err(|_| anyhow!("base64 decoded error"))?;

    Ok(string.to_string())
}

impl OnlineDecrypt {
    pub const NAME: &'static str = "builtin_online_decrypt";

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
        let algorithm = &args.algorithm[..];

        let alg = match algorithm {
            "aes128gcm" => &AES_128_GCM,
            "aes256gcm" => &AES_256_GCM,
            _ => bail!("Invalid algorithm"),
        };

        let result = decrypt_string_base64(&key, &nonce, &encrypted_data, alg)?;
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

    fn test_subroutine(args: FunctionArguments, result: &str) {
        let input_files = StagedFiles::default();
        let output_files = StagedFiles::default();
        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));
        let function = OnlineDecrypt;
        let summary = function.run(args, runtime).unwrap();
        assert_eq!(summary, result);
    }

    fn test_online_decrypt() {
        let args1 = FunctionArguments::from_json(json!({
            "key": "aqUdgZ0lJnuz9yiPkoDxM6ZcTcVVpd4KKLqzbHD88Lg=",
            "nonce": "AAECAwQFBgcICQoL",
            "encrypted_data": "CaZd8qSMMlBp8SjSXj2I4dQIuC9KkZ5DI/ATo1sWJw==",
            "algorithm": "aes256gcm"
        }))
        .unwrap();
        test_subroutine(args1, "Hello Teaclave!");

        let args2 = FunctionArguments::from_json(json!({
            "key": "aqUdgZ0lJnuz9yiPkoDxMw==",
            "nonce": "AAECAwQFBgcICQoL",
            "encrypted_data": "OqMscYqxk1CshHQZulTrrDlJjS/v6BE/clWJyTerUw==",
            "algorithm": "aes128gcm"
        }))
        .unwrap();
        test_subroutine(args2, "Hello Teaclave!");
    }
}
