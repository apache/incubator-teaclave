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
use teaclave_types::{StagedTask, TaskStatus};

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

#[into_request(TeaclaveSchedulerRequest::UpdateTaskResult)]
pub struct UpdateTaskResultRequest {
    pub task_id: String,
    pub return_value: Vec<u8>,
    pub output_file_hash: HashMap<String, String>,
}

impl UpdateTaskResultRequest {
    pub fn new(
        task_id: impl Into<String>,
        return_value: &[u8],
        output_file_hash: HashMap<String, String>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            return_value: return_value.to_vec(),
            output_file_hash,
        }
    }
}

#[into_request(TeaclaveSchedulerResponse::UpdateTaskResult)]
pub struct UpdateTaskResultResponse {}

#[into_request(TeaclaveSchedulerRequest::UpdateTaskStatus)]
pub struct UpdateTaskStatusRequest {
    pub task_id: String,
    pub task_status: TaskStatus,
}

impl UpdateTaskStatusRequest {
    pub fn new(task_id: impl Into<String>, task_status: TaskStatus) -> Self {
        Self {
            task_id: task_id.into(),
            task_status,
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
        let ret = Self {
            task_id: proto.task_id,
            return_value: proto.return_value,
            output_file_hash: proto.output_file_hash,
        };
        Ok(ret)
    }
}

impl std::convert::From<UpdateTaskResultRequest> for proto::UpdateTaskResultRequest {
    fn from(req: UpdateTaskResultRequest) -> Self {
        proto::UpdateTaskResultRequest {
            task_id: req.task_id,
            return_value: req.return_value,
            output_file_hash: req.output_file_hash,
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
            task_id: proto.task_id,
            task_status,
        };
        Ok(ret)
    }
}

impl std::convert::From<UpdateTaskStatusRequest> for proto::UpdateTaskStatusRequest {
    fn from(req: UpdateTaskStatusRequest) -> Self {
        let task_status = i32_from_task_status(req.task_status);
        proto::UpdateTaskStatusRequest {
            task_id: req.task_id,
            task_status,
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
