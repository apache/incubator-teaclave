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

use std::collections::HashMap;

use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Deserializer};

pub type SgxMeasurement = [u8; sgx_types::SGX_HASH_SIZE];

#[derive(Debug, Deserialize, Copy, Clone, Eq, PartialEq)]
pub struct EnclaveMeasurement {
    #[serde(deserialize_with = "from_hex")]
    pub mr_signer: SgxMeasurement,
    #[serde(deserialize_with = "from_hex")]
    pub mr_enclave: SgxMeasurement,
}

impl EnclaveMeasurement {
    pub fn new(mr_enclave: SgxMeasurement, mr_signer: SgxMeasurement) -> Self {
        Self {
            mr_enclave,
            mr_signer,
        }
    }
}

/// Deserializes a hex string to a `SgxMeasurement` (i.e., [0; 32]).
pub fn from_hex<'de, D>(deserializer: D) -> Result<SgxMeasurement, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer).and_then(|string| {
        let v = hex::decode(&string).map_err(|_| Error::custom("ParseError"))?;
        let mut array = [0; sgx_types::SGX_HASH_SIZE];
        let bytes = &v[..array.len()]; // panics if not enough data
        array.copy_from_slice(bytes);
        Ok(array)
    })
}

#[derive(Clone)]
pub struct EnclaveAttr {
    pub measurement: EnclaveMeasurement,
}

pub struct EnclaveInfo {
    pub measurements: HashMap<String, EnclaveMeasurement>,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct EnclaveInfoToml(HashMap<String, EnclaveMeasurement>);

impl EnclaveInfo {
    pub fn verify_and_new<T, U>(
        enclave_info: &[u8],
        public_keys: &[T],
        signatures: &[U],
    ) -> Result<Self>
    where
        T: AsRef<[u8]>,
        U: AsRef<[u8]>,
    {
        ensure!(
            signatures.len() <= public_keys.len(),
            "Invalid number of public keys"
        );
        if !Self::verify(enclave_info, public_keys, signatures) {
            bail!("Invalid enclave_info");
        }

        Ok(Self::from_bytes(enclave_info))
    }

    #[cfg(feature = "app")]
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        Ok(Self::from_bytes(&bytes))
    }

    pub fn from_bytes(enclave_info: &[u8]) -> Self {
        let config: EnclaveInfoToml = toml::from_slice(enclave_info)
            .expect("Content not correct, unable to load enclave info.");
        let mut info_map = std::collections::HashMap::new();
        for (k, v) in config.0 {
            info_map.insert(k, EnclaveMeasurement::new(v.mr_enclave, v.mr_signer));
        }

        Self {
            measurements: info_map,
        }
    }

    pub fn verify<T, U>(enclave_info: &[u8], public_keys: &[T], signatures: &[U]) -> bool
    where
        T: AsRef<[u8]>,
        U: AsRef<[u8]>,
    {
        use ring::signature;

        for s in signatures {
            let mut verified = false;
            for k in public_keys {
                if signature::UnparsedPublicKey::new(&signature::RSA_PKCS1_2048_8192_SHA256, k)
                    .verify(enclave_info, s.as_ref())
                    .is_ok()
                {
                    verified = true;
                }
            }
            if !verified {
                return false;
            }
        }

        true
    }

    pub fn get_enclave_attr(&self, service_name: &str) -> Option<EnclaveAttr> {
        if let Some(measurement) = self.measurements.get(service_name) {
            Some(EnclaveAttr {
                measurement: *measurement,
            })
        } else {
            None
        }
    }
}
