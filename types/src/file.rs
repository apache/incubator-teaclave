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

use crate::storage::Storable;
use crate::FileCrypto;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::prelude::v1::*;
use url::Url;
use uuid::Uuid;

const INPUT_FILE_PREFIX: &str = "input-file";
const OUTPUT_FILE_PREFIX: &str = "output-file";

fn create_uuid() -> Uuid {
    Uuid::new_v4()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeaclaveInputFile {
    pub url: Url,
    pub hash: String,
    pub crypto_info: FileCrypto,
    pub owner: HashSet<String>,
    pub uuid: Uuid,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TeaclaveOutputFile {
    pub url: Url,
    pub hash: Option<String>,
    pub crypto_info: FileCrypto,
    pub owner: HashSet<String>,
    pub uuid: Uuid,
}

impl TeaclaveInputFile {
    pub fn new(
        url: Url,
        hash: String,
        crypto_info: FileCrypto,
        owner: HashSet<String>,
    ) -> TeaclaveInputFile {
        TeaclaveInputFile {
            url,
            hash,
            crypto_info,
            owner,
            uuid: create_uuid(),
        }
    }

    pub fn from_output(output: TeaclaveOutputFile) -> Result<TeaclaveInputFile> {
        let input = TeaclaveInputFile {
            url: output.url,
            hash: output
                .hash
                .ok_or_else(|| anyhow!("output is not finished"))?,
            crypto_info: output.crypto_info,
            owner: output.owner,
            uuid: output.uuid,
        };
        Ok(input)
    }
}

impl Storable for TeaclaveInputFile {
    fn key_prefix() -> &'static str {
        INPUT_FILE_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.uuid
    }
}

impl TeaclaveOutputFile {
    pub fn new(url: Url, crypto_info: FileCrypto, owner: HashSet<String>) -> TeaclaveOutputFile {
        TeaclaveOutputFile {
            url,
            hash: None,
            crypto_info,
            owner,
            uuid: create_uuid(),
        }
    }

    pub fn new_fusion_data(owner: HashSet<String>) -> Result<TeaclaveOutputFile> {
        let uuid = create_uuid();
        let url = format!("fusion://path/{}?token=fusion_token", uuid.to_string());
        let url = Url::parse(&url).map_err(|_| anyhow!("invalid url"))?;
        let crypto_info = FileCrypto::default();

        Ok(TeaclaveOutputFile {
            url,
            hash: None,
            crypto_info,
            owner,
            uuid,
        })
    }
}

impl Storable for TeaclaveOutputFile {
    fn key_prefix() -> &'static str {
        OUTPUT_FILE_PREFIX
    }

    fn uuid(&self) -> Uuid {
        self.uuid
    }
}
