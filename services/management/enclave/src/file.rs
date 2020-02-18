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

const INPUT_FILE_PREFIX: &str = "input-file-";
const OUTPUT_FILE_PREFIX: &str = "output-file-";
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
        let data_id = format!("{}{}", INPUT_FILE_PREFIX, Uuid::new_v4().to_string());
        InputFile {
            url,
            hash,
            crypto_info,
            owner,
            data_id,
        }
    }

    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self> {
        let ret: InputFile =
            serde_json::from_slice(&bytes).map_err(|_| anyhow!("failed to Deserialize"))?;
        Ok(ret)
    }

    pub(crate) fn to_vec(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self).map_err(|_| anyhow!("failed to Serialize"))
    }

    pub(crate) fn get_key_vec(&self) -> Vec<u8> {
        self.data_id.as_bytes().to_vec()
    }

    pub(crate) fn is_input_file_id(id: &str) -> bool {
        id.starts_with(INPUT_FILE_PREFIX)
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
        let data_id = format!("{}{}", OUTPUT_FILE_PREFIX, Uuid::new_v4().to_string());
        OutputFile {
            url,
            hash: None,
            crypto_info,
            owner,
            data_id,
        }
    }

    pub(crate) fn from_slice(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(&bytes).map_err(|_| anyhow!("failed to Deserialize"))
    }

    pub(crate) fn to_vec(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(&self).map_err(|_| anyhow!("failed to Serialize"))
    }

    pub(crate) fn get_key_vec(&self) -> Vec<u8> {
        self.data_id.as_bytes().to_vec()
    }

    pub(crate) fn is_output_file_id(id: &str) -> bool {
        id.starts_with(OUTPUT_FILE_PREFIX)
    }
}
