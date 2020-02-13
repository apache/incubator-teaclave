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

const FUSION_DATA_PREFIX: &str = "fusion-data-";
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct FusionData {
    pub(crate) url: Url,
    pub(crate) hash: Option<String>,
    pub(crate) crypto_info: TeaclaveFileCryptoInfo,
    pub(crate) data_owner_id_list: Vec<String>,
    pub(crate) data_id: String,
}

fn gen_url_for_fusion_data(data_id: &str) -> Result<Url> {
    let url = format!("fusion://path/{}?token=fusion_token", data_id);
    info!("{}", url);
    Url::parse(&url).map_err(|_| anyhow!("invalid url"))
}

impl FusionData {
    pub fn new(data_owner_id_list: Vec<String>) -> Result<Self> {
        let data_id = format!("{}{}", FUSION_DATA_PREFIX, Uuid::new_v4().to_string());
        let url = gen_url_for_fusion_data(&data_id)?;
        let crypto_info = TeaclaveFileCryptoInfo::default();
        Ok(FusionData {
            url,
            hash: None,
            crypto_info,
            data_owner_id_list,
            data_id,
        })
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

    pub(crate) fn is_fusion_data_id(id: &str) -> bool {
        id.starts_with(FUSION_DATA_PREFIX)
    }
}
