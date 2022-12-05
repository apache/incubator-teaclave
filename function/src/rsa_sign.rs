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

use std::vec;

use ring::{rand, signature};

use std::convert::TryFrom;
use teaclave_types::{FunctionArguments, FunctionRuntime};

const IN_DATA: &str = "rsa_key";

#[derive(serde::Deserialize)]
struct RsaSignArguments {
    data: String,
}

impl TryFrom<FunctionArguments> for RsaSignArguments {
    type Error = anyhow::Error;

    fn try_from(arguments: FunctionArguments) -> Result<Self, Self::Error> {
        use anyhow::Context;
        serde_json::from_str(&arguments.into_string()).context("Cannot deserialize arguments")
    }
}

#[derive(Default)]
pub struct RsaSign;

impl RsaSign {
    pub const NAME: &'static str = "builtin-rsa-sign";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(
        &self,
        arguments: FunctionArguments,
        runtime: FunctionRuntime,
    ) -> anyhow::Result<String> {
        let args = RsaSignArguments::try_from(arguments)?;

        let mut key = Vec::new();
        let mut f = runtime.open_input(IN_DATA)?;
        f.read_to_end(&mut key)?;
        let key_pair =
            signature::RsaKeyPair::from_der(&key).map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let mut sig = vec![0; key_pair.public_modulus_len()];
        let rng = rand::SystemRandom::new();
        key_pair
            .sign(
                &signature::RSA_PKCS1_SHA256,
                &rng,
                args.data.as_bytes(),
                &mut sig,
            )
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let output_base64 = base64::encode(&sig);
        Ok(output_base64)
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use serde_json::json;
    use std::untrusted::fs;
    use teaclave_crypto::*;
    use teaclave_runtime::*;
    use teaclave_test_utils::*;
    use teaclave_types::*;

    pub fn run_tests() -> bool {
        run_tests!(test_rsa_sign)
    }

    fn test_rsa_sign() {
        let arguments = FunctionArguments::from_json(json!({
            "data": "test data",
        }))
        .unwrap();

        let plain_input = "fixtures/functions/rsa_sign/key.der";
        let expected_output = "fixtures/functions/rsa_sign/expected_rsasign.txt";

        let input_files = StagedFiles::new(hashmap!(
            IN_DATA =>
            StagedFileInfo::new(plain_input, TeaclaveFile128Key::random(), FileAuthTag::mock())
        ));

        let output_files = StagedFiles::default();
        let runtime = Box::new(RawIoRuntime::new(input_files, output_files));

        let summary = RsaSign::new().run(arguments, runtime).unwrap();
        let mut expected_string = fs::read_to_string(expected_output).unwrap();
        expected_string = expected_string.replace('\n', "");
        assert_eq!(summary, expected_string);
    }
}
