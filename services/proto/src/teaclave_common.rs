#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::teaclave_common_proto as proto;
use anyhow::{Error, Result};
use teaclave_types::TeaclaveFileCryptoInfo;
use teaclave_types::TeaclaveFileRootKey128;

#[derive(Debug)]
pub struct UserCredential {
    pub id: std::string::String,
    pub token: std::string::String,
}

impl UserCredential {
    pub fn new(id: impl Into<String>, token: impl Into<String>) -> Self {
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
        TeaclaveFileCryptoInfo::new(&proto.schema, &proto.key, &proto.iv)
    }
}

impl std::convert::TryFrom<proto::FileCryptoInfo> for TeaclaveFileRootKey128 {
    type Error = Error;
    fn try_from(proto: proto::FileCryptoInfo) -> Result<Self> {
        let file_crypto = TeaclaveFileCryptoInfo::new(&proto.schema, &proto.key, &proto.iv)?;
        let crypto = match file_crypto {
            TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(info) => info,
            _ => anyhow::bail!("FileCryptoInfo not supported"),
        };
        Ok(crypto)
    }
}

impl std::convert::From<TeaclaveFileCryptoInfo> for proto::FileCryptoInfo {
    fn from(crypto: TeaclaveFileCryptoInfo) -> Self {
        let (key, iv) = crypto.key_iv();
        proto::FileCryptoInfo {
            schema: crypto.schema(),
            key,
            iv,
        }
    }
}

impl std::convert::From<TeaclaveFileRootKey128> for proto::FileCryptoInfo {
    fn from(crypto: TeaclaveFileRootKey128) -> Self {
        let crypto = TeaclaveFileCryptoInfo::TeaclaveFileRootKey128(crypto);
        let (key, iv) = crypto.key_iv();
        proto::FileCryptoInfo {
            schema: crypto.schema(),
            key,
            iv,
        }
    }
}
