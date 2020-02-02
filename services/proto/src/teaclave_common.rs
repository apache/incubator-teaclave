#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use anyhow::{bail, ensure, Error, Result};
use std::format;
use teaclave_types::{
    AesGcm128CryptoInfo, AesGcm256CryptoInfo, TeaclaveFileCryptoInfo, TeaclaveFileRootKey128,
};

use crate::teaclave_common_proto as proto;

#[derive(Debug)]
pub struct UserCredential {
    pub id: std::string::String,
    pub token: std::string::String,
}

impl UserCredential {
    pub fn new<S, T>(id: S, token: T) -> Self
    where
        S: Into<String>,
        T: Into<String>,
    {
        Self {
            id: id.into(),
            token: token.into(),
        }
    }
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
                ensure!(
                    proto.iv.is_empty(),
                    "IV is not empty for teaclave_file_root_key_128"
                );
                let info = TeaclaveFileRootKey128::try_new(&proto.key)?;
                TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(info)
            }
            _ => bail!("Invalid crypto schema: {}", proto.schema.as_str()),
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
                key: info.key.to_vec(),
                iv: Vec::new(),
            },
        }
    }
}
