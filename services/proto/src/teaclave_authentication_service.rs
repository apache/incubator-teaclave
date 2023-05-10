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

use anyhow::{Error, Result};

use crate::teaclave_authentication_service_proto as proto;
use crate::teaclave_common;
pub use proto::teaclave_authentication_api_client::TeaclaveAuthenticationApiClient;
pub use proto::teaclave_authentication_api_server::TeaclaveAuthenticationApi;
pub use proto::teaclave_authentication_api_server::TeaclaveAuthenticationApiServer;
pub use proto::teaclave_authentication_internal_client::TeaclaveAuthenticationInternalClient;
pub use proto::teaclave_authentication_internal_server::{
    TeaclaveAuthenticationInternal, TeaclaveAuthenticationInternalServer,
};
pub use proto::*;
use teaclave_types::UserAuthClaims;

impl UserRegisterRequest {
    pub fn new(
        id: impl Into<String>,
        password: impl Into<String>,
        role: impl Into<String>,
        attribute: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            password: password.into(),
            role: role.into(),
            attribute: attribute.into(),
        }
    }
}

impl UserUpdateRequest {
    pub fn new(
        id: impl Into<String>,
        password: impl Into<String>,
        role: impl Into<String>,
        attribute: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            password: password.into(),
            role: role.into(),
            attribute: attribute.into(),
        }
    }
}

impl UserLoginRequest {
    pub fn new(id: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            password: password.into(),
        }
    }
}

impl UserLoginResponse {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl UserChangePasswordRequest {
    pub fn new(password: impl Into<String>) -> Self {
        Self {
            password: password.into(),
        }
    }
}

impl ResetUserPasswordRequest {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl DeleteUserRequest {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl ListUsersRequest {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl ListUsersResponse {
    pub fn new(ids: Vec<std::string::String>) -> Self {
        Self { ids }
    }
}

impl UserAuthenticateRequest {
    pub fn new(credential: teaclave_common::UserCredential) -> Self {
        Self {
            credential: Some(credential),
        }
    }
}

impl UserAuthenticateResponse {
    pub fn new(claims: UserAuthClaims) -> Self {
        Self {
            claims: Some(claims.into()),
        }
    }
}

impl std::convert::TryFrom<proto::UserAuthClaims> for UserAuthClaims {
    type Error = Error;

    fn try_from(proto: proto::UserAuthClaims) -> Result<Self> {
        let ret = Self {
            sub: proto.sub,
            role: proto.role,
            iss: proto.iss,
            exp: proto.exp,
        };

        Ok(ret)
    }
}

impl From<UserAuthClaims> for proto::UserAuthClaims {
    fn from(request: UserAuthClaims) -> Self {
        Self {
            sub: request.sub,
            role: request.role,
            iss: request.iss,
            exp: request.exp,
        }
    }
}
