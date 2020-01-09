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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use std::sync::Arc;

use teaclave_proto::teaclave_execution_service::{
    self, StagedFunctionExecuteRequest, StagedFunctionExecuteResponse, TeaclaveExecution,
};
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::{TeaclaveServiceResponseError, TeaclaveServiceResponseResult};


use thiserror::Error;
use crate::Worker;

#[derive(Error, Debug)]
pub enum TeaclaveExecutionError {
    #[error("woker running spec error")]
    WorkerRunningSpecError,
}

impl From<TeaclaveExecutionError> for TeaclaveServiceResponseError {
    fn from(error: TeaclaveExecutionError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

#[teaclave_service(teaclave_execution_service, TeaclaveExecution, TeaclaveExecutionError)]
#[derive(Clone)]
pub(crate) struct TeaclaveExecutionService {
    worker: Arc<Worker>,
}

impl TeaclaveExecutionService {
    pub(crate) fn new() -> Self {
        TeaclaveExecutionService {
            worker: Arc::new(Worker::new())
        }
    }
}

impl TeaclaveExecution for TeaclaveExecutionService {
    fn invoke_function(
        &self,
        request: StagedFunctionExecuteRequest,
    ) -> TeaclaveServiceResponseResult<StagedFunctionExecuteResponse> {
        match self.worker.invoke_function(&request) {
            Ok(summary) => {
                info!("[+] Invoking function ok: {}", summary);
                Ok(summary.into())
            },
            Err(e) => {
                error!("[+] Invoking function failed: {}", e);
                Err(TeaclaveExecutionError::WorkerRunningSpecError.into())
            }

        }
    }
}

#[cfg(test_mode)]
mod test_mode {
    use super::*;
}
