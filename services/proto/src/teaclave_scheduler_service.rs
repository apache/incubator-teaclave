#![allow(unused_imports)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::prelude::v1::*;

use crate::teaclave_execution_service::StagedFunctionExecuteRequest;
use crate::teaclave_scheduler_service_proto as proto;
use anyhow::{Error, Result};
use core::convert::TryInto;
pub use proto::TeaclaveScheduler;
pub use proto::TeaclaveSchedulerClient;
pub use proto::TeaclaveSchedulerRequest;
pub use proto::TeaclaveSchedulerResponse;
use teaclave_rpc::into_request;
use teaclave_types::StagedTask;

#[into_request(TeaclaveSchedulerRequest::Subscribe)]
pub struct SubscribeRequest {}

#[into_request(TeaclaveSchedulerResponse::Subscribe)]
pub struct SubscribeResponse {
    pub success: bool,
}

#[into_request(TeaclaveSchedulerRequest::PullTask)]
pub struct PullTaskRequest {}

#[into_request(TeaclaveSchedulerResponse::PullTask)]
pub struct PullTaskResponse {}

#[into_request(TeaclaveSchedulerRequest::UpdateTask)]
pub struct UpdateTaskRequest {
    pub staged_task_id: String,
}

#[into_request(TeaclaveSchedulerResponse::UpdateTask)]
pub struct UpdateTaskResponse {}

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
        let ret = Self {};
        Ok(ret)
    }
}

impl std::convert::From<PullTaskResponse> for proto::PullTaskResponse {
    fn from(req: PullTaskResponse) -> Self {
        proto::PullTaskResponse {}
    }
}

impl std::convert::TryFrom<proto::UpdateTaskRequest> for UpdateTaskRequest {
    type Error = Error;
    fn try_from(proto: proto::UpdateTaskRequest) -> Result<Self> {
        let ret = Self {
            staged_task_id: proto.staged_task_id,
        };
        Ok(ret)
    }
}

impl std::convert::From<UpdateTaskRequest> for proto::UpdateTaskRequest {
    fn from(req: UpdateTaskRequest) -> Self {
        proto::UpdateTaskRequest {
            staged_task_id: req.staged_task_id,
        }
    }
}

impl std::convert::TryFrom<proto::UpdateTaskResponse> for UpdateTaskResponse {
    type Error = Error;
    fn try_from(proto: proto::UpdateTaskResponse) -> Result<Self> {
        let ret = Self {};
        Ok(ret)
    }
}

impl std::convert::From<UpdateTaskResponse> for proto::UpdateTaskResponse {
    fn from(req: UpdateTaskResponse) -> Self {
        proto::UpdateTaskResponse {}
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
