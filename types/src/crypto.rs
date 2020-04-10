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

use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};
use std::format;

use teaclave_crypto::*;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FileCrypto {
    AesGcm128(AesGcm128Key),
    AesGcm256(AesGcm256Key),
    TeaclaveFile128(TeaclaveFile128Key),
    Raw,
}

impl FileCrypto {
    pub fn new(schema: &str, key: &[u8], iv: &[u8]) -> Result<Self> {
        let info = match schema {
            "aes_gcm_128" => {
                let crypto = AesGcm128Key::new(key, iv)?;
                FileCrypto::AesGcm128(crypto)
            }
            "aes_gcm_256" => {
                let crypto = AesGcm256Key::new(key, iv)?;
                FileCrypto::AesGcm256(crypto)
            }
            "teaclave_file_128" => {
                ensure!(iv.is_empty(), "IV is not empty for teaclave_file_128");
                let crypto = TeaclaveFile128Key::new(key)?;
                FileCrypto::TeaclaveFile128(crypto)
            }
            "raw" => FileCrypto::Raw,
            _ => bail!("Invalid crypto schema: {}", schema),
        };

        Ok(info)
    }

    pub fn schema(&self) -> &str {
        match self {
            FileCrypto::AesGcm128(_) => "aes_gcm_128",
            FileCrypto::AesGcm256(_) => "aes_gcm_256",
            FileCrypto::TeaclaveFile128(_) => "teaclave_file_128",
            FileCrypto::Raw => "raw",
        }
    }

    pub fn key_iv(&self) -> (Vec<u8>, Vec<u8>) {
        match self {
            FileCrypto::AesGcm128(crypto) => (crypto.key.to_vec(), crypto.iv.to_vec()),
            FileCrypto::AesGcm256(crypto) => (crypto.key.to_vec(), crypto.iv.to_vec()),
            FileCrypto::TeaclaveFile128(crypto) => (crypto.key.to_vec(), Vec::new()),
            FileCrypto::Raw => (vec![], vec![]),
        }
    }
}

impl std::convert::From<AesGcm128Key> for FileCrypto {
    fn from(crypto: AesGcm128Key) -> Self {
        FileCrypto::AesGcm128(crypto)
    }
}

impl std::convert::From<AesGcm256Key> for FileCrypto {
    fn from(crypto: AesGcm256Key) -> Self {
        FileCrypto::AesGcm256(crypto)
    }
}

impl std::convert::From<TeaclaveFile128Key> for FileCrypto {
    fn from(crypto: TeaclaveFile128Key) -> Self {
        FileCrypto::TeaclaveFile128(crypto)
    }
}

impl Default for FileCrypto {
    fn default() -> Self {
        FileCrypto::TeaclaveFile128(TeaclaveFile128Key::random())
    }
}
