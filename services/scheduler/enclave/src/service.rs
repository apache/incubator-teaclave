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

#![allow(unused_imports)]
#![allow(unused_variables)]

use std::collections::VecDeque;
use std::convert::TryInto;
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};

use std::collections::HashMap;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::*;
use uuid::Uuid;

use anyhow::anyhow;
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
    task_queue: Arc<Mutex<VecDeque<StagedTask>>>,
}

impl TeaclaveSchedulerService {
    pub(crate) fn new(storage_service_endpoint: Endpoint) -> Result<Self> {
        let mut i = 0;
        let channel = loop {
            match storage_service_endpoint.connect() {
                Ok(channel) => break channel,
                Err(_) => {
                    anyhow::ensure!(i < 10, "failed to connect to storage service");
                    log::debug!("Failed to connect to storage service, retry {}", i);
                    i += 1;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(3));
        };
        let storage_client = Arc::new(Mutex::new(TeaclaveStorageClient::new(channel)?));
        let task_queue = Arc::new(Mutex::new(VecDeque::new()));
        let service = Self {
            storage_client,
            task_queue,
        };

        Ok(service)
    }

    fn pull_staged_task<T: Storable>(&self, key: &[u8]) -> TeaclaveServiceResponseResult<T> {
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

    fn get_task_state(&self, task_id: &Uuid) -> Result<TaskState> {
        let key = ExternalID::new(TaskState::key_prefix(), task_id.to_owned());
        self.get_from_db(&key)
    }

    fn get_from_db<T: Storable>(&self, key: &ExternalID) -> Result<T> {
        anyhow::ensure!(T::match_prefix(&key.prefix), "Key prefix doesn't match.");
        let get_request = GetRequest::new(key.to_bytes());
        let response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("Cannot lock storage client"))?
            .get(get_request)?;
        T::from_slice(response.value.as_slice())
    }

    fn put_into_db(&self, item: &impl Storable) -> Result<()> {
        let k = item.key();
        let v = item.to_vec()?;
        let put_request = PutRequest::new(k.as_slice(), v.as_slice());
        let _put_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("Cannot lock storage client"))?
            .put(put_request)?;
        Ok(())
    }
}

impl TeaclaveScheduler for TeaclaveSchedulerService {
    // Publisher
    fn publish_task(
        &self,
        request: Request<PublishTaskRequest>,
    ) -> TeaclaveServiceResponseResult<PublishTaskResponse> {
        // XXX: Publisher is not implemented
        let mut task_queue = self
            .task_queue
            .lock()
            .map_err(|_| anyhow!("Cannot lock task queue"))?;
        let staged_task = request.message.staged_task;
        task_queue.push_back(staged_task);
        Ok(PublishTaskResponse {})
    }

    // Subscriber
    fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> TeaclaveServiceResponseResult<SubscribeResponse> {
        // TODO: subscribe a specific topic
        unimplemented!()
    }

    fn pull_task(
        &self,
        request: Request<PullTaskRequest>,
    ) -> TeaclaveServiceResponseResult<PullTaskResponse> {
        let key = StagedTask::get_queue_key().as_bytes();
        let staged_task = self.pull_staged_task(key)?;
        let response = PullTaskResponse::new(staged_task);
        Ok(response)
    }

    fn update_task_status(
        &self,
        request: Request<UpdateTaskStatusRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateTaskStatusResponse> {
        let request = request.message;
        let ts = self.get_task_state(&request.task_id)?;
        let task: Task<Run> = ts.try_into()?;

        log::info!("UpdateTaskStatus: Task {:?}", task);
        // Only TaskStatus::Running is implicitly allowed here.

        let ts = TaskState::from(task);
        self.put_into_db(&ts)?;
        Ok(UpdateTaskStatusResponse {})
    }

    fn update_task_result(
        &self,
        request: Request<UpdateTaskResultRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateTaskResultResponse> {
        let request = request.message;
        let ts = self.get_task_state(&request.task_id)?;
        let mut task: Task<Finish> = ts.try_into()?;

        if let TaskResult::Ok(outputs) = &request.task_result {
            for (key, auth_tag) in outputs.tags_map.iter() {
                let outfile = task.update_output_cmac(key, auth_tag)?;
                self.put_into_db(outfile)?;
            }
        };

        // Updating task result means we have finished execution
        task.update_result(request.task_result)?;
        log::info!("UpdateTaskResult: Task {:?}", task);

        let ts = TaskState::from(task);
        self.put_into_db(&ts)?;
        Ok(UpdateTaskResultResponse {})
    }
}

#[cfg(test_mode)]
mod test_mode {
    use super::*;
}
