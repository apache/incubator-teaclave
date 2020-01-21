#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use crate::teaclave_common::TeaclaveFileCryptoInfo;
use anyhow::{anyhow, Error, Result};
use core::convert::TryFrom;
use std::collections::HashMap;
use std::{fmt, format};

use crate::teaclave_common_proto;
pub mod proto {
    #![allow(clippy::all)]
    include!(concat!(
        env!("OUT_DIR"),
        "/teaclave_execution_service_proto.rs"
    ));
}

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

impl std::convert::TryFrom<&str> for TeaclaveExecutorSelector {
    type Error = Error;

    fn try_from(selector: &str) -> Result<Self> {
        let sel = match selector {
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

#[derive(Clone, Debug, serde_derive::Serialize, serde_derive::Deserialize)]
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

impl std::convert::TryFrom<&proto::WorkerFileInfo> for TeaclaveWorkerFileInfo {
    type Error = Error;
    fn try_from(proto: &proto::WorkerFileInfo) -> Result<Self> {
        let path = std::path::Path::new(&proto.path).to_path_buf();
        let info = proto
            .crypto_info
            .as_ref()
            .ok_or_else(|| anyhow!("Missing field: crypto_info"))?;
        let crypto_info = TeaclaveFileCryptoInfo::try_from(info)?;
        Ok(TeaclaveWorkerFileInfo { path, crypto_info })
    }
}

fn try_convert_files_info(
    files_info: &HashMap<String, proto::WorkerFileInfo>,
) -> Result<HashMap<String, TeaclaveWorkerFileInfo>> {
    let mut out_info: HashMap<String, TeaclaveWorkerFileInfo> = HashMap::new();
    files_info.iter().try_for_each(
        |(fid, finfo): (&String, &proto::WorkerFileInfo)| -> Result<()> {
            let worker_file_info = TeaclaveWorkerFileInfo::try_from(finfo)?;
            out_info.insert(fid.to_string(), worker_file_info);
            Ok(())
        },
    )?;
    Ok(out_info)
}

// For server side
impl std::convert::TryFrom<proto::StagedFunctionExecuteRequest> for StagedFunctionExecuteRequest {
    type Error = Error;

    fn try_from(proto: proto::StagedFunctionExecuteRequest) -> Result<Self> {
        let executor_type = TeaclaveExecutorSelector::try_from(proto.executor_type.as_ref())?;
        let input_files = try_convert_files_info(&proto.input_files)?;
        let output_files = try_convert_files_info(&proto.output_files)?;

        let ret = Self {
            runtime_name: proto.runtime_name,
            executor_type,
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
    files_info: &HashMap<String, TeaclaveWorkerFileInfo>,
) -> HashMap<String, proto::WorkerFileInfo> {
    let mut out_info: HashMap<String, proto::WorkerFileInfo> = HashMap::new();
    files_info
        .iter()
        .for_each(|(fid, finfo): (&String, &TeaclaveWorkerFileInfo)| {
            out_info.insert(fid.to_string(), proto::WorkerFileInfo::from(finfo));
        });
    out_info
}

// For client side
impl std::convert::From<&TeaclaveWorkerFileInfo> for proto::WorkerFileInfo {
    fn from(info: &TeaclaveWorkerFileInfo) -> Self {
        let path = info.path.clone().into_os_string().into_string().unwrap();
        let crypto_info = teaclave_common_proto::FileCryptoInfo::from(&info.crypto_info);
        proto::WorkerFileInfo {
            path,
            crypto_info: Some(crypto_info),
        }
    }
}

impl From<StagedFunctionExecuteRequest> for proto::StagedFunctionExecuteRequest {
    fn from(request: StagedFunctionExecuteRequest) -> Self {
        let input_files = convert_files_info(&request.input_files);
        let output_files = convert_files_info(&request.output_files);
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
