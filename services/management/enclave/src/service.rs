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

use crate::task::{
    assign_input_to_task, assign_output_to_task, try_update_task_to_approved_status,
    try_update_task_to_ready_status,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::collections::HashSet;
use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};
use teaclave_proto::teaclave_frontend_service::{
    ApproveTaskRequest, ApproveTaskResponse, AssignDataRequest, AssignDataResponse,
    CreateTaskRequest, CreateTaskResponse, GetFunctionRequest, GetFunctionResponse,
    GetInputFileRequest, GetInputFileResponse, GetOutputFileRequest, GetOutputFileResponse,
    GetTaskRequest, GetTaskResponse, InvokeTaskRequest, InvokeTaskResponse,
    RegisterFunctionRequest, RegisterFunctionResponse, RegisterFusionOutputRequest,
    RegisterFusionOutputResponse, RegisterInputFileRequest, RegisterInputFileResponse,
    RegisterInputFromOutputRequest, RegisterInputFromOutputResponse, RegisterOutputFileRequest,
    RegisterOutputFileResponse,
};
use teaclave_proto::teaclave_management_service::TeaclaveManagement;
use teaclave_proto::teaclave_storage_service::{
    EnqueueRequest, GetRequest, PutRequest, TeaclaveStorageClient,
};
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::Function;
#[cfg(test_mode)]
use teaclave_types::{FunctionInput, FunctionOutput};
use teaclave_types::{FunctionInputFile, FunctionOutputFile, StagedTask, Task, TaskStatus};
use teaclave_types::{Storable, TeaclaveInputFile, TeaclaveOutputFile};
use teaclave_types::{TeaclaveServiceResponseError, TeaclaveServiceResponseResult};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
enum TeaclaveManagementError {
    #[error("invalid request")]
    InvalidRequest,
    #[error("data error")]
    DataError,
    #[error("storage error")]
    StorageError,
    #[error("permission denied")]
    PermissionDenied,
    #[error("bad task")]
    BadTask,
}

impl From<TeaclaveManagementError> for TeaclaveServiceResponseError {
    fn from(error: TeaclaveManagementError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

#[teaclave_service(
    teaclave_management_service,
    TeaclaveManagement,
    TeaclaveManagementError
)]
#[derive(Clone)]
pub(crate) struct TeaclaveManagementService {
    storage_client: Arc<Mutex<TeaclaveStorageClient>>,
}

impl TeaclaveManagement for TeaclaveManagementService {
    // access control: none
    fn register_input_file(
        &self,
        request: Request<RegisterInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterInputFileResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();

        let request = request.message;
        let mut owner_list = HashSet::new();
        owner_list.insert(user_id);
        let input_file =
            TeaclaveInputFile::new(request.url, request.hash, request.crypto_info, owner_list);
        self.write_to_db(&input_file)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        let response = RegisterInputFileResponse {
            data_id: input_file.external_id(),
        };
        Ok(response)
    }

    // access control: none
    fn register_output_file(
        &self,
        request: Request<RegisterOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterOutputFileResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();

        let request = request.message;
        let mut owner_list = HashSet::new();
        owner_list.insert(user_id);
        let output_file = TeaclaveOutputFile::new(request.url, request.crypto_info, owner_list);
        self.write_to_db(&output_file)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        let response = RegisterOutputFileResponse {
            data_id: output_file.external_id(),
        };
        Ok(response)
    }

    // access control: user_id in owner_list
    fn register_fusion_output(
        &self,
        request: Request<RegisterFusionOutputRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFusionOutputResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();

        let owner_list = request.message.owner_list;
        if !owner_list.contains(&user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }

        let output_file = TeaclaveOutputFile::new_fusion_data(owner_list)
            .map_err(|_| TeaclaveManagementError::DataError)?;
        self.write_to_db(&output_file)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        let response = RegisterFusionOutputResponse {
            data_id: output_file.external_id(),
        };
        Ok(response)
    }

    // access control:
    // 1) user_id in output.owner
    // 2) hash != none
    fn register_input_from_output(
        &self,
        request: Request<RegisterInputFromOutputRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterInputFromOutputResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        if !TeaclaveOutputFile::match_prefix(&request.message.data_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let output: TeaclaveOutputFile = self
            .read_from_db(request.message.data_id.as_bytes())
            .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
        if !output.owner.contains(&user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }

        let input = TeaclaveInputFile::from_output(output)
            .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
        self.write_to_db(&input)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        let response = RegisterInputFromOutputResponse {
            data_id: input.external_id(),
        };
        Ok(response)
    }

    // access control: output_file.owner contains user_id
    fn get_output_file(
        &self,
        request: Request<GetOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetOutputFileResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();

        if !TeaclaveOutputFile::match_prefix(&request.message.data_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }

        let output_file: TeaclaveOutputFile = self
            .read_from_db(&request.message.data_id.as_bytes())
            .map_err(|_| TeaclaveManagementError::PermissionDenied)?;

        if !output_file.owner.contains(&user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let response = GetOutputFileResponse {
            owner: output_file.owner,
            hash: output_file.hash.unwrap_or_else(|| "".to_string()),
        };
        Ok(response)
    }

    // access control: input_file.owner contains user_id
    fn get_input_file(
        &self,
        request: Request<GetInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetInputFileResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();

        if !TeaclaveInputFile::match_prefix(&request.message.data_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }

        let input_file: TeaclaveInputFile = self
            .read_from_db(&request.message.data_id.as_bytes())
            .map_err(|_| TeaclaveManagementError::PermissionDenied)?;

        if !input_file.owner.contains(&user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let response = GetInputFileResponse {
            owner: input_file.owner,
            hash: input_file.hash,
        };
        Ok(response)
    }

    // access_control: none
    fn register_function(
        &self,
        request: Request<RegisterFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFunctionResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();

        let request = request.message;
        let function = Function::new()
            .id(Uuid::new_v4())
            .name(request.name)
            .description(request.description)
            .payload(request.payload)
            .public(request.is_public)
            .arguments(request.arg_list)
            .inputs(request.input_list)
            .outputs(request.output_list)
            .owner(user_id);

        self.write_to_db(&function)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        let response = RegisterFunctionResponse {
            function_id: function.external_id(),
        };
        Ok(response)
    }

    // access control: function.public || function.owner == user_id
    fn get_function(
        &self,
        request: Request<GetFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<GetFunctionResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        if !Function::match_prefix(&request.message.function_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }

        let function: Function = self
            .read_from_db(request.message.function_id.as_bytes())
            .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
        if !(function.public || function.owner == user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let response = GetFunctionResponse {
            name: function.name,
            description: function.description,
            owner: function.owner,
            payload: function.payload,
            is_public: function.public,
            arg_list: function.arguments,
            input_list: function.inputs,
            output_list: function.outputs,
        };
        Ok(response)
    }

    // access control: none
    // when a task is created, following rules will be verified:
    // 1) arugments match function definition
    // 2) input match function definition
    // 3) output match function definition
    fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> TeaclaveServiceResponseResult<CreateTaskResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let request = request.message;
        let function: Function = self
            .read_from_db(request.function_id.as_bytes())
            .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
        let task = crate::task::create_task(
            function,
            user_id,
            request.function_arguments,
            request.input_data_owner_list,
            request.output_data_owner_list,
        )
        .map_err(|_| TeaclaveManagementError::BadTask)?;
        self.write_to_db(&task)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        Ok(CreateTaskResponse {
            task_id: task.external_id(),
        })
    }

    // access control: task.participants.contains(&user_id)
    fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> TeaclaveServiceResponseResult<GetTaskResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();

        if !Task::match_prefix(&request.message.task_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let task: Task = self
            .read_from_db(request.message.task_id.as_bytes())
            .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
        if !task.participants.contains(&user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let response = GetTaskResponse {
            task_id: task.external_id(),
            creator: task.creator,
            function_id: task.function_id,
            function_owner: task.function_owner,
            function_arguments: task.function_arguments,
            input_data_owner_list: task.input_data_owner_list,
            output_data_owner_list: task.output_data_owner_list,
            participants: task.participants,
            approved_user_list: task.approved_user_list,
            input_map: task.input_map,
            output_map: task.output_map,
            status: task.status,
        };
        Ok(response)
    }

    // access control:
    // 1) task.participants.contains(user_id)
    // 2) task.status == Created
    // 3) user can use the data:
    //    * input file: user_id == input_file.owner contains user_id
    //    * output file: output_file.owner contains user_id && output_file.hash.is_none()
    // 4) the data can be assgined to the task:
    //    * input_data_owner_list or output_data_owner_list contains the data name
    //    * input file: DataOwnerList match input_file.owner
    //    * output file: DataOwnerList match output_file.owner
    fn assign_data(
        &self,
        request: Request<AssignDataRequest>,
    ) -> TeaclaveServiceResponseResult<AssignDataResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let request = request.message;
        if !Task::match_prefix(&request.task_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let mut task: Task = self
            .read_from_db(request.task_id.as_bytes())
            .map_err(|_| TeaclaveManagementError::PermissionDenied)?;

        if !task.participants.contains(&user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        match task.status {
            TaskStatus::Created => {}
            _ => return Err(TeaclaveManagementError::PermissionDenied.into()),
        }
        for (data_name, data_id) in request.input_map.iter() {
            if TeaclaveInputFile::match_prefix(data_id) {
                let input_file: TeaclaveInputFile = self
                    .read_from_db(data_id.as_bytes())
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
                assign_input_to_task(&mut task, data_name, &input_file, &user_id)
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
            } else {
                return Err(TeaclaveManagementError::PermissionDenied.into());
            }
        }
        for (data_name, data_id) in request.output_map.iter() {
            if TeaclaveOutputFile::match_prefix(data_id) {
                let output_file: TeaclaveOutputFile = self
                    .read_from_db(data_id.as_bytes())
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
                assign_output_to_task(&mut task, data_name, &output_file, &user_id)
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
            } else {
                return Err(TeaclaveManagementError::PermissionDenied.into());
            }
        }
        try_update_task_to_ready_status(&mut task);
        self.write_to_db(&task)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        Ok(AssignDataResponse)
    }

    // access_control:
    // 1) task status == Ready
    // 2) user_id in task.participants
    fn approve_task(
        &self,
        request: Request<ApproveTaskRequest>,
    ) -> TeaclaveServiceResponseResult<ApproveTaskResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let request = request.message;
        if !Task::match_prefix(&request.task_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let mut task: Task = self
            .read_from_db(request.task_id.as_bytes())
            .map_err(|_| TeaclaveManagementError::PermissionDenied)?;

        if !task.participants.contains(&user_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        match task.status {
            TaskStatus::Ready => {}
            _ => return Err(TeaclaveManagementError::PermissionDenied.into()),
        }
        task.approved_user_list.insert(user_id);
        try_update_task_to_approved_status(&mut task);
        self.write_to_db(&task)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        Ok(ApproveTaskResponse)
    }

    // access_control:
    // 1) task status == Approved
    // 2) user_id == task.creator
    fn invoke_task(
        &self,
        request: Request<InvokeTaskRequest>,
    ) -> TeaclaveServiceResponseResult<InvokeTaskResponse> {
        let user_id = request
            .metadata
            .get("id")
            .ok_or_else(|| TeaclaveManagementError::InvalidRequest)?
            .to_string();
        let request = request.message;
        if !Task::match_prefix(&request.task_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let mut task: Task = self
            .read_from_db(request.task_id.as_bytes())
            .map_err(|_| TeaclaveManagementError::PermissionDenied)?;

        if task.creator != user_id {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        match task.status {
            TaskStatus::Approved => {}
            _ => return Err(TeaclaveManagementError::PermissionDenied.into()),
        }
        if !Function::match_prefix(&task.function_id) {
            return Err(TeaclaveManagementError::PermissionDenied.into());
        }
        let function: Function = self
            .read_from_db(task.function_id.as_bytes())
            .map_err(|_| TeaclaveManagementError::PermissionDenied)?;

        let function_arguments = task.function_arguments.clone();
        let mut input_map: HashMap<String, FunctionInputFile> = HashMap::new();
        let mut output_map: HashMap<String, FunctionOutputFile> = HashMap::new();
        for (data_name, data_id) in task.input_map.iter() {
            let input_data: FunctionInputFile = if TeaclaveInputFile::match_prefix(data_id) {
                let input_file: TeaclaveInputFile = self
                    .read_from_db(data_id.as_bytes())
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
                FunctionInputFile::from_teaclave_input_file(&input_file)
            } else {
                return Err(TeaclaveManagementError::PermissionDenied.into());
            };
            input_map.insert(data_name.to_string(), input_data);
        }

        for (data_name, data_id) in task.output_map.iter() {
            let output_data: FunctionOutputFile = if TeaclaveOutputFile::match_prefix(data_id) {
                let output_file: TeaclaveOutputFile = self
                    .read_from_db(data_id.as_bytes())
                    .map_err(|_| TeaclaveManagementError::PermissionDenied)?;
                if output_file.hash.is_some() {
                    return Err(TeaclaveManagementError::PermissionDenied.into());
                }
                FunctionOutputFile::from_teaclave_output_file(&output_file)
            } else {
                return Err(TeaclaveManagementError::PermissionDenied.into());
            };
            output_map.insert(data_name.to_string(), output_data);
        }

        let staged_task = StagedTask::new()
            .task_id(task.task_id)
            .function_id(function.id)
            .function_name(&function.name)
            .function_payload(function.payload)
            .function_arguments(function_arguments)
            .input_data(input_map)
            .output_data(output_map);
        self.enqueue_to_db(StagedTask::get_queue_key().as_bytes(), &staged_task)?;
        task.status = TaskStatus::Running;
        self.write_to_db(&task)
            .map_err(|_| TeaclaveManagementError::StorageError)?;
        Ok(InvokeTaskResponse)
    }
}

impl TeaclaveManagementService {
    #[cfg(test_mode)]
    fn add_mock_data(&self) -> Result<()> {
        let mut owner = HashSet::new();
        owner.insert("mock_user1".to_string());
        owner.insert("frontend_user".to_string());
        let mut output_file = TeaclaveOutputFile::new_fusion_data(owner)?;
        output_file.uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001")?;
        output_file.hash = Some("deadbeef".to_string());
        self.write_to_db(&output_file)?;

        let mut owner = HashSet::new();
        owner.insert("mock_user2".to_string());
        owner.insert("mock_user3".to_string());
        let mut output_file = TeaclaveOutputFile::new_fusion_data(owner)?;
        output_file.uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000002")?;
        output_file.hash = Some("deadbeef".to_string());
        self.write_to_db(&output_file)?;
        let mut input_file = TeaclaveInputFile::from_output(output_file)?;
        input_file.uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000002")?;
        self.write_to_db(&input_file)?;

        let function_input = FunctionInput::new("input", "input_desc");
        let function_output = FunctionOutput::new("output", "output_desc");
        let function_input2 = FunctionInput::new("input2", "input_desc");
        let function_output2 = FunctionOutput::new("output2", "output_desc");

        let function = Function::new()
            .id(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
            .name("mock-func-1")
            .description("mock-desc")
            .payload(b"mock-payload".to_vec())
            .public(true)
            .arguments(vec!["arg1".to_string(), "arg2".to_string()])
            .inputs(vec![function_input, function_input2])
            .outputs(vec![function_output, function_output2])
            .owner("teaclave".to_string());

        self.write_to_db(&function)?;

        let function_output = FunctionOutput::new("output", "output_desc");
        let function = Function::new()
            .id(Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap())
            .name("mock-func-2")
            .description("mock-desc")
            .payload(b"mock-payload".to_vec())
            .public(true)
            .arguments(vec!["arg1".to_string()])
            .outputs(vec![function_output])
            .owner("teaclave".to_string());

        self.write_to_db(&function)?;
        Ok(())
    }

    pub(crate) fn new(storage_service_endpoint: Endpoint) -> Result<Self> {
        let mut i = 0;
        let channel = loop {
            match storage_service_endpoint.connect() {
                Ok(channel) => break channel,
                Err(_) => {
                    anyhow::ensure!(i < 3, "failed to connect to storage service");
                    log::debug!("Failed to connect to storage service, retry {}", i);
                    i += 1;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        };
        let storage_client = Arc::new(Mutex::new(TeaclaveStorageClient::new(channel)?));
        let service = Self { storage_client };
        #[cfg(test_mode)]
        service.add_mock_data()?;
        Ok(service)
    }

    fn write_to_db(&self, item: &impl Storable) -> Result<()> {
        let k = item.key();
        let v = item.to_vec()?;
        let put_request = PutRequest::new(k.as_slice(), v.as_slice());
        let _put_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("Cannot lock storage client"))?
            .put(put_request)?;
        Ok(())
    }

    fn read_from_db<T: Storable>(&self, key: &[u8]) -> Result<T> {
        let get_request = GetRequest::new(key);
        let get_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("Cannot lock storage client"))?
            .get(get_request)?;
        T::from_slice(get_response.value.as_slice())
    }

    fn enqueue_to_db(&self, key: &[u8], item: &impl Storable) -> TeaclaveServiceResponseResult<()> {
        let value = item
            .to_vec()
            .map_err(|_| TeaclaveManagementError::DataError)?;
        let enqueue_request = EnqueueRequest::new(key, value);
        let _enqueue_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| TeaclaveManagementError::StorageError)?
            .enqueue(enqueue_request)?;
        Ok(())
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use teaclave_types::{
        hashmap, FileCrypto, FunctionArguments, FunctionInput, FunctionOutput, TeaclaveFile128Key,
    };
    use url::Url;

    pub fn handle_input_file() {
        let url = Url::parse("s3://bucket_id/path?token=mock_token").unwrap();
        let hash = "a6d604b5987b693a19d94704532b5d928c2729f24dfd40745f8d03ac9ac75a8b".to_string();
        let mut user_id: HashSet<String> = HashSet::new();
        user_id.insert("mock_user".to_string());
        let crypto_info = FileCrypto::TeaclaveFile128(TeaclaveFile128Key::new(&[0; 16]).unwrap());
        let input_file = TeaclaveInputFile::new(url, hash, crypto_info, user_id);
        assert!(TeaclaveInputFile::match_prefix(&input_file.key_string()));
        let value = input_file.to_vec().unwrap();
        let deserialized_file = TeaclaveInputFile::from_slice(&value).unwrap();
        info!("file: {:?}", deserialized_file);
    }

    pub fn handle_output_file() {
        let url = Url::parse("s3://bucket_id/path?token=mock_token").unwrap();
        let mut user_id: HashSet<String> = HashSet::new();
        user_id.insert("mock_user".to_string());
        let crypto_info = FileCrypto::TeaclaveFile128(TeaclaveFile128Key::new(&[0; 16]).unwrap());
        let output_file = TeaclaveOutputFile::new(url, crypto_info, user_id);
        assert!(TeaclaveOutputFile::match_prefix(&output_file.key_string()));
        let value = output_file.to_vec().unwrap();
        let deserialized_file = TeaclaveOutputFile::from_slice(&value).unwrap();
        info!("file: {:?}", deserialized_file);
    }

    pub fn handle_function() {
        let function_input = FunctionInput::new("input", "input_desc");
        let function_output = FunctionOutput::new("output", "output_desc");
        let function = Function::new()
            .id(Uuid::new_v4())
            .name("mock_function")
            .description("mock function")
            .payload(b"python script".to_vec())
            .arguments(vec!["arg".to_string()])
            .inputs(vec![function_input])
            .outputs(vec![function_output])
            .public(true)
            .owner("mock_user");
        assert!(Function::match_prefix(&function.key_string()));
        let value = function.to_vec().unwrap();
        let deserialized_function = Function::from_slice(&value).unwrap();
        info!("function: {:?}", deserialized_function);
    }

    pub fn handle_task() {
        let function = Function::new()
            .id(Uuid::new_v4())
            .name("mock_function")
            .description("mock function")
            .payload(b"python script".to_vec())
            .arguments(vec!["arg".to_string()])
            .public(true)
            .owner("mock_user");
        let function_arguments = FunctionArguments::new(hashmap!("arg" => "data"));

        let task = crate::task::create_task(
            function,
            "mock_user".to_string(),
            function_arguments,
            HashMap::new(),
            HashMap::new(),
        )
        .unwrap();

        assert!(Task::match_prefix(&task.key_string()));
        let value = task.to_vec().unwrap();
        let deserialized_task = Task::from_slice(&value).unwrap();
        info!("task: {:?}", deserialized_task);
    }

    pub fn handle_staged_task() {
        let function = Function::new()
            .id(Uuid::new_v4())
            .name("mock_function")
            .description("mock function")
            .payload(b"python script".to_vec())
            .public(true)
            .owner("mock_user");
        let mut arg_list = HashMap::new();
        arg_list.insert("arg".to_string(), "data".to_string());
        let function_arguments = arg_list.into();

        let url = Url::parse("s3://bucket_id/path?token=mock_token").unwrap();
        let hash = "a6d604b5987b693a19d94704532b5d928c2729f24dfd40745f8d03ac9ac75a8b".to_string();
        let crypto_info = FileCrypto::TeaclaveFile128(TeaclaveFile128Key::new(&[0; 16]).unwrap());
        let input_data = FunctionInputFile::new(url.clone(), hash, crypto_info);
        let output_data = FunctionOutputFile::new(url, crypto_info);
        let mut input_map = HashMap::new();
        input_map.insert("input".to_string(), input_data);
        let mut output_map = HashMap::new();
        output_map.insert("output".to_string(), output_data);

        let staged_task = StagedTask::new()
            .task_id(Uuid::new_v4())
            .function_id(function.id)
            .function_name(&function.name)
            .function_payload(function.payload)
            .function_arguments(function_arguments)
            .input_data(input_map)
            .output_data(output_map);

        let value = staged_task.to_vec().unwrap();
        let deserialized_data = StagedTask::from_slice(&value).unwrap();
        info!("staged task: {:?}", deserialized_data);
    }
}
