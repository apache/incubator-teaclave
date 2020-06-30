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

use std::fmt;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type SgxStatus = sgx_types::sgx_status_t;

/// Status for Ecall
#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ECallStatus(pub u32);

impl ECallStatus {
    pub fn is_err(&self) -> bool {
        self.0 != 0
    }

    pub fn is_ok(&self) -> bool {
        self.0 == 0
    }

    pub fn is_err_ffi_outbuf(&self) -> bool {
        self.0 == 0x0000_000c
    }
}

impl fmt::Display for ECallStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum TeeServiceError {
    #[error("SgxError")]
    SgxError,
    #[error("ServiceError")]
    ServiceError,
    #[error("CommandNotRegistered")]
    CommandNotRegistered,
}

pub type TeeServiceResult<T> = std::result::Result<T, TeeServiceError>;

#[derive(Error, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TeaclaveServiceResponseError {
    #[error("Request error: {0}")]
    RequestError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<anyhow::Error> for TeaclaveServiceResponseError {
    fn from(error: anyhow::Error) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

pub type TeaclaveServiceResponseResult<T> = std::result::Result<T, TeaclaveServiceResponseError>;
