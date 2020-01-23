#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use anyhow::{anyhow, Error, Result};
use core::convert::TryInto;
use teaclave_types::{TeaclaveFunctionArguments, TeaclaveWorkerFileInfo, WorkerInvocation};

use crate::teaclave_execution_service_proto as proto;
pub use proto::TeaclaveExecution;
pub use proto::TeaclaveExecutionClient;
pub use proto::TeaclaveExecutionRequest;
pub use proto::TeaclaveExecutionResponse;

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct StagedFunctionExecuteRequest {
    #[serde(flatten)]
    invocation: WorkerInvocation,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct StagedFunctionExecuteResponse {
    pub summary: std::string::String,
}

impl std::convert::TryFrom<proto::WorkerFileInfo> for TeaclaveWorkerFileInfo {
    type Error = Error;
    fn try_from(proto: proto::WorkerFileInfo) -> Result<Self> {
        let path = std::path::Path::new(&proto.path).to_path_buf();
        let crypto_info = proto
            .crypto_info
            .ok_or_else(|| anyhow!("Missing field: crypto_info"))?
            .try_into()?;
        Ok(TeaclaveWorkerFileInfo { path, crypto_info })
    }
}

// For server side
impl std::convert::TryFrom<proto::StagedFunctionExecuteRequest> for StagedFunctionExecuteRequest {
    type Error = Error;

    fn try_from(proto: proto::StagedFunctionExecuteRequest) -> Result<Self> {
        let ret = Self {
            invocation: WorkerInvocation {
                runtime_name: proto.runtime_name,
                executor_type: proto.executor_type.try_into()?,
                function_name: proto.function_name,
                function_payload: proto.function_payload,
                function_args: TeaclaveFunctionArguments {
                    args: proto.function_args,
                },
                input_files: proto.input_files.try_into()?,
                output_files: proto.output_files.try_into()?,
            },
        };

        Ok(ret)
    }
}

// For client side
impl std::convert::From<TeaclaveWorkerFileInfo> for proto::WorkerFileInfo {
    fn from(info: TeaclaveWorkerFileInfo) -> Self {
        proto::WorkerFileInfo {
            path: info.path.to_string_lossy().to_string(),
            crypto_info: Some(info.crypto_info.into()),
        }
    }
}

impl From<StagedFunctionExecuteRequest> for WorkerInvocation {
    fn from(request: StagedFunctionExecuteRequest) -> Self {
        request.invocation
    }
}

impl From<StagedFunctionExecuteRequest> for proto::StagedFunctionExecuteRequest {
    fn from(request: StagedFunctionExecuteRequest) -> Self {
        Self {
            runtime_name: request.invocation.runtime_name,
            executor_type: request.invocation.executor_type.to_string(),
            function_name: request.invocation.function_name,
            function_payload: request.invocation.function_payload,
            function_args: request.invocation.function_args.args,
            input_files: request.invocation.input_files.into(),
            output_files: request.invocation.output_files.into(),
        }
    }
}

impl From<StagedFunctionExecuteResponse> for proto::StagedFunctionExecuteResponse {
    fn from(response: StagedFunctionExecuteResponse) -> Self {
        Self {
            summary: response.summary,
        }
    }
}

impl From<String> for StagedFunctionExecuteResponse {
    fn from(summary: String) -> Self {
        Self { summary }
    }
}
