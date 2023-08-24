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

use crate::teaclave_common::{i32_from_task_status, ExecutorCommand, ExecutorStatus};
use crate::teaclave_scheduler_service_proto as proto;
use anyhow::Result;
pub use proto::teaclave_scheduler_client::TeaclaveSchedulerClient;
pub use proto::teaclave_scheduler_server::TeaclaveScheduler;
pub use proto::teaclave_scheduler_server::TeaclaveSchedulerServer;
pub use proto::{
    HeartbeatRequest, PublishTaskRequest, PullTaskRequest, UpdateTaskResultRequest,
    UpdateTaskStatusRequest,
};
pub use proto::{HeartbeatResponse, PullTaskResponse, SubscribeResponse};
use teaclave_types::Storable;
use teaclave_types::{StagedTask, TaskFailure, TaskOutputs, TaskResult, TaskStatus};
use uuid::Uuid;

impl_custom_server!(TeaclaveSchedulerServer, TeaclaveScheduler);
impl_custom_client!(TeaclaveSchedulerClient);

impl HeartbeatRequest {
    pub fn new(executor_id: Uuid, status: ExecutorStatus) -> Self {
        Self {
            executor_id: executor_id.to_string(),
            status: status.into(),
        }
    }
}

impl HeartbeatResponse {
    pub fn new(command: ExecutorCommand) -> Self {
        Self {
            command: command.into(),
        }
    }
}

impl PullTaskResponse {
    pub fn new(staged_task: StagedTask) -> Self {
        Self {
            staged_task: staged_task.to_vec().unwrap(),
        }
    }
}

impl UpdateTaskResultRequest {
    pub fn new(task_id: Uuid, task_result: Result<TaskOutputs>) -> Self {
        let result = match task_result {
            Ok(task_output) => TaskResult::Ok(task_output),
            Err(e) => TaskResult::Err(TaskFailure {
                reason: e.to_string(),
            }),
        };
        Self {
            task_id: task_id.to_string(),
            result: Some(result.into()),
        }
    }
}

impl UpdateTaskStatusRequest {
    pub fn new(task_id: Uuid, task_status: TaskStatus) -> Self {
        let task_status = i32_from_task_status(task_status);
        Self {
            task_id: task_id.to_string(),
            task_status,
        }
    }
}
