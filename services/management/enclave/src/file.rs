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
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::prelude::v1::*;
use teaclave_types::TeaclaveFileCryptoInfo;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InputFile {
    pub(crate) url: Url,
    pub(crate) hash: String,
    pub(crate) crypto_info: TeaclaveFileCryptoInfo,
    pub(crate) owner: String,
    pub(crate) data_id: String,
}

impl InputFile {
    pub(crate) fn new(
        url: Url,
        hash: String,
        crypto_info: TeaclaveFileCryptoInfo,
        owner: String,
    ) -> InputFile {
        InputFile {
            url,
            hash,
            crypto_info,
            owner,
            data_id: Uuid::new_v4().to_string(),
        }
    }

    #[cfg(feature = "enclave_unit_test")]
    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self> {
        let ret: InputFile =
            serde_json::from_slice(&bytes).map_err(|_| anyhow!("failed to Deserialize"))?;
        Ok(ret)
    }

    pub(crate) fn to_vec(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self).map_err(|_| anyhow!("failed to Serialize"))
    }

    pub(crate) fn get_key_vec_from_id(id: &str) -> Vec<u8> {
        let mut key = b"input-file-".to_vec();
        key.extend_from_slice(id.as_bytes());
        key
    }
    pub(crate) fn get_key_vec(&self) -> Vec<u8> {
        InputFile::get_key_vec_from_id(&self.data_id)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct OutputFile {
    pub(crate) url: Url,
    pub(crate) hash: Option<String>,
    pub(crate) crypto_info: TeaclaveFileCryptoInfo,
    pub(crate) owner: String,
    pub(crate) data_id: String,
}

impl OutputFile {
    pub(crate) fn new(url: Url, crypto_info: TeaclaveFileCryptoInfo, owner: String) -> OutputFile {
        OutputFile {
            url,
            hash: None,
            crypto_info,
            owner,
            data_id: Uuid::new_v4().to_string(),
        }
    }

    #[cfg(feature = "enclave_unit_test")]
    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(&bytes).map_err(|_| anyhow!("failed to Deserialize"))
    }

    pub(crate) fn to_vec(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self).map_err(|_| anyhow!("failed to Serialize"))
    }

    pub(crate) fn get_key_vec_from_id(id: &str) -> Vec<u8> {
        let mut key = b"output-file-".to_vec();
        key.extend_from_slice(id.as_bytes());
        key
    }
    pub(crate) fn get_key_vec(&self) -> Vec<u8> {
        OutputFile::get_key_vec_from_id(&self.data_id)
    }
}
