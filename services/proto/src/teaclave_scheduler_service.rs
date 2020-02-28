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

#[into_request(TeaclaveSchedulerRequest::QueryTask)]
#[derive(Debug)]
pub struct QueryTaskRequest {
    pub worker_id: String,
}

#[into_request(TeaclaveSchedulerResponse::QueryTask)]
#[derive(Debug)]
pub struct QueryTaskResponse {
    pub function_execute_request: Option<StagedFunctionExecuteRequest>,
    pub staged_task_id: String,
}

#[into_request(TeaclaveSchedulerRequest::UploadTaskResult)]
pub struct UploadTaskResultRequest {
    pub success: bool,
    pub staged_task_id: String,
    pub worker_id: String,
    pub output_results: HashMap<String, String>,
}

#[into_request(TeaclaveSchedulerResponse::UploadTaskResult)]
pub struct UploadTaskResultResponse;

impl std::convert::TryFrom<proto::QueryTaskRequest> for QueryTaskRequest {
    type Error = Error;
    fn try_from(proto: proto::QueryTaskRequest) -> Result<Self> {
        let ret = Self {
            worker_id: proto.worker_id,
        };
        Ok(ret)
    }
}

impl std::convert::From<QueryTaskRequest> for proto::QueryTaskRequest {
    fn from(req: QueryTaskRequest) -> Self {
        proto::QueryTaskRequest {
            worker_id: req.worker_id,
        }
    }
}

impl std::convert::TryFrom<proto::QueryTaskResponse> for QueryTaskResponse {
    type Error = Error;
    fn try_from(proto: proto::QueryTaskResponse) -> Result<Self> {
        let function_execute_request = match proto.function_execute_request {
            Some(v) => Some(v.try_into()?),
            None => None,
        };
        let ret = Self {
            function_execute_request,
            staged_task_id: proto.staged_task_id,
        };
        Ok(ret)
    }
}

impl std::convert::From<QueryTaskResponse> for proto::QueryTaskResponse {
    fn from(resp: QueryTaskResponse) -> Self {
        proto::QueryTaskResponse {
            function_execute_request: resp.function_execute_request.map(|v| v.into()),
            staged_task_id: resp.staged_task_id,
        }
    }
}

impl std::convert::TryFrom<proto::UploadTaskResultRequest> for UploadTaskResultRequest {
    type Error = Error;
    fn try_from(proto: proto::UploadTaskResultRequest) -> Result<Self> {
        let ret = Self {
            success: proto.success,
            staged_task_id: proto.staged_task_id,
            worker_id: proto.worker_id,
            output_results: proto
                .output_results
                .into_iter()
                .map(|output| (output.output_arg_name, output.hash))
                .collect(),
        };
        Ok(ret)
    }
}

impl std::convert::From<UploadTaskResultRequest> for proto::UploadTaskResultRequest {
    fn from(req: UploadTaskResultRequest) -> Self {
        proto::UploadTaskResultRequest {
            success: req.success,
            staged_task_id: req.staged_task_id,
            worker_id: req.worker_id,
            output_results: req
                .output_results
                .into_iter()
                .map(|(output_arg_name, hash)| proto::OutputHash {
                    output_arg_name,
                    hash,
                })
                .collect(),
        }
    }
}

impl std::convert::TryFrom<proto::UploadTaskResultResponse> for UploadTaskResultResponse {
    type Error = Error;
    fn try_from(_proto: proto::UploadTaskResultResponse) -> Result<Self> {
        Ok(UploadTaskResultResponse)
    }
}

impl std::convert::From<UploadTaskResultResponse> for proto::UploadTaskResultResponse {
    fn from(_resp: UploadTaskResultResponse) -> Self {
        proto::UploadTaskResultResponse {}
    }
}
