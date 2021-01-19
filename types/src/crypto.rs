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

use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::format;

use teaclave_crypto::*;

pub const FILE_AUTH_TAG_LENGTH: usize = 16;

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileAuthTag {
    tag: [u8; FILE_AUTH_TAG_LENGTH],
}

impl FileAuthTag {
    pub fn from_bytes(input: &[u8]) -> Result<Self> {
        ensure!(input.len() == FILE_AUTH_TAG_LENGTH, "Invalid length");
        let mut file_auth_tag = FileAuthTag::default();
        file_auth_tag.tag.clone_from_slice(&input);
        Ok(file_auth_tag)
    }

    pub fn from_hex(input: impl AsRef<str>) -> Result<Self> {
        let hex = hex::decode(input.as_ref()).context("Illegal AuthTag provided")?;
        let tag = hex
            .as_slice()
            .try_into()
            .context("Illegal AuthTag provided")?;
        Ok(FileAuthTag { tag })
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.tag)
    }

    #[cfg(test_mode)]
    pub fn mock() -> Self {
        Self {
            tag: [0; FILE_AUTH_TAG_LENGTH],
        }
    }
}

impl std::convert::From<[u8; FILE_AUTH_TAG_LENGTH]> for FileAuthTag {
    fn from(tag: [u8; FILE_AUTH_TAG_LENGTH]) -> Self {
        Self { tag }
    }
}

impl std::cmp::PartialEq<[u8]> for FileAuthTag {
    fn eq(&self, other: &[u8]) -> bool {
        self.tag == other
    }
}

impl std::cmp::PartialEq<[u8; FILE_AUTH_TAG_LENGTH]> for FileAuthTag {
    fn eq(&self, other: &[u8; FILE_AUTH_TAG_LENGTH]) -> bool {
        &self.tag[..] == other
    }
}

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
            AesGcm128Key::SCHEMA => {
                let crypto = AesGcm128Key::new(key, iv)?;
                FileCrypto::AesGcm128(crypto)
            }
            AesGcm256Key::SCHEMA => {
                let crypto = AesGcm256Key::new(key, iv)?;
                FileCrypto::AesGcm256(crypto)
            }
            TeaclaveFile128Key::SCHEMA => {
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
            FileCrypto::AesGcm128(_) => AesGcm128Key::SCHEMA,
            FileCrypto::AesGcm256(_) => AesGcm256Key::SCHEMA,
            FileCrypto::TeaclaveFile128(_) => TeaclaveFile128Key::SCHEMA,
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
