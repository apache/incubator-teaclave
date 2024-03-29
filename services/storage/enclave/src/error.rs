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

use teaclave_rpc::{Code, Status};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum StorageServiceError {
    #[error("none")]
    None,
    #[error("leveldb error")]
    Database(#[from] rusty_leveldb::Status),
    #[error("service internal error")]
    Service(#[from] anyhow::Error),
}

impl From<StorageServiceError> for teaclave_rpc::Status {
    fn from(error: StorageServiceError) -> Self {
        log::debug!("StorageServiceError: {:?}", error);
        let msg = error.to_string();
        let code = match error {
            StorageServiceError::Service(_) => Code::Internal,
            _ => Code::Unknown,
        };
        Status::new(code, msg)
    }
}
