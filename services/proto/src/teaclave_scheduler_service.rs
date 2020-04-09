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

use std::collections::HashMap;
use std::prelude::v1::*;

use crate::teaclave_common::{i32_from_task_status, i32_to_task_status};
use crate::teaclave_scheduler_service_proto as proto;
use anyhow::{Error, Result};
use core::convert::TryInto;
pub use proto::TeaclaveScheduler;
pub use proto::TeaclaveSchedulerClient;
pub use proto::TeaclaveSchedulerRequest;
pub use proto::TeaclaveSchedulerResponse;
use teaclave_rpc::into_request;
use teaclave_types::{StagedTask, TaskFailure, TaskOutputs, TaskStatus};
use uuid::Uuid;

#[into_request(TeaclaveSchedulerRequest::Subscribe)]
pub struct SubscribeRequest {}

#[into_request(TeaclaveSchedulerResponse::Subscribe)]
pub struct SubscribeResponse {
    pub success: bool,
}

#[into_request(TeaclaveSchedulerRequest::PullTask)]
pub struct PullTaskRequest {}

#[into_request(TeaclaveSchedulerResponse::PullTask)]
#[derive(Debug)]
pub struct PullTaskResponse {
    pub staged_task: StagedTask,
}

impl PullTaskResponse {
    pub fn new(staged_task: StagedTask) -> Self {
        Self { staged_task }
    }
}

pub type TaskResult = std::result::Result<TaskOutputs, TaskFailure>;

#[into_request(TeaclaveSchedulerRequest::UpdateTaskResult)]
pub struct UpdateTaskResultRequest {
    pub task_id: Uuid,
    pub task_result: TaskResult,
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
            task_id,
            task_result: result,
        }
    }
}

#[into_request(TeaclaveSchedulerResponse::UpdateTaskResult)]
pub struct UpdateTaskResultResponse {}

#[into_request(TeaclaveSchedulerRequest::UpdateTaskStatus)]
pub struct UpdateTaskStatusRequest {
    pub task_id: Uuid,
    pub task_status: TaskStatus,
    pub status_info: String,
}

impl UpdateTaskStatusRequest {
    pub fn new(task_id: Uuid, task_status: TaskStatus, status_info: String) -> Self {
        Self {
            task_id,
            task_status,
            status_info,
        }
    }
}
#[into_request(TeaclaveSchedulerResponse::UpdateTaskStatus)]
pub struct UpdateTaskStatusResponse {}

#[into_request(TeaclaveSchedulerRequest::PublishTask)]
pub struct PublishTaskRequest {
    pub staged_task: StagedTask,
}

#[into_request(TeaclaveSchedulerResponse::PublishTask)]
pub struct PublishTaskResponse {}

impl std::convert::TryFrom<proto::SubscribeRequest> for SubscribeRequest {
    type Error = Error;
    fn try_from(proto: proto::SubscribeRequest) -> Result<Self> {
        let ret = Self {};
        Ok(ret)
    }
}

impl std::convert::From<SubscribeRequest> for proto::SubscribeRequest {
    fn from(req: SubscribeRequest) -> Self {
        proto::SubscribeRequest {}
    }
}

impl std::convert::TryFrom<proto::SubscribeResponse> for SubscribeResponse {
    type Error = Error;
    fn try_from(proto: proto::SubscribeResponse) -> Result<Self> {
        let ret = Self {
            success: proto.success,
        };
        Ok(ret)
    }
}

impl std::convert::From<SubscribeResponse> for proto::SubscribeResponse {
    fn from(req: SubscribeResponse) -> Self {
        proto::SubscribeResponse {
            success: req.success,
        }
    }
}

impl std::convert::TryFrom<proto::PullTaskRequest> for PullTaskRequest {
    type Error = Error;
    fn try_from(proto: proto::PullTaskRequest) -> Result<Self> {
        let ret = Self {};
        Ok(ret)
    }
}

impl std::convert::From<PullTaskRequest> for proto::PullTaskRequest {
    fn from(req: PullTaskRequest) -> Self {
        proto::PullTaskRequest {}
    }
}

impl std::convert::TryFrom<proto::PullTaskResponse> for PullTaskResponse {
    type Error = Error;
    fn try_from(proto: proto::PullTaskResponse) -> Result<Self> {
        let staged_task = StagedTask::from_slice(&proto.staged_task)?;
        let ret = Self { staged_task };
        Ok(ret)
    }
}

impl std::convert::From<PullTaskResponse> for proto::PullTaskResponse {
    fn from(req: PullTaskResponse) -> Self {
        proto::PullTaskResponse {
            staged_task: req.staged_task.to_vec().unwrap(),
        }
    }
}

impl std::convert::TryFrom<proto::UpdateTaskResultRequest> for UpdateTaskResultRequest {
    type Error = Error;
    fn try_from(proto: proto::UpdateTaskResultRequest) -> Result<Self> {
        let task_id = Uuid::parse_str(&proto.task_id)?;
        let proto_result = proto
            .task_result
            .ok_or_else(|| anyhow::anyhow!("task result is empty"))?;
        let task_result = match proto_result {
            proto::update_task_result_request::TaskResult::Ok(task_outputs) => {
                let outputs_info = task_outputs.try_into()?;
                Ok(outputs_info)
            }
            proto::update_task_result_request::TaskResult::Err(task_failure) => {
                let failure_info = task_failure.try_into()?;
                Err(failure_info)
            }
        };

        let ret = Self {
            task_id,
            task_result,
        };
        Ok(ret)
    }
}

impl std::convert::From<UpdateTaskResultRequest> for proto::UpdateTaskResultRequest {
    fn from(req: UpdateTaskResultRequest) -> Self {
        let task_result = match req.task_result {
            Ok(task_outputs) => {
                proto::update_task_result_request::TaskResult::Ok(task_outputs.into())
            }
            Err(task_failure) => {
                proto::update_task_result_request::TaskResult::Err(task_failure.into())
            }
        };

        proto::UpdateTaskResultRequest {
            task_id: req.task_id.to_string(),
            task_result: Some(task_result),
        }
    }
}

impl std::convert::TryFrom<proto::UpdateTaskResultResponse> for UpdateTaskResultResponse {
    type Error = Error;
    fn try_from(proto: proto::UpdateTaskResultResponse) -> Result<Self> {
        let ret = Self {};
        Ok(ret)
    }
}

impl std::convert::From<UpdateTaskResultResponse> for proto::UpdateTaskResultResponse {
    fn from(req: UpdateTaskResultResponse) -> Self {
        proto::UpdateTaskResultResponse {}
    }
}

impl std::convert::TryFrom<proto::UpdateTaskStatusRequest> for UpdateTaskStatusRequest {
    type Error = Error;
    fn try_from(proto: proto::UpdateTaskStatusRequest) -> Result<Self> {
        let task_status = i32_to_task_status(proto.task_status)?;
        let ret = Self {
            task_id: Uuid::parse_str(&proto.task_id)?,
            task_status,
            status_info: proto.status_info,
        };
        Ok(ret)
    }
}

impl std::convert::From<UpdateTaskStatusRequest> for proto::UpdateTaskStatusRequest {
    fn from(req: UpdateTaskStatusRequest) -> Self {
        let task_status = i32_from_task_status(req.task_status);
        proto::UpdateTaskStatusRequest {
            task_id: req.task_id.to_string(),
            task_status,
            status_info: req.status_info,
        }
    }
}

impl std::convert::TryFrom<proto::UpdateTaskStatusResponse> for UpdateTaskStatusResponse {
    type Error = Error;
    fn try_from(proto: proto::UpdateTaskStatusResponse) -> Result<Self> {
        let ret = Self {};
        Ok(ret)
    }
}

impl std::convert::From<UpdateTaskStatusResponse> for proto::UpdateTaskStatusResponse {
    fn from(req: UpdateTaskStatusResponse) -> Self {
        proto::UpdateTaskStatusResponse {}
    }
}

use teaclave_types::Storable;
impl std::convert::TryFrom<proto::PublishTaskRequest> for PublishTaskRequest {
    type Error = Error;
    fn try_from(proto: proto::PublishTaskRequest) -> Result<Self> {
        let staged_task = StagedTask::from_slice(&proto.staged_task)?;
        let ret = Self { staged_task };
        Ok(ret)
    }
}

impl std::convert::From<PublishTaskRequest> for proto::PublishTaskRequest {
    fn from(req: PublishTaskRequest) -> Self {
        proto::PublishTaskRequest {
            staged_task: req.staged_task.to_vec().unwrap(),
        }
    }
}

impl std::convert::TryFrom<proto::PublishTaskResponse> for PublishTaskResponse {
    type Error = Error;
    fn try_from(proto: proto::PublishTaskResponse) -> Result<Self> {
        let ret = Self {};
        Ok(ret)
    }
}

impl std::convert::From<PublishTaskResponse> for proto::PublishTaskResponse {
    fn from(req: PublishTaskResponse) -> Self {
        proto::PublishTaskResponse {}
    }
}
