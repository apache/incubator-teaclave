#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use anyhow::{anyhow, Error, Result};
use std::format;

use crate::teaclave_common_proto as proto;

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct UserCredential {
    pub id: std::string::String,
    pub token: std::string::String,
}

impl std::convert::TryFrom<proto::UserCredential> for UserCredential {
    type Error = Error;

    fn try_from(proto: proto::UserCredential) -> Result<Self> {
        let ret = Self {
            id: proto.id,
            token: proto.token,
        };

        Ok(ret)
    }
}

impl From<UserCredential> for proto::UserCredential {
    fn from(request: UserCredential) -> Self {
        Self {
            id: request.id,
            token: request.token,
        }
    }
}

const AES_GCM_256_KEY_LENGTH: usize = 32;
const AES_GCM_256_IV_LENGTH: usize = 12;

#[derive(Default, Debug, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct AesGcm256CryptoInfo {
    pub key: [u8; 32],
    pub iv: [u8; 12],
}

impl AesGcm256CryptoInfo {
    fn try_new(key: &[u8], iv: &[u8]) -> Result<Self> {
        if key.len() != AES_GCM_256_KEY_LENGTH {
            return Err(anyhow!(format!(
                "Invalid key length for AesGcm256: {}",
                key.len()
            )));
        }
        if iv.len() != AES_GCM_256_IV_LENGTH {
            return Err(anyhow!(format!(
                "Invalid iv length for AesGcm256: {}",
                iv.len()
            )));
        }

        let mut info = AesGcm256CryptoInfo::default();
        info.key.copy_from_slice(key);
        info.iv.copy_from_slice(iv);
        Ok(info)
    }
}

const AES_GCM_128_KEY_LENGTH: usize = 16;
const AES_GCM_128_IV_LENGTH: usize = 12;

#[derive(Default, Debug, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct AesGcm128CryptoInfo {
    pub key: [u8; AES_GCM_128_KEY_LENGTH],
    pub iv: [u8; AES_GCM_128_IV_LENGTH],
}

impl AesGcm128CryptoInfo {
    fn try_new(key: &[u8], iv: &[u8]) -> Result<Self> {
        if key.len() != AES_GCM_128_KEY_LENGTH {
            return Err(anyhow!(format!(
                "Invalid key length for AesGcm128: {}",
                key.len()
            )));
        }
        if iv.len() != AES_GCM_128_IV_LENGTH {
            return Err(anyhow!(format!(
                "Invalid iv length for AesGcm128: {}",
                iv.len()
            )));
        }

        let mut info = AesGcm128CryptoInfo::default();
        info.key.copy_from_slice(key);
        info.iv.copy_from_slice(iv);
        Ok(info)
    }
}

const TEACLAVE_FILE_ROOT_KEY_128_LENGTH: usize = 16;

#[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
pub enum TeaclaveFileCryptoInfo {
    AesGcm128(AesGcm128CryptoInfo),
    AesGcm256(AesGcm256CryptoInfo),
    TeaclaveFileRootKey128([u8; 16]),
}

impl std::convert::TryFrom<proto::FileCryptoInfo> for TeaclaveFileCryptoInfo {
    type Error = Error;
    fn try_from(proto: proto::FileCryptoInfo) -> Result<Self> {
        let info = match proto.schema.as_str() {
            "aes_gcm_128" => {
                let info = AesGcm128CryptoInfo::try_new(&proto.key, &proto.iv)?;
                TeaclaveFileCryptoInfo::AesGcm128(info)
            }
            "aes_gcm_256" => {
                let info = AesGcm256CryptoInfo::try_new(&proto.key, &proto.iv)?;
                TeaclaveFileCryptoInfo::AesGcm256(info)
            }
            "teaclave_file_root_key_128" => {
                let rkey = &proto.key;
                if rkey.len() != TEACLAVE_FILE_ROOT_KEY_128_LENGTH {
                    return Err(anyhow!(format!(
                        "Invalid key length for teaclave_file_root_key_128: {}",
                        rkey.len()
                    )));
                }
                let mut key = [0; TEACLAVE_FILE_ROOT_KEY_128_LENGTH];
                key.copy_from_slice(rkey);
                TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(key)
            }
            _ => {
                return Err(anyhow!(format!(
                    "Invalid crypto schema: {}",
                    proto.schema.as_str()
                )))
            }
        };

        Ok(info)
    }
}

impl std::convert::From<TeaclaveFileCryptoInfo> for proto::FileCryptoInfo {
    fn from(crypto: TeaclaveFileCryptoInfo) -> Self {
        match crypto {
            TeaclaveFileCryptoInfo::AesGcm128(info) => proto::FileCryptoInfo {
                schema: "aes_gcm_128".to_string(),
                key: info.key.to_vec(),
                iv: info.iv.to_vec(),
            },
            TeaclaveFileCryptoInfo::AesGcm256(info) => proto::FileCryptoInfo {
                schema: "aes_gcm_256".to_string(),
                key: info.key.to_vec(),
                iv: info.iv.to_vec(),
            },
            TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(info) => proto::FileCryptoInfo {
                schema: "teaclave_file_root_key_128".to_string(),
                key: info.to_vec(),
                iv: Vec::new(),
            },
        }
    }
}
