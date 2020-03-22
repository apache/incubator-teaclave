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

use anyhow::anyhow;
use anyhow::{Error, Result};
use core::convert::TryInto;
use std::prelude::v1::*;
use teaclave_rpc::into_request;

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

#[into_request(TeaclaveAuthenticationApiRequest::UserRegister)]
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

#[into_request(TeaclaveAuthenticationApiResponse::UserRegister)]
#[derive(Debug, Default)]
pub struct UserRegisterResponse;

#[into_request(TeaclaveAuthenticationApiRequest::UserLogin)]
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

#[into_request(TeaclaveAuthenticationApiResponse::UserLogin)]
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

#[into_request(TeaclaveAuthenticationInternalRequest::UserAuthenticate)]
#[derive(Debug)]
pub struct UserAuthenticateRequest {
    pub credential: teaclave_common::UserCredential,
}

impl UserAuthenticateRequest {
    pub fn new(credential: teaclave_common::UserCredential) -> Self {
        Self { credential }
    }
}

#[into_request(TeaclaveAuthenticationInternalResponse::UserAuthenticate)]
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

impl std::convert::TryFrom<proto::UserLoginResponse> for UserLoginResponse {
    type Error = Error;

    fn try_from(proto: proto::UserLoginResponse) -> Result<Self> {
        let ret = Self { token: proto.token };

        Ok(ret)
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

impl std::convert::TryFrom<proto::UserAuthenticateResponse> for UserAuthenticateResponse {
    type Error = Error;

    fn try_from(proto: proto::UserAuthenticateResponse) -> Result<Self> {
        let ret = Self {
            accept: proto.accept,
        };

        Ok(ret)
    }
}

impl From<UserAuthenticateResponse> for proto::UserAuthenticateResponse {
    fn from(response: UserAuthenticateResponse) -> Self {
        Self {
            accept: response.accept,
        }
    }
}
