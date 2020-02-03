use anyhow::anyhow;
use anyhow::{Error, Result};
use core::convert::TryInto;
use std::prelude::v1::*;

use crate::teaclave_authentication_service_proto as proto;
use crate::teaclave_common;
pub use proto::TeaclaveAuthenticationApi;
pub use proto::TeaclaveAuthenticationApiClient;
pub use proto::TeaclaveAuthenticationApiRequest;
pub use proto::TeaclaveAuthenticationApiResponse;
pub use proto::TeaclaveAuthenticationInternal;
pub use proto::TeaclaveAuthenticationInternalClient;
pub use proto::TeaclaveAuthenticationInternalRequest;
pub use proto::TeaclaveAuthenticationInternalResponse;

#[derive(Debug)]
pub struct UserRegisterRequest {
    pub id: std::string::String,
    pub password: std::string::String,
}

impl UserRegisterRequest {
    pub fn new(id: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            password: password.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct UserRegisterResponse;

#[derive(Debug)]
pub struct UserLoginRequest {
    pub id: std::string::String,
    pub password: std::string::String,
}

impl UserLoginRequest {
    pub fn new(id: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            password: password.into(),
        }
    }
}

#[derive(Debug)]
pub struct UserLoginResponse {
    pub token: std::string::String,
}

impl UserLoginResponse {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

#[derive(Debug)]
pub struct UserAuthenticateRequest {
    pub credential: teaclave_common::UserCredential,
}

impl UserAuthenticateRequest {
    pub fn new(credential: teaclave_common::UserCredential) -> Self {
        Self { credential }
    }
}

#[derive(Debug)]
pub struct UserAuthenticateResponse {
    pub accept: bool,
}

impl UserAuthenticateResponse {
    pub fn new(accept: bool) -> Self {
        Self { accept }
    }
}

impl std::convert::TryFrom<proto::UserRegisterRequest> for UserRegisterRequest {
    type Error = Error;

    fn try_from(proto: proto::UserRegisterRequest) -> Result<Self> {
        let ret = Self {
            id: proto.id,
            password: proto.password,
        };

        Ok(ret)
    }
}

impl From<UserRegisterRequest> for proto::UserRegisterRequest {
    fn from(request: UserRegisterRequest) -> Self {
        Self {
            id: request.id,
            password: request.password,
        }
    }
}

impl std::convert::TryFrom<proto::UserRegisterResponse> for UserRegisterResponse {
    type Error = Error;

    fn try_from(_reponse: proto::UserRegisterResponse) -> Result<Self> {
        Ok(Self {})
    }
}

impl From<UserRegisterResponse> for proto::UserRegisterResponse {
    fn from(_response: UserRegisterResponse) -> Self {
        Self {}
    }
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

impl std::convert::TryFrom<proto::UserAuthenticateRequest> for UserAuthenticateRequest {
    type Error = Error;

    fn try_from(proto: proto::UserAuthenticateRequest) -> Result<Self> {
        let ret = Self {
            credential: proto
                .credential
                .ok_or_else(|| anyhow!("Missing credential"))?
                .try_into()?,
        };

        Ok(ret)
    }
}

impl From<UserAuthenticateRequest> for proto::UserAuthenticateRequest {
    fn from(request: UserAuthenticateRequest) -> Self {
        Self {
            credential: Some(request.credential.into()),
        }
    }
}

impl From<UserAuthenticateResponse> for proto::UserAuthenticateResponse {
    fn from(response: UserAuthenticateResponse) -> Self {
        Self {
            accept: response.accept,
        }
    }
}
