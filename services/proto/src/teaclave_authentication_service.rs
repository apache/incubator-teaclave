use crate::teaclave_common;
use crate::teaclave_common::proto as teaclave_common_proto;
use anyhow::anyhow;
use anyhow::{Error, Result};
use core::convert::TryInto;

pub mod proto {
    #![allow(clippy::all)]
    include!(concat!(
        env!("OUT_DIR"),
        "/teaclave_authentication_service_proto.rs"
    ));
}

pub use proto::TeaclaveAuthentication;
pub use proto::TeaclaveAuthenticationClient;
pub use proto::TeaclaveAuthenticationRequest;
pub use proto::TeaclaveAuthenticationResponse;

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct UserLoginRequest {
    pub id: std::string::String,
    pub password: std::string::String,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct UserLoginResponse {
    pub token: std::string::String,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct UserAuthorizeRequest {
    pub credential: teaclave_common::UserCredential,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct UserAuthorizeResponse {
    pub accept: bool,
}

impl std::convert::TryFrom<proto::UserLoginRequest> for UserLoginRequest {
    type Error = Error;

    fn try_from(proto: proto::UserLoginRequest) -> Result<Self> {
        let ret = Self {
            id: proto.id,
            password: proto.password,
        };

        Ok(ret)
    }
}

impl From<UserLoginRequest> for proto::UserLoginRequest {
    fn from(request: UserLoginRequest) -> Self {
        Self {
            id: request.id,
            password: request.password,
        }
    }
}

impl From<UserLoginResponse> for proto::UserLoginResponse {
    fn from(response: UserLoginResponse) -> Self {
        Self {
            token: response.token,
        }
    }
}

impl std::convert::TryFrom<proto::UserAuthorizeRequest> for UserAuthorizeRequest {
    type Error = Error;

    fn try_from(proto: proto::UserAuthorizeRequest) -> Result<Self> {
        let ret = Self {
            credential: proto
                .credential
                .ok_or_else(|| anyhow!("Missing credential"))?
                .try_into()?,
        };

        Ok(ret)
    }
}

impl From<UserAuthorizeRequest> for proto::UserAuthorizeRequest {
    fn from(request: UserAuthorizeRequest) -> Self {
        Self {
            credential: Some(request.credential.into()),
        }
    }
}

impl From<UserAuthorizeResponse> for proto::UserAuthorizeResponse {
    fn from(response: UserAuthorizeResponse) -> Self {
        Self {
            accept: response.accept,
        }
    }
}
