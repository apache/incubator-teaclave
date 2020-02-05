#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::teaclave_common_proto as proto;
use anyhow::{Error, Result};
use teaclave_types::TeaclaveFileCryptoInfo;

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
