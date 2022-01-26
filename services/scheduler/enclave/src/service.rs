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

use crate::error::TeaclaveSchedulerError;

use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryInto;
use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};
use std::time::{Duration, SystemTime};
use std::untrusted::time::SystemTimeEx;

use teaclave_proto::teaclave_common::{ExecutorCommand, ExecutorStatus};
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::*;
use uuid::Uuid;

use anyhow::anyhow;
use anyhow::Result;

const EXECUTOR_TIMEOUT_SECS: u64 = 30;

#[teaclave_service(teaclave_scheduler_service, TeaclaveScheduler, TeaclaveSchedulerError)]
#[derive(Clone)]
pub(crate) struct TeaclaveSchedulerService {
    resources: Arc<Mutex<TeaclaveSchedulerResources>>,
}

pub struct TeaclaveSchedulerResources {
    storage_client: Arc<Mutex<TeaclaveStorageClient>>,
    // map executor_id to task_id
    task_queue: VecDeque<StagedTask>,
    executors_tasks: HashMap<Uuid, Uuid>,
    executors_last_heartbeat: HashMap<Uuid, SystemTime>,
    executors_status: HashMap<Uuid, ExecutorStatus>,
    tasks_to_cancel: HashSet<Uuid>,
}

pub struct TeaclaveSchedulerDeamon {
    resources: Arc<Mutex<TeaclaveSchedulerResources>>,
}

impl TeaclaveSchedulerDeamon {
    pub fn run(&self) -> Result<()> {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));

            let mut resources = self
                .resources
                .lock()
                .map_err(|_| anyhow!("Cannot lock scheduler resources"))?;

            let key = StagedTask::get_queue_key().as_bytes();

            log::debug!("Pulling task/cancel queue");
            while let Ok(canceled_task) = resources.pull_cancel_queue() {
                resources.tasks_to_cancel.insert(canceled_task.task_id);
            }

            while let Ok(staged_task) = resources.pull_staged_task::<StagedTask>(key) {
                log::debug!("deamon: Pulled staged task: {:?}", staged_task);
                resources.task_queue.push_back(staged_task);
            }

            let current_time = SystemTime::now();
            let mut to_remove = Vec::new();
            for (executor_id, last_heartbeat) in resources.executors_last_heartbeat.iter() {
                if current_time
                    .duration_since(*last_heartbeat)
                    .unwrap_or_else( |_| Duration::from_secs(EXECUTOR_TIMEOUT_SECS + 1))
                    > Duration::from_secs(EXECUTOR_TIMEOUT_SECS)
                {
                    // executor lost
                    to_remove.push(*executor_id);
                    log::warn!("Executor {} lost", executor_id);
                }
            }

            for executor_id in to_remove {
                resources.executors_last_heartbeat.remove(&executor_id);
                resources.executors_status.remove(&executor_id);
                if let Some(task_id) = resources.executors_tasks.remove(&executor_id) {
                    // report task faliure
                    let ts = resources.get_task_state(&task_id)?;
                    if ts.is_ended() {
                        continue;
                    }

                    log::warn!("Executor {} lost, canceling task {}", executor_id, task_id);

                    let mut task: Task<Fail> = ts.try_into()?;

                    log::debug!("Task failed because of Executor lost: Task {:?}", task);
                    // Only TaskStatus::Running/Staged is allowed here.
                    let result_err =
                        TaskResult::Err(TaskFailure::new("Runtime Error: Executor Timeout"));

                    // Updating task result means we have finished execution
                    task.update_result(result_err)?;

                    let ts = TaskState::from(task);
                    resources.put_into_db(&ts)?;
                }
            }
        }
    }
}

impl TeaclaveSchedulerService {
    pub fn new(resources: &Arc<Mutex<TeaclaveSchedulerResources>>) -> Self {
        Self {
            resources: resources.clone(),
        }
    }
}

impl TeaclaveSchedulerDeamon {
    pub fn new(resources: &Arc<Mutex<TeaclaveSchedulerResources>>) -> Self {
        Self {
            resources: resources.clone(),
        }
    }
}

impl TeaclaveSchedulerResources {
    pub(crate) fn new(storage_service_endpoint: Endpoint) -> Result<Self> {
        let mut i = 0;
        let channel = loop {
            match storage_service_endpoint.connect() {
                Ok(channel) => break channel,
                Err(_) => {
                    anyhow::ensure!(i < 10, "failed to connect to storage service");
                    log::warn!("Failed to connect to storage service, retry {}", i);
                    i += 1;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(3));
        };
        let storage_client = Arc::new(Mutex::new(TeaclaveStorageClient::new(channel)?));
        let task_queue = VecDeque::new();
        let executors_tasks = HashMap::new();
        let executors_status = HashMap::new();
        let tasks_to_cancel = HashSet::new();
        let executors_last_heartbeat = HashMap::new();

        let resources = TeaclaveSchedulerResources {
            storage_client,
            task_queue,
            executors_tasks,
            executors_last_heartbeat,
            executors_status,
            tasks_to_cancel,
        };

        Ok(resources)
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

    fn pull_cancel_queue(&self) -> Result<TaskState> {
        let dequeue_request = DequeueRequest::new(CANCEL_QUEUE_KEY.as_bytes());
        let dequeue_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| TeaclaveSchedulerError::StorageError)?
            .dequeue(dequeue_request)?;
        TaskState::from_slice(dequeue_response.value.as_slice())
            .map_err(|_| TeaclaveSchedulerError::DataError.into())
    }

    fn cancel_task(&self, task_id: Uuid) -> Result<()> {
        let ts = self.get_task_state(&task_id)?;
        let mut task: Task<Cancel> = ts.try_into()?;

        // Only TaskStatus::Running/Staged is allowed here.
        let result_err = TaskResult::Err(TaskFailure::new("Task Canceled by the user"));

        task.update_result(result_err)?;

        let ts = TaskState::from(task);
        self.put_into_db(&ts)?;

        Ok(())
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

        let mut resources = self
            .resources
            .lock()
            .map_err(|_| anyhow!("Cannot lock scheduler resources"))?;

        let staged_task = request.message.staged_task;
        resources.task_queue.push_back(staged_task);
        Ok(PublishTaskResponse {})
    }

    // Subscriber
    fn subscribe(
        &self,
        _request: Request<SubscribeRequest>,
    ) -> TeaclaveServiceResponseResult<SubscribeResponse> {
        // TODO: subscribe a specific topic
        unimplemented!()
    }

    fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> TeaclaveServiceResponseResult<HeartbeatResponse> {
        let mut resources = self
            .resources
            .lock()
            .map_err(|_| anyhow!("Cannot lock scheduler resources"))?;

        let mut command = ExecutorCommand::NoAction;

        let executor_id = request.message.executor_id;
        let status = request.message.status;

        resources.executors_status.insert(executor_id, status);

        resources
            .executors_last_heartbeat
            .insert(executor_id, SystemTime::now());

        // check if the executor need to be stopped
        if let Some(task_id) = resources.executors_tasks.get(&executor_id) {
            match status {
                ExecutorStatus::Executing => {
                    if resources.tasks_to_cancel.contains(task_id) {
                        command = ExecutorCommand::Stop;
                        let task_id = task_id.to_owned();
                        resources.tasks_to_cancel.remove(&task_id);
                        log::debug!(
                            "Sending stop command to executor {}, killing executor {} because of task cancelation",
                            executor_id,
                            task_id
                        );
                        resources.cancel_task(task_id)?;
                        return Ok(HeartbeatResponse { command });
                    }
                }
                ExecutorStatus::Idle => {
                    resources.executors_tasks.remove(&executor_id);
                }
            }
        }

        if !resources.task_queue.is_empty() {
            command = ExecutorCommand::NewTask;
        }

        let response = HeartbeatResponse { command };
        Ok(response)
    }

    fn pull_task(
        &self,
        request: Request<PullTaskRequest>,
    ) -> TeaclaveServiceResponseResult<PullTaskResponse> {
        let request = request.message;
        let mut resources = self
            .resources
            .lock()
            .map_err(|_| anyhow!("Cannot lock scheduler resources"))?;

        match resources.task_queue.pop_front() {
            Some(task) => match resources.tasks_to_cancel.take(&task.task_id) {
                Some(task_id) => {
                    resources.cancel_task(task_id)?;
                    Err(TeaclaveServiceResponseError::InternalError(
                        "Task to pull has been canceled".into(),
                    ))
                }
                None => {
                    resources
                        .executors_tasks
                        .insert(request.executor_id, task.task_id);
                    Ok(PullTaskResponse::new(task))
                }
            },
            None => Err(TeaclaveServiceResponseError::InternalError(
                "No staged task in task_queue".into(),
            )),
        }
    }

    fn update_task_status(
        &self,
        request: Request<UpdateTaskStatusRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateTaskStatusResponse> {
        let resources = self
            .resources
            .lock()
            .map_err(|_| anyhow!("Cannot lock scheduler resources"))?;

        let request = request.message;
        let ts = resources.get_task_state(&request.task_id)?;
        let task: Task<Run> = ts.try_into()?;

        log::debug!("UpdateTaskStatus: Task {:?}", task);
        // Only TaskStatus::Running is implicitly allowed here.

        let ts = TaskState::from(task);
        resources.put_into_db(&ts)?;
        Ok(UpdateTaskStatusResponse {})
    }

    fn update_task_result(
        &self,
        request: Request<UpdateTaskResultRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateTaskResultResponse> {
        let resources = self
            .resources
            .lock()
            .map_err(|_| anyhow!("Cannot lock scheduler resources"))?;

        let request = request.message;
        let ts = resources.get_task_state(&request.task_id)?;
        let mut task: Task<Finish> = ts.try_into()?;

        if let TaskResult::Ok(outputs) = &request.task_result {
            for (key, auth_tag) in outputs.tags_map.iter() {
                let outfile = task.update_output_cmac(key, auth_tag)?;
                resources.put_into_db(outfile)?;
            }
        };

        // Updating task result means we have finished execution
        task.update_result(request.task_result)?;
        log::debug!("UpdateTaskResult: Task {:?}", task);

        let ts = TaskState::from(task);
        resources.put_into_db(&ts)?;
        Ok(UpdateTaskResultResponse {})
    }
}

#[cfg(test_mode)]
mod test_mode {
    use super::*;
}
