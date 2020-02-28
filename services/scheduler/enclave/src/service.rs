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
use std::sync::{Arc, SgxMutex as Mutex};

use teaclave_proto::teaclave_scheduler_service::{
    QueryTaskRequest, QueryTaskResponse, TeaclaveScheduler, UploadTaskResultRequest,
    UploadTaskResultResponse,
};
use teaclave_proto::teaclave_storage_service::{DequeueRequest, TeaclaveStorageClient};
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::{
    StagedTask, Storable, TeaclaveServiceResponseError, TeaclaveServiceResponseResult,
};

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TeaclaveSchedulerError {
    #[error("scheduler service error")]
    SchedulerServiceErr,
    #[error("data error")]
    DataError,
    #[error("storage error")]
    StorageError,
}

impl From<TeaclaveSchedulerError> for TeaclaveServiceResponseError {
    fn from(error: TeaclaveSchedulerError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

#[teaclave_service(teaclave_scheduler_service, TeaclaveScheduler, TeaclaveSchedulerError)]
#[derive(Clone)]
pub(crate) struct TeaclaveSchedulerService {
    storage_client: Arc<Mutex<TeaclaveStorageClient>>,
}

impl TeaclaveSchedulerService {
    pub(crate) fn new(storage_service_endpoint: Endpoint) -> Result<Self> {
        let channel = storage_service_endpoint.connect()?;
        let client = TeaclaveStorageClient::new(channel)?;
        let service = Self {
            storage_client: Arc::new(Mutex::new(client)),
        };
        Ok(service)
    }

    fn dequeue_from_db<T: Storable>(&self, key: &[u8]) -> TeaclaveServiceResponseResult<T> {
        let dequeue_request = DequeueRequest::new(key);
        let dequeue_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| TeaclaveSchedulerError::StorageError)?
            .dequeue(dequeue_request)?;
        T::from_slice(dequeue_response.value.as_slice())
            .map_err(|_| TeaclaveSchedulerError::DataError.into())
    }
}

impl TeaclaveScheduler for TeaclaveSchedulerService {
    fn query_task(
        &self,
        request: Request<QueryTaskRequest>,
    ) -> TeaclaveServiceResponseResult<QueryTaskResponse> {
        let request = request.message;
        let _worker_id = request.worker_id;
        let key = StagedTask::get_queue_key().as_bytes();
        let task: TeaclaveServiceResponseResult<StagedTask> = self.dequeue_from_db(key);
        let response = match task {
            Ok(_task) => unimplemented!(),
            Err(_) => QueryTaskResponse {
                function_execute_request: None,
                staged_task_id: "".to_owned(),
            },
        };
        Ok(response)
    }
    fn upload_task_result(
        &self,
        _request: Request<UploadTaskResultRequest>,
    ) -> TeaclaveServiceResponseResult<UploadTaskResultResponse> {
        unimplemented!();
    }
}

#[cfg(test_mode)]
mod test_mode {
    use super::*;
}
