// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use mesatee_core;
use mesatee_core::Result;
use serde_derive::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum KMSRequest {
    Create(CreateKeyRequest),
    Get(GetKeyRequest),
    Delete(DeleteKeyRequest),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum KMSResponse {
    Create(CreateKeyResponse),
    Get(GetKeyResponse),
    Delete(DeleteKeyResponse),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct AEADKeyConfig {
    #[serde(with = "base64_coder")]
    pub key: Vec<u8>,
    #[serde(with = "base64_coder")]
    pub nonce: Vec<u8>,
    #[serde(with = "base64_coder")]
    pub ad: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CreateKeyRequest {}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CreateKeyResponse {
    pub key_id: String,
    pub config: AEADKeyConfig,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetKeyRequest {
    pub key_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetKeyResponse {
    pub config: AEADKeyConfig,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeleteKeyRequest {
    pub key_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeleteKeyResponse {
    pub config: AEADKeyConfig,
}

impl KMSRequest {
    pub fn new_create_key() -> KMSRequest {
        KMSRequest::Create(CreateKeyRequest {})
    }

    pub fn new_get_key(key_id: &str) -> KMSRequest {
        let req = GetKeyRequest {
            key_id: key_id.to_owned(),
        };
        KMSRequest::Get(req)
    }

    pub fn new_del_key(key_id: &str) -> KMSRequest {
        let req = DeleteKeyRequest {
            key_id: key_id.to_owned(),
        };
        KMSRequest::Delete(req)
    }
}

impl KMSResponse {
    pub fn new_create_key(key_id: &str, key: &AEADKeyConfig) -> KMSResponse {
        let resp = CreateKeyResponse {
            key_id: key_id.to_owned(),
            config: key.clone(),
        };
        KMSResponse::Create(resp)
    }

    pub fn new_get_key(config: &AEADKeyConfig) -> KMSResponse {
        let resp = GetKeyResponse {
            config: config.clone(),
        };
        KMSResponse::Get(resp)
    }

    pub fn new_del_key(config: &AEADKeyConfig) -> KMSResponse {
        let resp = DeleteKeyResponse {
            config: config.clone(),
        };
        KMSResponse::Delete(resp)
    }
}

impl AEADKeyConfig {
    pub fn new() -> Result<Self> {
        use rand::prelude::RngCore;

        let mut key_config = AEADKeyConfig {
            key: vec![0; 32],
            nonce: vec![0; 12],
            ad: vec![0; 5],
        };

        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut key_config.key);
        rng.fill_bytes(&mut key_config.nonce);
        rng.fill_bytes(&mut key_config.ad);

        Ok(key_config)
    }
}

mod base64_coder {
    // Insert std prelude in the top for the sgx feature
    #[cfg(feature = "mesalock_sgx")]
    use std::prelude::v1::*;

    extern crate base64;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&base64::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        base64::decode(s).map_err(de::Error::custom)
    }
}
