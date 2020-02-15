use crate::teaclave_frontend_service_proto as proto;
use crate::teaclave_management_service::TeaclaveManagementRequest;
use crate::teaclave_management_service::TeaclaveManagementResponse;
use anyhow::anyhow;
use anyhow::{Error, Result};
use core::convert::TryInto;
use core::iter::FromIterator;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::prelude::v1::*;
use teaclave_rpc::into_request;
use teaclave_types::TeaclaveFileCryptoInfo;
use url::Url;

pub use proto::TeaclaveFrontend;
pub use proto::TeaclaveFrontendClient;
pub use proto::TeaclaveFrontendRequest;
pub use proto::TeaclaveFrontendResponse;

#[into_request(TeaclaveFrontendRequest::RegisterInputFile)]
#[into_request(TeaclaveManagementRequest::RegisterInputFile)]
#[derive(Debug, PartialEq)]
pub struct RegisterInputFileRequest {
    pub url: Url,
    pub hash: String,
    pub crypto_info: TeaclaveFileCryptoInfo,
}

#[into_request(TeaclaveFrontendResponse::RegisterInputFile)]
#[into_request(TeaclaveManagementResponse::RegisterInputFile)]
#[derive(Debug, PartialEq)]
pub struct RegisterInputFileResponse {
    pub data_id: String,
}

#[into_request(TeaclaveFrontendRequest::RegisterOutputFile)]
#[into_request(TeaclaveManagementRequest::RegisterOutputFile)]
#[derive(Debug)]
pub struct RegisterOutputFileRequest {
    pub url: Url,
    pub crypto_info: TeaclaveFileCryptoInfo,
}

#[into_request(TeaclaveFrontendResponse::RegisterOutputFile)]
#[into_request(TeaclaveManagementResponse::RegisterOutputFile)]
#[derive(Debug)]
pub struct RegisterOutputFileResponse {
    pub data_id: String,
}

#[into_request(TeaclaveManagementRequest::GetOutputFile)]
#[derive(Debug)]
pub struct GetOutputFileRequest {
    pub data_id: String,
}

#[into_request(TeaclaveManagementResponse::GetOutputFile)]
#[derive(Debug)]
pub struct GetOutputFileResponse {
    pub hash: String,
}

#[into_request(TeaclaveManagementRequest::GetFusionData)]
#[derive(Debug)]
pub struct GetFusionDataRequest {
    pub data_id: String,
}

#[into_request(TeaclaveManagementResponse::GetFusionData)]
#[derive(Debug)]
pub struct GetFusionDataResponse {
    pub hash: String,
    pub data_owner_id_list: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionInput {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionOutput {
    pub name: String,
    pub description: String,
}

#[into_request(TeaclaveManagementRequest::RegisterFunction)]
#[derive(Debug)]
pub struct RegisterFunctionRequest {
    pub name: String,
    pub description: String,
    pub payload: Vec<u8>,
    pub is_public: bool,
    pub arg_list: Vec<String>,
    pub input_list: Vec<FunctionInput>,
    pub output_list: Vec<FunctionOutput>,
}

#[into_request(TeaclaveManagementResponse::RegisterFunction)]
#[derive(Debug)]
pub struct RegisterFunctionResponse {
    pub function_id: String,
}

#[into_request(TeaclaveManagementRequest::GetFunction)]
#[derive(Debug)]
pub struct GetFunctionRequest {
    pub function_id: String,
}

#[into_request(TeaclaveManagementResponse::GetFunction)]
#[derive(Debug)]
pub struct GetFunctionResponse {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub payload: Vec<u8>,
    pub is_public: bool,
    pub arg_list: Vec<String>,
    pub input_list: Vec<FunctionInput>,
    pub output_list: Vec<FunctionOutput>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataOwnerList {
    pub user_id_list: HashSet<String>,
}

#[into_request(TeaclaveManagementRequest::CreateTask)]
#[derive(Debug)]
pub struct CreateTaskRequest {
    pub function_id: String,
    pub arg_list: HashMap<String, String>,
    pub input_data_owner_list: HashMap<String, DataOwnerList>,
    pub output_data_owner_list: HashMap<String, DataOwnerList>,
}

#[into_request(TeaclaveManagementResponse::CreateTask)]
#[derive(Debug)]
pub struct CreateTaskResponse {
    pub task_id: String,
}

#[derive(Debug)]
pub struct DataMap {
    pub data_name: String,
    pub data_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TaskStatus {
    Created,
    Ready,
    Approved,
    Running,
    Failed,
    Finished,
}

#[into_request(TeaclaveManagementRequest::GetTask)]
#[derive(Debug)]
pub struct GetTaskRequest {
    pub task_id: String,
}

#[into_request(TeaclaveManagementResponse::GetTask)]
#[derive(Debug)]
pub struct GetTaskResponse {
    pub task_id: String,
    pub creator: String,
    pub function_id: String,
    pub function_owner: String,
    pub arg_list: HashMap<String, String>,
    pub input_data_owner_list: HashMap<String, DataOwnerList>,
    pub output_data_owner_list: HashMap<String, DataOwnerList>,
    pub participants: HashSet<String>,
    pub approved_user_list: HashSet<String>,
    pub input_map: HashMap<String, String>,
    pub output_map: HashMap<String, String>,
    pub status: TaskStatus,
}

fn arg_list_from_proto(vector: Vec<proto::Argument>) -> Result<HashMap<String, String>> {
    let mut ret = HashMap::with_capacity(vector.len());
    for item in vector.into_iter() {
        ret.insert(item.arg_name, item.arg_value);
    }
    Ok(ret)
}

fn arg_list_to_proto(map: HashMap<String, String>) -> Vec<proto::Argument> {
    let mut ret = Vec::with_capacity(map.len());
    for (arg_name, arg_value) in map.into_iter() {
        let argument = proto::Argument {
            arg_name,
            arg_value,
        };
        ret.push(argument);
    }
    ret
}

fn data_map_to_proto(map: HashMap<String, String>) -> Vec<proto::DataMap> {
    let mut ret = Vec::with_capacity(map.len());
    for (data_name, data_id) in map.into_iter() {
        let data_map = proto::DataMap { data_name, data_id };
        ret.push(data_map);
    }
    ret
}

fn data_map_from_proto(vector: Vec<proto::DataMap>) -> Result<HashMap<String, String>> {
    let mut ret = HashMap::with_capacity(vector.len());
    for item in vector.into_iter() {
        ret.insert(item.data_name, item.data_id);
    }
    Ok(ret)
}

fn data_owner_list_from_proto(
    vector: Vec<proto::DataOwnerList>,
) -> Result<HashMap<String, DataOwnerList>> {
    let mut ret = HashMap::with_capacity(vector.len());
    for item in vector.into_iter() {
        let data_owner_list = DataOwnerList {
            user_id_list: HashSet::from_iter(item.user_id_list.into_iter()),
        };
        ret.insert(item.data_name, data_owner_list);
    }
    Ok(ret)
}

fn data_owner_list_to_proto(map: HashMap<String, DataOwnerList>) -> Vec<proto::DataOwnerList> {
    let mut ret = Vec::with_capacity(map.len());
    for (data_name, data_owner_list) in map.into_iter() {
        let data_owner_list = proto::DataOwnerList {
            data_name,
            user_id_list: data_owner_list.user_id_list.into_iter().collect(),
        };
        ret.push(data_owner_list);
    }
    ret
}
impl std::convert::TryFrom<proto::RegisterInputFileRequest> for RegisterInputFileRequest {
    type Error = Error;

    fn try_from(proto: proto::RegisterInputFileRequest) -> Result<Self> {
        let ret = Self {
            url: Url::parse(&proto.url)?,
            hash: proto.hash,
            crypto_info: proto
                .crypto_info
                .ok_or_else(|| anyhow!("missing crypto_info"))?
                .try_into()?,
        };

        Ok(ret)
    }
}

impl From<RegisterInputFileRequest> for proto::RegisterInputFileRequest {
    fn from(request: RegisterInputFileRequest) -> Self {
        Self {
            url: request.url.into_string(),
            hash: request.hash,
            crypto_info: Some(request.crypto_info.into()),
        }
    }
}

impl std::convert::TryFrom<proto::RegisterInputFileResponse> for RegisterInputFileResponse {
    type Error = Error;

    fn try_from(proto: proto::RegisterInputFileResponse) -> Result<Self> {
        Ok(Self {
            data_id: proto.data_id,
        })
    }
}

impl From<RegisterInputFileResponse> for proto::RegisterInputFileResponse {
    fn from(request: RegisterInputFileResponse) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::RegisterOutputFileRequest> for RegisterOutputFileRequest {
    type Error = Error;

    fn try_from(proto: proto::RegisterOutputFileRequest) -> Result<Self> {
        let ret = Self {
            url: Url::parse(&proto.url)?,
            crypto_info: proto
                .crypto_info
                .ok_or_else(|| anyhow!("missing crypto_info"))?
                .try_into()?,
        };

        Ok(ret)
    }
}

impl From<RegisterOutputFileRequest> for proto::RegisterOutputFileRequest {
    fn from(request: RegisterOutputFileRequest) -> Self {
        Self {
            url: request.url.into_string(),
            crypto_info: Some(request.crypto_info.into()),
        }
    }
}

impl std::convert::TryFrom<proto::RegisterOutputFileResponse> for RegisterOutputFileResponse {
    type Error = Error;

    fn try_from(proto: proto::RegisterOutputFileResponse) -> Result<Self> {
        Ok(Self {
            data_id: proto.data_id,
        })
    }
}

impl From<RegisterOutputFileResponse> for proto::RegisterOutputFileResponse {
    fn from(request: RegisterOutputFileResponse) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetOutputFileRequest> for GetOutputFileRequest {
    type Error = Error;

    fn try_from(proto: proto::GetOutputFileRequest) -> Result<Self> {
        let ret = Self {
            data_id: proto.data_id,
        };

        Ok(ret)
    }
}

impl From<GetOutputFileRequest> for proto::GetOutputFileRequest {
    fn from(request: GetOutputFileRequest) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetOutputFileResponse> for GetOutputFileResponse {
    type Error = Error;

    fn try_from(proto: proto::GetOutputFileResponse) -> Result<Self> {
        Ok(Self { hash: proto.hash })
    }
}

impl From<GetOutputFileResponse> for proto::GetOutputFileResponse {
    fn from(response: GetOutputFileResponse) -> Self {
        Self {
            hash: response.hash,
        }
    }
}

impl std::convert::TryFrom<proto::GetFusionDataRequest> for GetFusionDataRequest {
    type Error = Error;

    fn try_from(proto: proto::GetFusionDataRequest) -> Result<Self> {
        let ret = Self {
            data_id: proto.data_id,
        };

        Ok(ret)
    }
}

impl From<GetFusionDataRequest> for proto::GetFusionDataRequest {
    fn from(request: GetFusionDataRequest) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetFusionDataResponse> for GetFusionDataResponse {
    type Error = Error;

    fn try_from(proto: proto::GetFusionDataResponse) -> Result<Self> {
        Ok(Self {
            hash: proto.hash,
            data_owner_id_list: proto.data_owner_id_list,
        })
    }
}

impl From<GetFusionDataResponse> for proto::GetFusionDataResponse {
    fn from(response: GetFusionDataResponse) -> Self {
        Self {
            hash: response.hash,
            data_owner_id_list: response.data_owner_id_list,
        }
    }
}

impl std::convert::TryFrom<proto::FunctionInput> for FunctionInput {
    type Error = Error;

    fn try_from(proto: proto::FunctionInput) -> Result<Self> {
        let ret = Self {
            name: proto.name,
            description: proto.description,
        };

        Ok(ret)
    }
}

impl From<FunctionInput> for proto::FunctionInput {
    fn from(input: FunctionInput) -> Self {
        Self {
            name: input.name,
            description: input.description,
        }
    }
}

impl std::convert::TryFrom<proto::FunctionOutput> for FunctionOutput {
    type Error = Error;

    fn try_from(proto: proto::FunctionOutput) -> Result<Self> {
        let ret = Self {
            name: proto.name,
            description: proto.description,
        };

        Ok(ret)
    }
}

impl From<FunctionOutput> for proto::FunctionOutput {
    fn from(output: FunctionOutput) -> Self {
        Self {
            name: output.name,
            description: output.description,
        }
    }
}

impl std::convert::TryFrom<proto::RegisterFunctionRequest> for RegisterFunctionRequest {
    type Error = Error;

    fn try_from(proto: proto::RegisterFunctionRequest) -> Result<Self> {
        let input_list: Result<Vec<FunctionInput>> = proto
            .input_list
            .into_iter()
            .map(FunctionInput::try_from)
            .collect();
        let output_list: Result<Vec<FunctionOutput>> = proto
            .output_list
            .into_iter()
            .map(FunctionOutput::try_from)
            .collect();

        let ret = Self {
            name: proto.name,
            description: proto.description,
            payload: proto.payload,
            is_public: proto.is_public,
            arg_list: proto.arg_list,
            input_list: input_list?,
            output_list: output_list?,
        };
        Ok(ret)
    }
}

impl From<RegisterFunctionRequest> for proto::RegisterFunctionRequest {
    fn from(request: RegisterFunctionRequest) -> Self {
        let input_list: Vec<proto::FunctionInput> = request
            .input_list
            .into_iter()
            .map(proto::FunctionInput::from)
            .collect();
        let output_list: Vec<proto::FunctionOutput> = request
            .output_list
            .into_iter()
            .map(proto::FunctionOutput::from)
            .collect();

        Self {
            name: request.name,
            description: request.description,
            payload: request.payload,
            is_public: request.is_public,
            arg_list: request.arg_list,
            input_list,
            output_list,
        }
    }
}

impl std::convert::TryFrom<proto::RegisterFunctionResponse> for RegisterFunctionResponse {
    type Error = Error;

    fn try_from(proto: proto::RegisterFunctionResponse) -> Result<Self> {
        let ret = Self {
            function_id: proto.function_id,
        };

        Ok(ret)
    }
}

impl From<RegisterFunctionResponse> for proto::RegisterFunctionResponse {
    fn from(response: RegisterFunctionResponse) -> Self {
        Self {
            function_id: response.function_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetFunctionRequest> for GetFunctionRequest {
    type Error = Error;

    fn try_from(proto: proto::GetFunctionRequest) -> Result<Self> {
        let ret = Self {
            function_id: proto.function_id,
        };

        Ok(ret)
    }
}

impl From<GetFunctionRequest> for proto::GetFunctionRequest {
    fn from(request: GetFunctionRequest) -> Self {
        Self {
            function_id: request.function_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetFunctionResponse> for GetFunctionResponse {
    type Error = Error;

    fn try_from(proto: proto::GetFunctionResponse) -> Result<Self> {
        let input_list: Result<Vec<FunctionInput>> = proto
            .input_list
            .into_iter()
            .map(FunctionInput::try_from)
            .collect();
        let output_list: Result<Vec<FunctionOutput>> = proto
            .output_list
            .into_iter()
            .map(FunctionOutput::try_from)
            .collect();

        let ret = Self {
            name: proto.name,
            description: proto.description,
            owner: proto.owner,
            payload: proto.payload,
            is_public: proto.is_public,
            arg_list: proto.arg_list,
            input_list: input_list?,
            output_list: output_list?,
        };

        Ok(ret)
    }
}

impl From<GetFunctionResponse> for proto::GetFunctionResponse {
    fn from(response: GetFunctionResponse) -> Self {
        let input_list: Vec<proto::FunctionInput> = response
            .input_list
            .into_iter()
            .map(proto::FunctionInput::from)
            .collect();
        let output_list: Vec<proto::FunctionOutput> = response
            .output_list
            .into_iter()
            .map(proto::FunctionOutput::from)
            .collect();

        Self {
            name: response.name,
            description: response.description,
            owner: response.owner,
            payload: response.payload,
            is_public: response.is_public,
            arg_list: response.arg_list,
            input_list,
            output_list,
        }
    }
}

impl std::convert::TryFrom<proto::CreateTaskRequest> for CreateTaskRequest {
    type Error = Error;

    fn try_from(proto: proto::CreateTaskRequest) -> Result<Self> {
        let arg_list = arg_list_from_proto(proto.arg_list)?;
        let input_data_owner_list = data_owner_list_from_proto(proto.input_data_owner_list)?;
        let output_data_owner_list = data_owner_list_from_proto(proto.output_data_owner_list)?;
        let ret = Self {
            function_id: proto.function_id,
            arg_list,
            input_data_owner_list,
            output_data_owner_list,
        };
        Ok(ret)
    }
}

impl From<CreateTaskRequest> for proto::CreateTaskRequest {
    fn from(request: CreateTaskRequest) -> Self {
        let arg_list = arg_list_to_proto(request.arg_list);
        let input_data_owner_list = data_owner_list_to_proto(request.input_data_owner_list);
        let output_data_owner_list = data_owner_list_to_proto(request.output_data_owner_list);

        Self {
            function_id: request.function_id,
            arg_list,
            input_data_owner_list,
            output_data_owner_list,
        }
    }
}

impl std::convert::TryFrom<proto::CreateTaskResponse> for CreateTaskResponse {
    type Error = Error;

    fn try_from(proto: proto::CreateTaskResponse) -> Result<Self> {
        let ret = Self {
            task_id: proto.task_id,
        };

        Ok(ret)
    }
}

impl From<CreateTaskResponse> for proto::CreateTaskResponse {
    fn from(response: CreateTaskResponse) -> Self {
        Self {
            task_id: response.task_id,
        }
    }
}

impl std::convert::TryFrom<i32> for TaskStatus {
    type Error = Error;

    fn try_from(status: i32) -> Result<Self> {
        let ret = match proto::TaskStatus::from_i32(status) {
            Some(proto::TaskStatus::Created) => TaskStatus::Created,
            Some(proto::TaskStatus::Ready) => TaskStatus::Ready,
            Some(proto::TaskStatus::Approved) => TaskStatus::Approved,
            Some(proto::TaskStatus::Running) => TaskStatus::Running,
            Some(proto::TaskStatus::Failed) => TaskStatus::Failed,
            Some(proto::TaskStatus::Finished) => TaskStatus::Finished,
            None => return Err(anyhow!("invalid task status")),
        };
        Ok(ret)
    }
}

impl From<TaskStatus> for i32 {
    fn from(status: TaskStatus) -> i32 {
        match status {
            TaskStatus::Created => proto::TaskStatus::Created as i32,
            TaskStatus::Ready => proto::TaskStatus::Ready as i32,
            TaskStatus::Approved => proto::TaskStatus::Approved as i32,
            TaskStatus::Running => proto::TaskStatus::Running as i32,
            TaskStatus::Failed => proto::TaskStatus::Failed as i32,
            TaskStatus::Finished => proto::TaskStatus::Finished as i32,
        }
    }
}

impl std::convert::TryFrom<proto::GetTaskRequest> for GetTaskRequest {
    type Error = Error;

    fn try_from(proto: proto::GetTaskRequest) -> Result<Self> {
        let ret = Self {
            task_id: proto.task_id,
        };

        Ok(ret)
    }
}

impl From<GetTaskRequest> for proto::GetTaskRequest {
    fn from(request: GetTaskRequest) -> Self {
        Self {
            task_id: request.task_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetTaskResponse> for GetTaskResponse {
    type Error = Error;

    fn try_from(proto: proto::GetTaskResponse) -> Result<Self> {
        let arg_list = arg_list_from_proto(proto.arg_list)?;
        let input_data_owner_list = data_owner_list_from_proto(proto.input_data_owner_list)?;
        let output_data_owner_list = data_owner_list_from_proto(proto.output_data_owner_list)?;
        let input_map = data_map_from_proto(proto.input_map)?;
        let output_map = data_map_from_proto(proto.output_map)?;
        let status = TaskStatus::try_from(proto.status)?;

        let ret = Self {
            task_id: proto.task_id,
            creator: proto.creator,
            function_id: proto.function_id,
            function_owner: proto.function_owner,
            arg_list,
            input_data_owner_list,
            output_data_owner_list,
            participants: proto.participants.into_iter().collect(),
            approved_user_list: proto.approved_user_list.into_iter().collect(),
            input_map,
            output_map,
            status,
        };

        Ok(ret)
    }
}

impl From<GetTaskResponse> for proto::GetTaskResponse {
    fn from(response: GetTaskResponse) -> Self {
        let arg_list = arg_list_to_proto(response.arg_list);
        let input_data_owner_list = data_owner_list_to_proto(response.input_data_owner_list);
        let output_data_owner_list = data_owner_list_to_proto(response.output_data_owner_list);
        let input_map = data_map_to_proto(response.input_map);
        let output_map = data_map_to_proto(response.output_map);
        let status = i32::from(response.status);
        Self {
            task_id: response.task_id,
            creator: response.creator,
            function_id: response.function_id,
            function_owner: response.function_owner,
            arg_list,
            input_data_owner_list,
            output_data_owner_list,
            participants: response.participants.into_iter().collect(),
            approved_user_list: response.approved_user_list.into_iter().collect(),
            input_map,
            output_map,
            status,
        }
    }
}
