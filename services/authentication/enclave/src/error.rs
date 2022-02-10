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

use std::prelude::v1::*;

use teaclave_types::TeaclaveServiceResponseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum AuthenticationError {
    #[error("invalid user id")]
    InvalidUserId,
    #[error("invalid token")]
    InvalidToken,
    #[error("invalid password")]
    InvalidPassword,
    #[error("user id not found")]
    UserIdNotFound,
    #[error("incorrect password")]
    IncorrectPassword,
    #[error("incorrect token")]
    IncorrectToken,
}

impl From<AuthenticationError> for AuthenticationServiceError {
    fn from(error: AuthenticationError) -> Self {
        AuthenticationServiceError::Authentication(error)
    }
}

impl From<AuthenticationError> for TeaclaveServiceResponseError {
    fn from(error: AuthenticationError) -> Self {
        TeaclaveServiceResponseError::RequestError(
            AuthenticationServiceError::from(error).to_string(),
        )
    }
}

#[derive(Error, Debug)]
pub(crate) enum AuthenticationServiceError {
    #[error("permission denied")]
    PermissionDenied,
    #[error("authentication failed")]
    Authentication(AuthenticationError),
    #[error("invalid user id")]
    InvalidUserId,
    #[error("invalid role")]
    InvalidRole,
    #[error("user id exist")]
    UserIdExist,
    #[error("service internal error")]
    Service(#[from] anyhow::Error),
    #[error("missing user id")]
    MissingUserId,
    #[error("missing token")]
    MissingToken,
}

impl From<AuthenticationServiceError> for TeaclaveServiceResponseError {
    fn from(error: AuthenticationServiceError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}
