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

use crate::error::SchedulerServiceError;

use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryInto;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
#[allow(unused_imports)]
use std::untrusted::time::SystemTimeEx;
use tokio::sync::Mutex;

use teaclave_proto::teaclave_common::{ExecutorCommand, ExecutorStatus};
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_rpc::transport::{channel::Endpoint, Channel};
use teaclave_rpc::{Request, Response};
use teaclave_types::*;
use uuid::Uuid;

use anyhow::anyhow;
use anyhow::Result;

const EXECUTOR_TIMEOUT_SECS: u64 = 30;

#[derive(Clone)]
pub(crate) struct TeaclaveSchedulerService {
    resources: Arc<Mutex<TeaclaveSchedulerResources>>,
}

pub struct TeaclaveSchedulerResources {
    storage_client: Arc<Mutex<TeaclaveStorageClient<Channel>>>,
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
    pub async fn run(&self) -> Result<()> {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));

            let mut resources = self.resources.lock().await;

            let key = StagedTask::get_queue_key().as_bytes();

            log::debug!("Pulling task/cancel queue");
            while let Ok(canceled_task) = resources.pull_cancel_queue().await {
                resources.tasks_to_cancel.insert(canceled_task.task_id);
            }

            while let Ok(staged_task) = resources.pull_staged_task::<StagedTask>(key).await {
                log::debug!("deamon: Pulled staged task: {:?}", staged_task);
                resources.task_queue.push_back(staged_task);
            }

            let current_time = SystemTime::now();
            let mut to_remove = Vec::new();
            for (executor_id, last_heartbeat) in resources.executors_last_heartbeat.iter() {
                if current_time
                    .duration_since(*last_heartbeat)
                    .unwrap_or_else(|_| Duration::from_secs(EXECUTOR_TIMEOUT_SECS + 1))
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
                    let ts = resources.get_task_state(&task_id).await?;
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
                    resources.put_into_db(&ts).await?;
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
    pub(crate) async fn new(storage_service_endpoint: Endpoint) -> Result<Self> {
        let channel = storage_service_endpoint
            .connect()
            .await
            .map_err(|e| anyhow!("Failed to connect to storage service.{:?}", e))?;
        let storage_client = Arc::new(Mutex::new(TeaclaveStorageClient::new(channel)));
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

    async fn pull_staged_task<T: Storable>(
        &self,
        key: &[u8],
    ) -> std::result::Result<T, SchedulerServiceError> {
        let dequeue_request = DequeueRequest::new(key);
        let dequeue_response = self
            .storage_client
            .clone()
            .lock()
            .await
            .dequeue(dequeue_request)
            .await
            .map_err(|_| SchedulerServiceError::StorageError)?
            .into_inner();
        T::from_slice(dequeue_response.value.as_slice()).map_err(SchedulerServiceError::Service)
    }

    async fn pull_cancel_queue(&self) -> std::result::Result<TaskState, SchedulerServiceError> {
        let dequeue_request = DequeueRequest::new(CANCEL_QUEUE_KEY.as_bytes());
        let dequeue_response = self
            .storage_client
            .clone()
            .lock()
            .await
            .dequeue(dequeue_request)
            .await
            .map_err(|_| SchedulerServiceError::StorageError)?
            .into_inner();
        TaskState::from_slice(dequeue_response.value.as_slice())
            .map_err(SchedulerServiceError::Service)
    }

    async fn cancel_task(&self, task_id: Uuid) -> std::result::Result<(), SchedulerServiceError> {
        let ts = self.get_task_state(&task_id).await?;
        let mut task: Task<Cancel> = ts.try_into()?;

        // Only TaskStatus::Running/Staged is allowed here.
        let result_err = TaskResult::Err(TaskFailure::new("Task Canceled by the user"));

        task.update_result(result_err)?;

        let ts = TaskState::from(task);
        self.put_into_db(&ts).await?;

        Ok(())
    }

    async fn get_task_state(&self, task_id: &Uuid) -> Result<TaskState> {
        let key = ExternalID::new(TaskState::key_prefix(), task_id.to_owned());
        self.get_from_db(&key).await
    }

    async fn get_from_db<T: Storable>(&self, key: &ExternalID) -> Result<T> {
        anyhow::ensure!(T::match_prefix(&key.prefix), "Key prefix doesn't match.");
        let get_request = GetRequest::new(key.to_bytes());
        let storage = self.storage_client.clone();
        let mut storage = storage.lock().await;

        let response = storage.get(get_request).await?.into_inner();
        T::from_slice(response.value.as_slice())
    }

    async fn put_into_db(&self, item: &impl Storable) -> Result<()> {
        let k = item.key();
        let v = item.to_vec()?;
        let put_request = PutRequest::new(k.as_slice(), v.as_slice());
        let cli = self.storage_client.clone();
        let mut client = cli.lock().await;

        let _put_response = client.put(put_request).await?;
        Ok(())
    }
}

#[teaclave_rpc::async_trait]
impl TeaclaveScheduler for TeaclaveSchedulerService {
    // Publisher
    async fn publish_task(
        &self,
        request: Request<PublishTaskRequest>,
    ) -> TeaclaveServiceResponseResult<PublishTaskResponse> {
        // XXX: Publisher is not implemented

        let mut resources = self.resources.lock().await;

        let staged_task =
            StagedTask::from_slice(&request.get_ref().staged_task).map_err(tonic_error)?;
        resources.task_queue.push_back(staged_task);
        Ok(Response::new(PublishTaskResponse {}))
    }

    // Subscriber
    async fn subscribe(
        &self,
        _request: Request<SubscribeRequest>,
    ) -> TeaclaveServiceResponseResult<SubscribeResponse> {
        // TODO: subscribe a specific topic
        unimplemented!()
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> TeaclaveServiceResponseResult<HeartbeatResponse> {
        let mut resources = self.resources.lock().await;

        let mut command = ExecutorCommand::NoAction;

        let executor_id = Uuid::parse_str(&request.get_ref().executor_id).map_err(tonic_error)?;
        let status = request.get_ref().status.try_into().map_err(tonic_error)?;

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
                        resources.cancel_task(task_id).await.map_err(tonic_error)?;
                        return Ok(Response::new(HeartbeatResponse::new(command)));
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

        let response = HeartbeatResponse::new(command);
        Ok(Response::new(response))
    }

    async fn pull_task(
        &self,
        request: Request<PullTaskRequest>,
    ) -> TeaclaveServiceResponseResult<PullTaskResponse> {
        let request = request.get_ref();
        let mut resources = self.resources.lock().await;
        match resources.task_queue.pop_front() {
            Some(task) => match resources.tasks_to_cancel.take(&task.task_id) {
                Some(task_id) => {
                    resources.cancel_task(task_id).await?;
                    Err(SchedulerServiceError::TaskCanceled.into())
                }
                None => {
                    resources.executors_tasks.insert(
                        Uuid::parse_str(&request.executor_id).map_err(tonic_error)?,
                        task.task_id,
                    );
                    Ok(Response::new(PullTaskResponse::new(task)))
                }
            },
            None => Err(SchedulerServiceError::TaskQueueEmpty.into()),
        }
    }

    async fn update_task_status(
        &self,
        request: Request<UpdateTaskStatusRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateTaskStatusResponse> {
        let resources = self.resources.lock().await;

        let task_id = Uuid::parse_str(&request.get_ref().task_id).map_err(tonic_error)?;
        let ts = resources
            .get_task_state(&task_id)
            .await
            .map_err(tonic_error)?;
        let task: Task<Run> = ts.try_into().map_err(tonic_error)?;

        log::debug!("UpdateTaskStatus: Task {:?}", task);
        // Only TaskStatus::Running is implicitly allowed here.

        let ts = TaskState::from(task);
        resources.put_into_db(&ts).await.map_err(tonic_error)?;
        Ok(Response::new(UpdateTaskStatusResponse {}))
    }

    async fn update_task_result(
        &self,
        request: Request<UpdateTaskResultRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateTaskResultResponse> {
        let resources = self.resources.lock().await;

        let request = request.into_inner();
        let ts = resources
            .get_task_state(&Uuid::parse_str(&request.task_id).map_err(tonic_error)?)
            .await
            .map_err(tonic_error)?;
        let mut task: Task<Finish> = ts.try_into().map_err(tonic_error)?;
        let task_result: TaskResult = request.result.try_into().map_err(tonic_error)?;
        if let TaskResult::Ok(outputs) = task_result.clone() {
            for (key, auth_tag) in outputs.tags_map.iter() {
                let outfile = task
                    .update_output_cmac(key, auth_tag)
                    .map_err(tonic_error)?;
                resources.put_into_db(outfile).await.map_err(tonic_error)?;
            }
        };

        // Updating task result means we have finished execution
        task.update_result(task_result).map_err(tonic_error)?;
        log::debug!("UpdateTaskResult: Task {:?}", task);

        let ts = TaskState::from(task);
        resources.put_into_db(&ts).await.map_err(tonic_error)?;
        Ok(Response::new(UpdateTaskResultResponse {}))
    }
}
