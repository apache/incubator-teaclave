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

use crate::teaclave_common::{i32_from_task_status, i32_to_task_status};
use crate::teaclave_frontend_service_proto as proto;
use crate::teaclave_management_service::TeaclaveManagementRequest;
use crate::teaclave_management_service::TeaclaveManagementResponse;
use anyhow::anyhow;
use anyhow::{Error, Result};
use core::convert::TryInto;
use core::iter::FromIterator;
use std::collections::{HashMap, HashSet};
use std::prelude::v1::*;
use teaclave_rpc::into_request;
use teaclave_types::{
    DataOwnerList, FileCrypto, FunctionArguments, FunctionInput, FunctionOutput, TaskStatus,
};
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
    pub crypto_info: FileCrypto,
}

impl RegisterInputFileRequest {
    pub fn new(url: Url, hash: impl Into<String>, crypto_info: FileCrypto) -> Self {
        Self {
            url,
            hash: hash.into(),
            crypto_info,
        }
    }
}

#[into_request(TeaclaveFrontendResponse::RegisterInputFile)]
#[into_request(TeaclaveManagementResponse::RegisterInputFile)]
#[derive(Debug, PartialEq)]
pub struct RegisterInputFileResponse {
    pub data_id: String,
}

impl RegisterInputFileResponse {
    pub fn new(data_id: impl Into<String>) -> Self {
        Self {
            data_id: data_id.into(),
        }
    }
}

#[into_request(TeaclaveFrontendRequest::RegisterOutputFile)]
#[into_request(TeaclaveManagementRequest::RegisterOutputFile)]
#[derive(Debug)]
pub struct RegisterOutputFileRequest {
    pub url: Url,
    pub crypto_info: FileCrypto,
}

impl RegisterOutputFileRequest {
    pub fn new(url: Url, crypto_info: FileCrypto) -> Self {
        Self { url, crypto_info }
    }
}

#[into_request(TeaclaveFrontendResponse::RegisterOutputFile)]
#[into_request(TeaclaveManagementResponse::RegisterOutputFile)]
#[derive(Debug)]
pub struct RegisterOutputFileResponse {
    pub data_id: String,
}

impl RegisterOutputFileResponse {
    pub fn new(data_id: impl Into<String>) -> Self {
        Self {
            data_id: data_id.into(),
        }
    }
}

#[into_request(TeaclaveFrontendRequest::RegisterFusionOutput)]
#[into_request(TeaclaveManagementRequest::RegisterFusionOutput)]
#[derive(Debug)]
pub struct RegisterFusionOutputRequest {
    pub owner_list: HashSet<String>,
}

impl RegisterFusionOutputRequest {
    pub fn new(owner_list: HashSet<String>) -> Self {
        Self { owner_list }
    }
}

#[into_request(TeaclaveFrontendResponse::RegisterFusionOutput)]
#[into_request(TeaclaveManagementResponse::RegisterFusionOutput)]
#[derive(Debug)]
pub struct RegisterFusionOutputResponse {
    pub data_id: String,
}

impl RegisterFusionOutputResponse {
    pub fn new(data_id: impl Into<String>) -> Self {
        Self {
            data_id: data_id.into(),
        }
    }
}

#[into_request(TeaclaveFrontendRequest::RegisterInputFromOutput)]
#[into_request(TeaclaveManagementRequest::RegisterInputFromOutput)]
#[derive(Debug)]
pub struct RegisterInputFromOutputRequest {
    pub data_id: String,
}

impl RegisterInputFromOutputRequest {
    pub fn new(data_id: impl Into<String>) -> Self {
        Self {
            data_id: data_id.into(),
        }
    }
}

#[into_request(TeaclaveFrontendResponse::RegisterInputFromOutput)]
#[into_request(TeaclaveManagementResponse::RegisterInputFromOutput)]
#[derive(Debug)]
pub struct RegisterInputFromOutputResponse {
    pub data_id: String,
}

impl RegisterInputFromOutputResponse {
    pub fn new(data_id: impl Into<String>) -> Self {
        Self {
            data_id: data_id.into(),
        }
    }
}

#[into_request(TeaclaveFrontendRequest::GetInputFile)]
#[into_request(TeaclaveManagementRequest::GetInputFile)]
#[derive(Debug)]
pub struct GetInputFileRequest {
    pub data_id: String,
}

impl GetInputFileRequest {
    pub fn new(data_id: impl Into<String>) -> Self {
        Self {
            data_id: data_id.into(),
        }
    }
}

#[into_request(TeaclaveFrontendResponse::GetInputFile)]
#[into_request(TeaclaveManagementResponse::GetInputFile)]
#[derive(Debug)]
pub struct GetInputFileResponse {
    pub owner: HashSet<String>,
    pub hash: String,
}

impl GetInputFileResponse {
    pub fn new(owner: HashSet<String>, hash: impl Into<String>) -> Self {
        Self {
            owner,
            hash: hash.into(),
        }
    }
}

#[into_request(TeaclaveFrontendRequest::GetOutputFile)]
#[into_request(TeaclaveManagementRequest::GetOutputFile)]
#[derive(Debug)]
pub struct GetOutputFileRequest {
    pub data_id: String,
}

impl GetOutputFileRequest {
    pub fn new(data_id: impl Into<String>) -> Self {
        Self {
            data_id: data_id.into(),
        }
    }
}

#[into_request(TeaclaveFrontendResponse::GetOutputFile)]
#[into_request(TeaclaveManagementResponse::GetOutputFile)]
#[derive(Debug)]
pub struct GetOutputFileResponse {
    pub owner: HashSet<String>,
    pub hash: String,
}

impl GetOutputFileResponse {
    pub fn new(owner: HashSet<String>, hash: impl Into<String>) -> Self {
        Self {
            owner,
            hash: hash.into(),
        }
    }
}

#[into_request(TeaclaveManagementRequest::RegisterFunction)]
#[into_request(TeaclaveFrontendRequest::RegisterFunction)]
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

impl RegisterFunctionResponse {
    pub fn new(function_id: impl Into<String>) -> Self {
        Self {
            function_id: function_id.into(),
        }
    }
}

#[into_request(TeaclaveManagementRequest::GetFunction)]
#[into_request(TeaclaveFrontendRequest::GetFunction)]
#[derive(Debug)]
pub struct GetFunctionRequest {
    pub function_id: String,
}

impl GetFunctionRequest {
    pub fn new(function_id: impl Into<String>) -> Self {
        Self {
            function_id: function_id.into(),
        }
    }
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

#[into_request(TeaclaveManagementRequest::CreateTask)]
#[into_request(TeaclaveFrontendRequest::CreateTask)]
#[derive(Debug)]
pub struct CreateTaskRequest {
    pub function_id: String,
    pub function_arguments: FunctionArguments,
    pub input_data_owner_list: HashMap<String, DataOwnerList>,
    pub output_data_owner_list: HashMap<String, DataOwnerList>,
}

#[into_request(TeaclaveManagementResponse::CreateTask)]
#[derive(Debug)]
pub struct CreateTaskResponse {
    pub task_id: String,
}

impl CreateTaskResponse {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
        }
    }
}

#[into_request(TeaclaveManagementRequest::GetTask)]
#[into_request(TeaclaveFrontendRequest::GetTask)]
#[derive(Debug)]
pub struct GetTaskRequest {
    pub task_id: String,
}

impl GetTaskRequest {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
        }
    }
}

#[into_request(TeaclaveManagementResponse::GetTask)]
#[derive(Debug)]
pub struct GetTaskResponse {
    pub task_id: String,
    pub creator: String,
    pub function_id: String,
    pub function_owner: String,
    pub function_arguments: FunctionArguments,
    pub input_data_owner_list: HashMap<String, DataOwnerList>,
    pub output_data_owner_list: HashMap<String, DataOwnerList>,
    pub participants: HashSet<String>,
    pub approved_user_list: HashSet<String>,
    pub input_map: HashMap<String, String>,
    pub output_map: HashMap<String, String>,
    pub return_value: Vec<u8>,
    pub output_file_hash: HashMap<String, String>,
    pub status: TaskStatus,
}

#[into_request(TeaclaveManagementRequest::AssignData)]
#[into_request(TeaclaveFrontendRequest::AssignData)]
#[derive(Debug)]
pub struct AssignDataRequest {
    pub task_id: String,
    pub input_map: HashMap<String, String>,
    pub output_map: HashMap<String, String>,
}

impl AssignDataRequest {
    pub fn new(
        task_id: impl Into<String>,
        input_map: HashMap<String, String>,
        output_map: HashMap<String, String>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            input_map,
            output_map,
        }
    }
}

#[derive(Debug)]
pub struct AssignDataResponse;

#[into_request(TeaclaveManagementRequest::ApproveTask)]
#[into_request(TeaclaveFrontendRequest::ApproveTask)]
#[derive(Debug)]
pub struct ApproveTaskRequest {
    pub task_id: String,
}

impl ApproveTaskRequest {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
        }
    }
}

#[derive(Debug)]
pub struct ApproveTaskResponse;

#[into_request(TeaclaveManagementRequest::InvokeTask)]
#[into_request(TeaclaveFrontendRequest::InvokeTask)]
#[derive(Debug)]
pub struct InvokeTaskRequest {
    pub task_id: String,
}

impl InvokeTaskRequest {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
        }
    }
}

#[derive(Debug)]
pub struct InvokeTaskResponse;

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

impl std::convert::TryFrom<proto::RegisterFusionOutputRequest> for RegisterFusionOutputRequest {
    type Error = Error;

    fn try_from(proto: proto::RegisterFusionOutputRequest) -> Result<Self> {
        let ret = Self {
            owner_list: proto.owner_list.into_iter().collect(),
        };

        Ok(ret)
    }
}

impl From<RegisterFusionOutputRequest> for proto::RegisterFusionOutputRequest {
    fn from(request: RegisterFusionOutputRequest) -> Self {
        Self {
            owner_list: request.owner_list.into_iter().collect(),
        }
    }
}

impl std::convert::TryFrom<proto::RegisterFusionOutputResponse> for RegisterFusionOutputResponse {
    type Error = Error;

    fn try_from(proto: proto::RegisterFusionOutputResponse) -> Result<Self> {
        Ok(Self {
            data_id: proto.data_id,
        })
    }
}

impl From<RegisterFusionOutputResponse> for proto::RegisterFusionOutputResponse {
    fn from(request: RegisterFusionOutputResponse) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::RegisterInputFromOutputRequest>
    for RegisterInputFromOutputRequest
{
    type Error = Error;

    fn try_from(proto: proto::RegisterInputFromOutputRequest) -> Result<Self> {
        let ret = Self {
            data_id: proto.data_id,
        };

        Ok(ret)
    }
}

impl From<RegisterInputFromOutputRequest> for proto::RegisterInputFromOutputRequest {
    fn from(request: RegisterInputFromOutputRequest) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::RegisterInputFromOutputResponse>
    for RegisterInputFromOutputResponse
{
    type Error = Error;

    fn try_from(proto: proto::RegisterInputFromOutputResponse) -> Result<Self> {
        Ok(Self {
            data_id: proto.data_id,
        })
    }
}

impl From<RegisterInputFromOutputResponse> for proto::RegisterInputFromOutputResponse {
    fn from(request: RegisterInputFromOutputResponse) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetInputFileRequest> for GetInputFileRequest {
    type Error = Error;

    fn try_from(proto: proto::GetInputFileRequest) -> Result<Self> {
        let ret = Self {
            data_id: proto.data_id,
        };

        Ok(ret)
    }
}

impl From<GetInputFileRequest> for proto::GetInputFileRequest {
    fn from(request: GetInputFileRequest) -> Self {
        Self {
            data_id: request.data_id,
        }
    }
}

impl std::convert::TryFrom<proto::GetInputFileResponse> for GetInputFileResponse {
    type Error = Error;

    fn try_from(proto: proto::GetInputFileResponse) -> Result<Self> {
        Ok(Self {
            owner: proto.owner.into_iter().collect(),
            hash: proto.hash,
        })
    }
}

impl From<GetInputFileResponse> for proto::GetInputFileResponse {
    fn from(request: GetInputFileResponse) -> Self {
        Self {
            owner: request.owner.into_iter().collect(),
            hash: request.hash,
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
        Ok(Self {
            owner: proto.owner.into_iter().collect(),
            hash: proto.hash,
        })
    }
}

impl From<GetOutputFileResponse> for proto::GetOutputFileResponse {
    fn from(request: GetOutputFileResponse) -> Self {
        Self {
            owner: request.owner.into_iter().collect(),
            hash: request.hash,
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

pub fn data_owner_list_from_proto(
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

pub fn data_owner_list_to_proto<S: std::hash::BuildHasher>(
    map: HashMap<String, DataOwnerList, S>,
) -> Vec<proto::DataOwnerList> {
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

impl std::convert::TryFrom<proto::CreateTaskRequest> for CreateTaskRequest {
    type Error = Error;

    fn try_from(proto: proto::CreateTaskRequest) -> Result<Self> {
        let function_arguments = proto.function_arguments.into();
        let input_data_owner_list = data_owner_list_from_proto(proto.input_data_owner_list)?;
        let output_data_owner_list = data_owner_list_from_proto(proto.output_data_owner_list)?;
        let ret = Self {
            function_id: proto.function_id,
            function_arguments,
            input_data_owner_list,
            output_data_owner_list,
        };
        Ok(ret)
    }
}

impl From<CreateTaskRequest> for proto::CreateTaskRequest {
    fn from(request: CreateTaskRequest) -> Self {
        let function_arguments = request.function_arguments.into();
        let input_data_owner_list = data_owner_list_to_proto(request.input_data_owner_list);
        let output_data_owner_list = data_owner_list_to_proto(request.output_data_owner_list);

        Self {
            function_id: request.function_id,
            function_arguments,
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
        let function_arguments = proto.function_arguments.into();
        let input_data_owner_list = data_owner_list_from_proto(proto.input_data_owner_list)?;
        let output_data_owner_list = data_owner_list_from_proto(proto.output_data_owner_list)?;
        let input_map = data_map_from_proto(proto.input_map)?;
        let output_map = data_map_from_proto(proto.output_map)?;
        let status = i32_to_task_status(proto.status)?;

        let ret = Self {
            task_id: proto.task_id,
            creator: proto.creator,
            function_id: proto.function_id,
            function_owner: proto.function_owner,
            function_arguments,
            input_data_owner_list,
            output_data_owner_list,
            participants: proto.participants.into_iter().collect(),
            approved_user_list: proto.approved_user_list.into_iter().collect(),
            input_map,
            output_map,
            return_value: proto.return_value,
            output_file_hash: proto.output_file_hash,
            status,
        };

        Ok(ret)
    }
}

impl From<GetTaskResponse> for proto::GetTaskResponse {
    fn from(response: GetTaskResponse) -> Self {
        let function_arguments = response.function_arguments.into();
        let input_data_owner_list = data_owner_list_to_proto(response.input_data_owner_list);
        let output_data_owner_list = data_owner_list_to_proto(response.output_data_owner_list);
        let input_map = data_map_to_proto(response.input_map);
        let output_map = data_map_to_proto(response.output_map);
        let status = i32_from_task_status(response.status);
        Self {
            task_id: response.task_id,
            creator: response.creator,
            function_id: response.function_id,
            function_owner: response.function_owner,
            function_arguments,
            input_data_owner_list,
            output_data_owner_list,
            participants: response.participants.into_iter().collect(),
            approved_user_list: response.approved_user_list.into_iter().collect(),
            input_map,
            output_map,
            return_value: response.return_value,
            output_file_hash: response.output_file_hash,
            status,
        }
    }
}

impl std::convert::TryFrom<proto::AssignDataRequest> for AssignDataRequest {
    type Error = Error;

    fn try_from(proto: proto::AssignDataRequest) -> Result<Self> {
        let input_map = data_map_from_proto(proto.input_map)?;
        let output_map = data_map_from_proto(proto.output_map)?;
        let ret = Self {
            task_id: proto.task_id,
            input_map,
            output_map,
        };

        Ok(ret)
    }
}

impl From<AssignDataRequest> for proto::AssignDataRequest {
    fn from(request: AssignDataRequest) -> Self {
        let input_map = data_map_to_proto(request.input_map);
        let output_map = data_map_to_proto(request.output_map);
        Self {
            task_id: request.task_id,
            input_map,
            output_map,
        }
    }
}

impl std::convert::TryFrom<proto::AssignDataResponse> for AssignDataResponse {
    type Error = Error;

    fn try_from(_proto: proto::AssignDataResponse) -> Result<Self> {
        Ok(AssignDataResponse)
    }
}

impl From<AssignDataResponse> for proto::AssignDataResponse {
    fn from(_response: AssignDataResponse) -> Self {
        Self {}
    }
}

impl std::convert::TryFrom<proto::ApproveTaskRequest> for ApproveTaskRequest {
    type Error = Error;

    fn try_from(proto: proto::ApproveTaskRequest) -> Result<Self> {
        let ret = Self {
            task_id: proto.task_id,
        };

        Ok(ret)
    }
}

impl From<ApproveTaskRequest> for proto::ApproveTaskRequest {
    fn from(request: ApproveTaskRequest) -> Self {
        Self {
            task_id: request.task_id,
        }
    }
}

impl std::convert::TryFrom<proto::ApproveTaskResponse> for ApproveTaskResponse {
    type Error = Error;

    fn try_from(_proto: proto::ApproveTaskResponse) -> Result<Self> {
        Ok(ApproveTaskResponse)
    }
}

impl From<ApproveTaskResponse> for proto::ApproveTaskResponse {
    fn from(_response: ApproveTaskResponse) -> Self {
        Self {}
    }
}

impl std::convert::TryFrom<proto::InvokeTaskRequest> for InvokeTaskRequest {
    type Error = Error;

    fn try_from(proto: proto::InvokeTaskRequest) -> Result<Self> {
        let ret = Self {
            task_id: proto.task_id,
        };

        Ok(ret)
    }
}

impl From<InvokeTaskRequest> for proto::InvokeTaskRequest {
    fn from(request: InvokeTaskRequest) -> Self {
        Self {
            task_id: request.task_id,
        }
    }
}

impl std::convert::TryFrom<proto::InvokeTaskResponse> for InvokeTaskResponse {
    type Error = Error;

    fn try_from(_proto: proto::InvokeTaskResponse) -> Result<Self> {
        Ok(InvokeTaskResponse)
    }
}

impl From<InvokeTaskResponse> for proto::InvokeTaskResponse {
    fn from(_response: InvokeTaskResponse) -> Self {
        Self {}
    }
}
