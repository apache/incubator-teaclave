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
use crate::proto;
use mesatee_core::{Error, ErrorKind, Result};
use rand::prelude::RngCore;
use std::convert::TryFrom;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

#[derive(Clone, Debug)]
pub enum EncType {
    Aead,
    ProtectedFs,
}

#[derive(Clone, PartialEq, Debug, serde_derive::Serialize, serde_derive::Deserialize)]
pub enum KeyConfig {
    Aead(AEADKeyConfig),
    ProtectedFs([u8; 16]),
}

#[derive(Clone, PartialEq, Debug, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct AEADKeyConfig {
    pub key: [u8; 32],
    pub nonce: [u8; 12],
    pub ad: [u8; 5], // Todo: removed ad;
}

impl KeyConfig {
    pub fn new_aead_config() -> KeyConfig {
        let mut key_config = AEADKeyConfig {
            key: [0; 32],
            nonce: [0; 12],
            ad: [0; 5],
        };
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut key_config.key);
        rng.fill_bytes(&mut key_config.nonce);
        rng.fill_bytes(&mut key_config.ad);
        KeyConfig::Aead(key_config)
    }
    pub fn new_protected_fs_config() -> KeyConfig {
        let mut key = [0; 16];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut key);
        KeyConfig::ProtectedFs(key)
    }
}

impl From<AEADKeyConfig> for proto::AeadConfig {
    fn from(config: AEADKeyConfig) -> Self {
        proto::AeadConfig {
            key: config.key.to_vec(),
            nonce: config.nonce.to_vec(),
            ad: config.ad.to_vec(),
        }
    }
}
impl TryFrom<proto::AeadConfig> for AEADKeyConfig {
    type Error = Error;
    fn try_from(config: proto::AeadConfig) -> Result<Self> {
        if config.key.len() != 32 || config.nonce.len() != 12 || config.ad.len() != 5 {
            return Err(Error::from(ErrorKind::InvalidOutputError));
        }
        let mut result = AEADKeyConfig {
            key: [0; 32],
            nonce: [0; 12],
            ad: [0; 5],
        };
        result.key.copy_from_slice(&config.key[0..32]);
        result.nonce.copy_from_slice(&config.nonce[0..12]);
        result.ad.copy_from_slice(&config.ad[0..5]);
        Ok(result)
    }
}
impl From<KeyConfig> for proto::KeyConfig {
    fn from(config: KeyConfig) -> Self {
        let key_config = match config {
            KeyConfig::Aead(config) => crate::proto::key_config::Config::Aead(config.into()),
            KeyConfig::ProtectedFs(key) => {
                proto::key_config::Config::ProtectedFs(proto::ProtectedFsConfig {
                    key: key.to_vec(),
                })
            }
        };
        proto::KeyConfig {
            config: Some(key_config),
        }
    }
}

impl TryFrom<proto::KeyConfig> for KeyConfig {
    type Error = Error;
    fn try_from(config: proto::KeyConfig) -> Result<Self> {
        let config = match config.config {
            Some(v) => v,
            None => return Err(Error::from(ErrorKind::InvalidOutputError)),
        };
        match config {
            proto::key_config::Config::Aead(config) => {
                AEADKeyConfig::try_from(config).map(KeyConfig::Aead)
            }
            proto::key_config::Config::ProtectedFs(config) => {
                if config.key.len() != 16 {
                    Err(Error::from(ErrorKind::InvalidOutputError))
                } else {
                    let mut key = [0; 16];
                    key.copy_from_slice(&config.key[0..16]);
                    Ok(KeyConfig::ProtectedFs(key))
                }
            }
        }
    }
}

impl proto::CreateKeyRequest {
    pub fn new(enc_type: EncType) -> Self {
        let proto_enc_type = match enc_type {
            EncType::Aead => proto::EncType::Aead,
            EncType::ProtectedFs => proto::EncType::ProtectedFs,
        };
        proto::CreateKeyRequest {
            enc_type: proto_enc_type as i32,
        }
    }
    pub fn get_enc_type(&self) -> Result<EncType> {
        match proto::EncType::from_i32(self.enc_type) {
            Some(proto::EncType::Aead) => Ok(EncType::Aead),
            Some(proto::EncType::ProtectedFs) => Ok(EncType::ProtectedFs),
            None => Err(Error::from(ErrorKind::InvalidInputError)),
        }
    }
}

impl proto::CreateKeyResponse {
    pub fn new(key_id: &str, config: &KeyConfig) -> Self {
        proto::CreateKeyResponse {
            key_id: key_id.to_owned(),
            config: proto::KeyConfig::from(config.clone()),
        }
    }
    pub fn get_key_id(&self) -> String {
        self.key_id.clone()
    }
    pub fn get_key_config(&self) -> Result<KeyConfig> {
        KeyConfig::try_from(self.config.clone())
    }
}

impl proto::DeleteKeyRequest {
    pub fn new(key_id: &str) -> Self {
        proto::DeleteKeyRequest {
            key_id: key_id.to_owned(),
        }
    }
    pub fn get_key_id(&self) -> String {
        self.key_id.clone()
    }
}

impl proto::DeleteKeyResponse {
    pub fn new(config: &KeyConfig) -> Self {
        proto::DeleteKeyResponse {
            config: proto::KeyConfig::from(config.clone()),
        }
    }
    pub fn get_key_config(&self) -> Result<KeyConfig> {
        KeyConfig::try_from(self.config.clone())
    }
}

impl proto::GetKeyRequest {
    pub fn new(key_id: &str) -> Self {
        proto::GetKeyRequest {
            key_id: key_id.to_owned(),
        }
    }
    pub fn get_key_id(&self) -> String {
        self.key_id.clone()
    }
}

impl proto::GetKeyResponse {
    pub fn new(config: &KeyConfig) -> Self {
        proto::GetKeyResponse {
            config: proto::KeyConfig::from(config.clone()),
        }
    }
    pub fn get_key_config(&self) -> Result<KeyConfig> {
        KeyConfig::try_from(self.config.clone())
    }
}
