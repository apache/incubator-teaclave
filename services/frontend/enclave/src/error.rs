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

use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum AuthenticationError {
    #[error("missing user id")]
    MissingUserId,
    #[error("missing token")]
    MissingToken,
    #[error("incorrent credential")]
    IncorrectCredential,
}

impl From<AuthenticationError> for FrontendServiceError {
    fn from(error: AuthenticationError) -> Self {
        FrontendServiceError::Authentication(error)
    }
}

#[derive(Error, Debug)]
pub(crate) enum FrontendServiceError {
    #[error("permission denied")]
    PermissionDenied,
    #[error("service internal error")]
    Service(#[from] anyhow::Error),
    #[error("authentication failed")]
    Authentication(AuthenticationError),
}

impl From<FrontendServiceError> for teaclave_rpc::Status {
    fn from(error: FrontendServiceError) -> Self {
        log::debug!("FrontendServiceError: {:?}", error);
        match error {
            FrontendServiceError::PermissionDenied => {
                teaclave_rpc::Status::permission_denied("permission denied")
            }
            FrontendServiceError::Service(e) => teaclave_rpc::Status::internal(e.to_string()),
            FrontendServiceError::Authentication(e) => {
                teaclave_rpc::Status::unauthenticated(e.to_string())
            }
        }
    }
}
