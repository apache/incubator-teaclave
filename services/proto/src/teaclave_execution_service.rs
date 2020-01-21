#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::teaclave_common::TeaclaveFileCryptoInfo;
use anyhow::{anyhow, Error, Result};
use core::convert::TryInto;
use std::collections::HashMap;
use std::{fmt, format};

use crate::teaclave_execution_service_proto as proto;
pub use proto::TeaclaveExecution;
pub use proto::TeaclaveExecutionClient;
pub use proto::TeaclaveExecutionRequest;
pub use proto::TeaclaveExecutionResponse;

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
#[serde(rename_all(deserialize = "snake_case"))]
pub enum TeaclaveExecutorSelector {
    Native,
    Python,
}

impl std::convert::TryFrom<String> for TeaclaveExecutorSelector {
    type Error = Error;

    fn try_from(selector: String) -> Result<Self> {
        let sel = match selector.as_ref() {
            "python" => TeaclaveExecutorSelector::Python,
            "native" => TeaclaveExecutorSelector::Native,
            _ => return Err(anyhow!(format!("Invalid executor selector: {}", selector))),
        };
        Ok(sel)
    }
}

impl fmt::Display for TeaclaveExecutorSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TeaclaveExecutorSelector::Native => write!(f, "native"),
            TeaclaveExecutorSelector::Python => write!(f, "python"),
        }
    }
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct TeaclaveWorkerFileInfo {
    pub path: std::path::PathBuf,
    pub crypto_info: TeaclaveFileCryptoInfo,
}

#[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
pub struct StagedFunctionExecuteRequest {
    pub runtime_name: String,
    pub executor_type: TeaclaveExecutorSelector, // "native" | "python"
    pub function_name: String,                   // "gbdt_training" | "mesapy" |
    pub function_payload: String,
    pub function_args: HashMap<String, String>,
    pub input_files: HashMap<String, TeaclaveWorkerFileInfo>,
    pub output_files: HashMap<String, TeaclaveWorkerFileInfo>,
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

fn try_convert_files_info(
    files_info: HashMap<String, proto::WorkerFileInfo>,
) -> Result<HashMap<String, TeaclaveWorkerFileInfo>> {
    let mut out_info: HashMap<String, TeaclaveWorkerFileInfo> = HashMap::new();
    files_info.into_iter().try_for_each(
        |(fid, finfo): (String, proto::WorkerFileInfo)| -> Result<()> {
            out_info.insert(fid, finfo.try_into()?);
            Ok(())
        },
    )?;
    Ok(out_info)
}

// For server side
impl std::convert::TryFrom<proto::StagedFunctionExecuteRequest> for StagedFunctionExecuteRequest {
    type Error = Error;

    fn try_from(proto: proto::StagedFunctionExecuteRequest) -> Result<Self> {
        let input_files = try_convert_files_info(proto.input_files)?;
        let output_files = try_convert_files_info(proto.output_files)?;

        let ret = Self {
            runtime_name: proto.runtime_name,
            executor_type: proto.executor_type.try_into()?,
            function_name: proto.function_name,
            function_payload: proto.function_payload,
            function_args: proto.function_args,
            input_files,
            output_files,
        };

        Ok(ret)
    }
}

fn convert_files_info(
    files_info: HashMap<String, TeaclaveWorkerFileInfo>,
) -> HashMap<String, proto::WorkerFileInfo> {
    let mut out_info: HashMap<String, proto::WorkerFileInfo> = HashMap::new();
    files_info
        .into_iter()
        .for_each(|(fid, finfo): (String, TeaclaveWorkerFileInfo)| {
            out_info.insert(fid, finfo.into());
        });
    out_info
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

impl From<StagedFunctionExecuteRequest> for proto::StagedFunctionExecuteRequest {
    fn from(request: StagedFunctionExecuteRequest) -> Self {
        let input_files = convert_files_info(request.input_files);
        let output_files = convert_files_info(request.output_files);
        Self {
            runtime_name: request.runtime_name,
            executor_type: request.executor_type.to_string(),
            function_name: request.function_name,
            function_payload: request.function_payload,
            function_args: request.function_args,
            input_files,
            output_files,
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
