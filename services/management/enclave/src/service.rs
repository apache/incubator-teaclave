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

use crate::error::TeaclaveManagementServiceError;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::convert::TryInto;
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
    RegisterOutputFileResponse, UpdateInputFileRequest, UpdateInputFileResponse,
    UpdateOutputFileRequest, UpdateOutputFileResponse,
};
use teaclave_proto::teaclave_management_service::TeaclaveManagement;
use teaclave_proto::teaclave_storage_service::{
    EnqueueRequest, GetRequest, PutRequest, TeaclaveStorageClient,
};
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::{ensure, teaclave_service};
use teaclave_types::*;
use url::Url;
use uuid::Uuid;

#[teaclave_service(
    teaclave_management_service,
    TeaclaveManagement,
    TeaclaveManagementServiceError
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
        let user_id = self.get_request_user_id(request.metadata())?;
        let request = request.message;
        let input_file = TeaclaveInputFile::new(
            request.url,
            request.cmac,
            request.crypto_info,
            vec![user_id],
        );

        self.write_to_db(&input_file)
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?;

        let response = RegisterInputFileResponse::new(input_file.external_id());
        Ok(response)
    }

    // access control:
    // 1) exisiting_file.owner_list.len() == 1
    // 2) user_id in existing_file.owner_list
    fn update_input_file(
        &self,
        request: Request<UpdateInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateInputFileResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;
        let request = request.message;

        let old_input_file: TeaclaveInputFile = self
            .read_from_db(&request.data_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        ensure!(
            old_input_file.owner == OwnerList::from(vec![user_id]),
            TeaclaveManagementServiceError::PermissionDenied
        );

        let input_file = TeaclaveInputFile::new(
            request.url,
            old_input_file.cmac,
            old_input_file.crypto_info,
            old_input_file.owner,
        );

        self.write_to_db(&input_file)
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?;

        let response = UpdateInputFileResponse::new(input_file.external_id());
        Ok(response)
    }

    // access control: none
    fn register_output_file(
        &self,
        request: Request<RegisterOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterOutputFileResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;
        let request = request.message;
        let output_file = TeaclaveOutputFile::new(request.url, request.crypto_info, vec![user_id]);

        self.write_to_db(&output_file)
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?;

        let response = RegisterOutputFileResponse::new(output_file.external_id());
        Ok(response)
    }

    // access control:
    // 1) exisiting_file.owner_list.len() == 1
    // 2) user_id in existing_file.owner_list
    fn update_output_file(
        &self,
        request: Request<UpdateOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateOutputFileResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;
        let request = request.message;

        let old_output_file: TeaclaveOutputFile = self
            .read_from_db(&request.data_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        ensure!(
            old_output_file.owner == OwnerList::from(vec![user_id]),
            TeaclaveManagementServiceError::PermissionDenied
        );

        let output_file = TeaclaveOutputFile::new(
            request.url,
            old_output_file.crypto_info,
            old_output_file.owner,
        );

        self.write_to_db(&output_file)
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?;

        let response = UpdateOutputFileResponse::new(output_file.external_id());
        Ok(response)
    }

    // access control: user_id in owner_list
    fn register_fusion_output(
        &self,
        request: Request<RegisterFusionOutputRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFusionOutputResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;

        let owner_list = request.message.owner_list;
        ensure!(
            owner_list.len() > 1 && owner_list.contains(&user_id),
            TeaclaveManagementServiceError::PermissionDenied
        );

        let output_file = self
            .create_fusion_data(owner_list)
            .map_err(|_| TeaclaveManagementServiceError::DataError)?;

        self.write_to_db(&output_file)
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?;

        let response = RegisterFusionOutputResponse::new(output_file.external_id());
        Ok(response)
    }

    // access control:
    // 1) user_id in output.owner
    // 2) cmac != none
    fn register_input_from_output(
        &self,
        request: Request<RegisterInputFromOutputRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterInputFromOutputResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;

        let output: TeaclaveOutputFile = self
            .read_from_db(&request.message.data_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        ensure!(
            output.owner.contains(&user_id),
            TeaclaveManagementServiceError::PermissionDenied
        );

        let input = TeaclaveInputFile::from_output(output)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        self.write_to_db(&input)
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?;

        let response = RegisterInputFromOutputResponse::new(input.external_id());
        Ok(response)
    }

    // access control: output_file.owner contains user_id
    fn get_output_file(
        &self,
        request: Request<GetOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetOutputFileResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;

        let output_file: TeaclaveOutputFile = self
            .read_from_db(&request.message.data_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        ensure!(
            output_file.owner.contains(&user_id),
            TeaclaveManagementServiceError::PermissionDenied
        );

        let response = GetOutputFileResponse::new(output_file.owner, output_file.cmac);
        Ok(response)
    }

    // access control: input_file.owner contains user_id
    fn get_input_file(
        &self,
        request: Request<GetInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetInputFileResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;

        let input_file: TeaclaveInputFile = self
            .read_from_db(&request.message.data_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        ensure!(
            input_file.owner.contains(&user_id),
            TeaclaveManagementServiceError::PermissionDenied
        );

        let response = GetInputFileResponse::new(input_file.owner, input_file.cmac);
        Ok(response)
    }

    // access_control: none
    fn register_function(
        &self,
        request: Request<RegisterFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFunctionResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;

        let function = Function::from(request.message)
            .id(Uuid::new_v4())
            .owner(user_id);

        self.write_to_db(&function)
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?;

        let response = RegisterFunctionResponse::new(function.external_id());
        Ok(response)
    }

    // access control: function.public || function.owner == user_id
    fn get_function(
        &self,
        request: Request<GetFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<GetFunctionResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;

        let function: Function = self
            .read_from_db(&request.message.function_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        ensure!(
            (function.public || function.owner == user_id),
            TeaclaveManagementServiceError::PermissionDenied
        );

        let response = GetFunctionResponse {
            name: function.name,
            description: function.description,
            owner: function.owner,
            executor_type: function.executor_type,
            payload: function.payload,
            public: function.public,
            arguments: function.arguments,
            inputs: function.inputs,
            outputs: function.outputs,
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
        let user_id = self.get_request_user_id(request.metadata())?;

        let request = request.message;

        let function: Function = self
            .read_from_db(&request.function_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        let task = Task::<Create>::new(
            user_id,
            request.executor,
            request.function_arguments,
            request.inputs_ownership,
            request.outputs_ownership,
            function,
        )
        .map_err(|_| TeaclaveManagementServiceError::BadTask)?;

        log::debug!("CreateTask: {:?}", task);

        let ts: TaskState = task.into();
        self.write_to_db(&ts)
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?;

        let response = CreateTaskResponse::new(ts.external_id());
        Ok(response)
    }

    // access control: task.participants.contains(&user_id)
    fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> TeaclaveServiceResponseResult<GetTaskResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;

        let ts: TaskState = self
            .read_from_db(&request.message.task_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        ensure!(
            ts.has_participant(&user_id),
            TeaclaveManagementServiceError::PermissionDenied
        );

        log::debug!("GetTask: {:?}", ts);

        let response = GetTaskResponse {
            task_id: ts.external_id(),
            creator: ts.creator,
            function_id: ts.function_id,
            function_owner: ts.function_owner,
            function_arguments: ts.function_arguments,
            inputs_ownership: ts.inputs_ownership,
            outputs_ownership: ts.outputs_ownership,
            participants: ts.participants,
            approved_users: ts.approved_users,
            assigned_inputs: ts.assigned_inputs.external_ids(),
            assigned_outputs: ts.assigned_outputs.external_ids(),
            result: ts.result,
            status: ts.status,
        };
        Ok(response)
    }

    // access control:
    // 1) task.participants.contains(user_id)
    // 2) task.status == Created
    // 3) user can use the data:
    //    * input file: user_id == input_file.owner contains user_id
    //    * output file: output_file.owner contains user_id && output_file.cmac.is_none()
    // 4) the data can be assgined to the task:
    //    * inputs_ownership or outputs_ownership contains the data name
    //    * input file: OwnerList match input_file.owner
    //    * output file: OwnerList match output_file.owner
    fn assign_data(
        &self,
        request: Request<AssignDataRequest>,
    ) -> TeaclaveServiceResponseResult<AssignDataResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;

        let request = request.message;

        let ts: TaskState = self
            .read_from_db(&request.task_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        ensure!(
            ts.has_participant(&user_id),
            TeaclaveManagementServiceError::PermissionDenied
        );

        let mut task: Task<Assign> = ts.try_into().map_err(|e| {
            log::warn!("Assign state error: {:?}", e);
            TeaclaveManagementServiceError::PermissionDenied
        })?;

        for (data_name, data_id) in request.inputs.iter() {
            let file: TeaclaveInputFile = self
                .read_from_db(&data_id)
                .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;
            task.assign_input(&user_id, data_name, file)
                .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;
        }

        for (data_name, data_id) in request.outputs.iter() {
            let file: TeaclaveOutputFile = self
                .read_from_db(&data_id)
                .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;
            task.assign_output(&user_id, data_name, file)
                .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;
        }

        log::debug!("AssignData: {:?}", task);

        let ts: TaskState = task.into();
        self.write_to_db(&ts)
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?;

        Ok(AssignDataResponse)
    }

    // access_control:
    // 1) task status == Ready
    // 2) user_id in task.participants
    fn approve_task(
        &self,
        request: Request<ApproveTaskRequest>,
    ) -> TeaclaveServiceResponseResult<ApproveTaskResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;

        let request = request.message;
        let ts: TaskState = self
            .read_from_db(&request.task_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        let mut task: Task<Approve> = ts.try_into().map_err(|e| {
            log::warn!("Approve state error: {:?}", e);
            TeaclaveManagementServiceError::PermissionDenied
        })?;

        task.approve(&user_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        log::debug!("ApproveTask: approve:{:?}", task);

        let ts: TaskState = task.into();
        self.write_to_db(&ts)
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?;

        Ok(ApproveTaskResponse)
    }

    // access_control:
    // 1) task status == Approved
    // 2) user_id == task.creator
    fn invoke_task(
        &self,
        request: Request<InvokeTaskRequest>,
    ) -> TeaclaveServiceResponseResult<InvokeTaskResponse> {
        let user_id = self.get_request_user_id(request.metadata())?;
        let request = request.message;

        let ts: TaskState = self
            .read_from_db(&request.task_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        // Early validation
        ensure!(
            ts.has_creator(&user_id),
            TeaclaveManagementServiceError::PermissionDenied
        );

        let function: Function = self
            .read_from_db(&ts.function_id)
            .map_err(|_| TeaclaveManagementServiceError::PermissionDenied)?;

        log::debug!("InvokeTask: get function: {:?}", function);

        let mut task: Task<Stage> = ts.try_into().map_err(|e| {
            log::warn!("Stage state error: {:?}", e);
            TeaclaveManagementServiceError::PermissionDenied
        })?;

        log::debug!("InvokeTask: get task: {:?}", task);

        let staged_task = task.stage_for_running(&user_id, function)?;

        log::debug!("InvokeTask: staged task: {:?}", staged_task);

        self.enqueue_to_db(StagedTask::get_queue_key().as_bytes(), &staged_task)?;

        let ts: TaskState = task.into();
        self.write_to_db(&ts)
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?;

        Ok(InvokeTaskResponse)
    }
}

impl TeaclaveManagementService {
    pub(crate) fn new(storage_service_endpoint: Endpoint) -> Result<Self> {
        let mut i = 0;
        let channel = loop {
            match storage_service_endpoint.connect() {
                Ok(channel) => break channel,
                Err(_) => {
                    anyhow::ensure!(i < 10, "failed to connect to storage service");
                    log::debug!("Failed to connect to storage service, retry {}", i);
                    i += 1;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(3));
        };
        let storage_client = Arc::new(Mutex::new(TeaclaveStorageClient::new(channel)?));
        let service = Self { storage_client };

        #[cfg(test_mode)]
        service.add_mock_data()?;

        Ok(service)
    }

    pub fn create_fusion_data(&self, owners: impl Into<OwnerList>) -> Result<TeaclaveOutputFile> {
        let uuid = Uuid::new_v4();
        let url = format!("fusion:///TEACLAVE_FUSION_BASE/{}.fusion", uuid.to_string());
        let url = Url::parse(&url).map_err(|_| anyhow!("invalid url"))?;
        let crypto_info = FileCrypto::default();

        Ok(TeaclaveOutputFile::new(url, crypto_info, owners))
    }

    fn get_request_user_id(
        &self,
        meta: &HashMap<String, String>,
    ) -> TeaclaveServiceResponseResult<UserID> {
        let user_id = meta
            .get("id")
            .ok_or(TeaclaveManagementServiceError::InvalidRequest)?;
        Ok(user_id.to_string().into())
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

    fn read_from_db<T: Storable>(&self, key: &ExternalID) -> Result<T> {
        anyhow::ensure!(T::match_prefix(&key.prefix), "Key prefix doesn't match.");

        let request = GetRequest::new(key.to_bytes());
        let response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("Cannot lock storage client"))?
            .get(request)?;
        T::from_slice(response.value.as_slice())
    }

    fn enqueue_to_db(&self, key: &[u8], item: &impl Storable) -> TeaclaveServiceResponseResult<()> {
        let value = item
            .to_vec()
            .map_err(|_| TeaclaveManagementServiceError::DataError)?;
        let enqueue_request = EnqueueRequest::new(key, value);
        let _enqueue_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| TeaclaveManagementServiceError::StorageError)?
            .enqueue(enqueue_request)?;
        Ok(())
    }

    #[cfg(test_mode)]
    fn add_mock_data(&self) -> Result<()> {
        let mut output_file = self.create_fusion_data(vec!["mock_user1", "frontend_user"])?;
        output_file.uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001")?;
        output_file.cmac = Some(FileAuthTag::mock());
        self.write_to_db(&output_file)?;

        let mut output_file = self.create_fusion_data(vec!["mock_user2", "mock_user3"])?;
        output_file.uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000002")?;
        output_file.cmac = Some(FileAuthTag::mock());
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
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use teaclave_types::{
        hashmap, Executor, FileAuthTag, FileCrypto, FunctionArguments, FunctionInput,
        FunctionInputFile, FunctionOutput, FunctionOutputFile,
    };
    use url::Url;

    pub fn handle_input_file() {
        let url = Url::parse("s3://bucket_id/path?token=mock_token").unwrap();
        let cmac = FileAuthTag::mock();
        let input_file =
            TeaclaveInputFile::new(url, cmac, FileCrypto::default(), vec!["mock_user"]);
        assert!(TeaclaveInputFile::match_prefix(&input_file.key_string()));
        let value = input_file.to_vec().unwrap();
        let deserialized_file = TeaclaveInputFile::from_slice(&value).unwrap();
        debug!("file: {:?}", deserialized_file);
    }

    pub fn handle_output_file() {
        let url = Url::parse("s3://bucket_id/path?token=mock_token").unwrap();
        let output_file = TeaclaveOutputFile::new(url, FileCrypto::default(), vec!["mock_user"]);
        assert!(TeaclaveOutputFile::match_prefix(&output_file.key_string()));
        let value = output_file.to_vec().unwrap();
        let deserialized_file = TeaclaveOutputFile::from_slice(&value).unwrap();
        debug!("file: {:?}", deserialized_file);
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
        debug!("function: {:?}", deserialized_function);
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
        let function_arguments = FunctionArguments::from_json(json!({"arg": "data"})).unwrap();

        let task = Task::<Create>::new(
            UserID::from("mock_user"),
            Executor::MesaPy,
            function_arguments,
            HashMap::new(),
            HashMap::new(),
            function,
        )
        .unwrap();

        let ts: TaskState = task.try_into().unwrap();
        let value = ts.to_vec().unwrap();
        let deserialized_task = TaskState::from_slice(&value).unwrap();
        debug!("task: {:?}", deserialized_task);
    }

    pub fn handle_staged_task() {
        let function = Function::new()
            .id(Uuid::new_v4())
            .name("mock_function")
            .description("mock function")
            .payload(b"python script".to_vec())
            .public(true)
            .owner("mock_user");

        let url = Url::parse("s3://bucket_id/path?token=mock_token").unwrap();
        let cmac = FileAuthTag::mock();
        let input_data = FunctionInputFile::new(url.clone(), cmac, FileCrypto::default());
        let output_data = FunctionOutputFile::new(url, FileCrypto::default());

        let staged_task = StagedTask::new()
            .task_id(Uuid::new_v4())
            .executor(Executor::MesaPy)
            .function_payload(function.payload)
            .function_arguments(hashmap!("arg" => "data"))
            .input_data(hashmap!("input" => input_data))
            .output_data(hashmap!("output" => output_data));

        let value = staged_task.to_vec().unwrap();
        let deserialized_data = StagedTask::from_slice(&value).unwrap();
        debug!("staged task: {:?}", deserialized_data);
    }
}
