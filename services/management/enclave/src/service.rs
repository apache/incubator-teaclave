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

use crate::error::ManagementServiceError;
use anyhow::anyhow;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use teaclave_proto::teaclave_frontend_service::{
    ApproveTaskRequest, ApproveTaskResponse, AssignDataRequest, AssignDataResponse,
    CancelTaskRequest, CancelTaskResponse, CreateTaskRequest, CreateTaskResponse,
    DeleteFunctionRequest, DeleteFunctionResponse, DisableFunctionRequest, DisableFunctionResponse,
    GetFunctionRequest, GetFunctionResponse, GetFunctionUsageStatsRequest,
    GetFunctionUsageStatsResponse, GetInputFileRequest, GetInputFileResponse, GetOutputFileRequest,
    GetOutputFileResponse, GetTaskRequest, GetTaskResponse, InvokeTaskRequest, InvokeTaskResponse,
    ListFunctionsRequest, ListFunctionsResponse, RegisterFunctionRequest, RegisterFunctionResponse,
    RegisterFusionOutputRequest, RegisterFusionOutputResponse, RegisterInputFileRequest,
    RegisterInputFileResponse, RegisterInputFromOutputRequest, RegisterInputFromOutputResponse,
    RegisterOutputFileRequest, RegisterOutputFileResponse, UpdateFunctionRequest,
    UpdateFunctionResponse, UpdateInputFileRequest, UpdateInputFileResponse,
    UpdateOutputFileRequest, UpdateOutputFileResponse,
};
use teaclave_proto::teaclave_management_service::TeaclaveManagement;
use teaclave_proto::teaclave_storage_service::{
    DeleteRequest, EnqueueRequest, GetKeysByPrefixRequest, GetRequest, PutRequest,
    TeaclaveStorageClient,
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
    ManagementServiceError
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
        let user_id = get_request_user_id(&request)?;
        let request = request.message;
        let input_file = TeaclaveInputFile::new(
            request.url,
            request.cmac,
            request.crypto_info,
            vec![user_id],
        );

        self.write_to_db(&input_file)?;

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
        let user_id = get_request_user_id(&request)?;
        let request = request.message;

        let old_input_file: TeaclaveInputFile = self
            .read_from_db(&request.data_id)
            .map_err(|_| ManagementServiceError::InvalidDataId)?;

        ensure!(
            old_input_file.owner == OwnerList::from(vec![user_id]),
            ManagementServiceError::PermissionDenied
        );

        let input_file = TeaclaveInputFile::new(
            request.url,
            old_input_file.cmac,
            old_input_file.crypto_info,
            old_input_file.owner,
        );

        self.write_to_db(&input_file)?;

        let response = UpdateInputFileResponse::new(input_file.external_id());
        Ok(response)
    }

    // access control: none
    fn register_output_file(
        &self,
        request: Request<RegisterOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterOutputFileResponse> {
        let user_id = get_request_user_id(&request)?;
        let request = request.message;
        let output_file = TeaclaveOutputFile::new(request.url, request.crypto_info, vec![user_id]);

        self.write_to_db(&output_file)?;

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
        let user_id = get_request_user_id(&request)?;
        let request = request.message;

        let old_output_file: TeaclaveOutputFile = self
            .read_from_db(&request.data_id)
            .map_err(|_| ManagementServiceError::InvalidDataId)?;

        ensure!(
            old_output_file.owner == OwnerList::from(vec![user_id]),
            ManagementServiceError::PermissionDenied
        );

        let output_file = TeaclaveOutputFile::new(
            request.url,
            old_output_file.crypto_info,
            old_output_file.owner,
        );

        self.write_to_db(&output_file)?;

        let response = UpdateOutputFileResponse::new(output_file.external_id());
        Ok(response)
    }

    // access control: user_id in owner_list
    fn register_fusion_output(
        &self,
        request: Request<RegisterFusionOutputRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFusionOutputResponse> {
        let user_id = get_request_user_id(&request)?;

        let owner_list = request.message.owner_list;
        ensure!(
            owner_list.len() > 1 && owner_list.contains(&user_id),
            ManagementServiceError::PermissionDenied
        );

        let output_file = create_fusion_data(owner_list)?;

        self.write_to_db(&output_file)?;

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
        let user_id = get_request_user_id(&request)?;

        let output: TeaclaveOutputFile = self
            .read_from_db(&request.message.data_id)
            .map_err(|_| ManagementServiceError::InvalidDataId)?;

        ensure!(
            output.owner.contains(&user_id),
            ManagementServiceError::PermissionDenied
        );

        let input = TeaclaveInputFile::from_output(output)
            .map_err(|_| ManagementServiceError::InvalidOutputFile)?;

        self.write_to_db(&input)?;

        let response = RegisterInputFromOutputResponse::new(input.external_id());
        Ok(response)
    }

    // access control: output_file.owner contains user_id
    fn get_output_file(
        &self,
        request: Request<GetOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetOutputFileResponse> {
        let user_id = get_request_user_id(&request)?;

        let output_file: TeaclaveOutputFile = self
            .read_from_db(&request.message.data_id)
            .map_err(|_| ManagementServiceError::InvalidDataId)?;

        ensure!(
            output_file.owner.contains(&user_id),
            ManagementServiceError::PermissionDenied
        );

        let response = GetOutputFileResponse::new(output_file.owner, output_file.cmac);
        Ok(response)
    }

    // access control: input_file.owner contains user_id
    fn get_input_file(
        &self,
        request: Request<GetInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetInputFileResponse> {
        let user_id = get_request_user_id(&request)?;

        let input_file: TeaclaveInputFile = self
            .read_from_db(&request.message.data_id)
            .map_err(|_| ManagementServiceError::InvalidDataId)?;

        ensure!(
            input_file.owner.contains(&user_id),
            ManagementServiceError::PermissionDenied
        );

        let response = GetInputFileResponse::new(input_file.owner, input_file.cmac);
        Ok(response)
    }

    // access_control: none
    fn register_function(
        &self,
        request: Request<RegisterFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFunctionResponse> {
        let user_id = get_request_user_id(&request)?;

        let function = FunctionBuilder::from(request.message)
            .id(Uuid::new_v4())
            .owner(user_id.clone())
            .build();

        self.write_to_db(&function)?;

        let mut u = User {
            id: user_id,
            ..Default::default()
        };
        let external_id = u.external_id();

        let user = self.read_from_db::<User>(&external_id);
        match user {
            Ok(mut us) => {
                us.registered_functions
                    .push(function.external_id().to_string());
                self.write_to_db(&us)?;
            }
            Err(_) => {
                u.registered_functions
                    .push(function.external_id().to_string());
                self.write_to_db(&u)?;
            }
        }

        // Update allowed function list for users
        for user_id in &function.user_allowlist {
            let mut u = User {
                id: user_id.into(),
                ..Default::default()
            };
            let external_id = u.external_id();
            let user = self.read_from_db::<User>(&external_id);
            match user {
                Ok(mut us) => {
                    us.allowed_functions
                        .push(function.external_id().to_string());
                    self.write_to_db(&us)?;
                }
                Err(_) => {
                    u.allowed_functions.push(function.external_id().to_string());
                    self.write_to_db(&u)?;
                }
            }
        }

        let usage = FunctionUsage {
            function_id: function.id,
            ..Default::default()
        };
        self.write_to_db(&usage)?;

        let response = RegisterFunctionResponse::new(function.external_id());
        Ok(response)
    }

    fn update_function(
        &self,
        request: Request<UpdateFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateFunctionResponse> {
        let user_id = get_request_user_id(&request)?;

        let function = FunctionBuilder::from(request.message)
            .owner(user_id)
            .build();

        self.write_to_db(&function)?;

        let response = UpdateFunctionResponse::new(function.external_id());
        Ok(response)
    }

    // access control: function.public || function.owner == user_id || request.role == PlatformAdmin
    fn get_function(
        &self,
        request: Request<GetFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<GetFunctionResponse> {
        let user_id = get_request_user_id(&request)?;

        let function: Function = self
            .read_from_db(&request.message.function_id)
            .map_err(|_| ManagementServiceError::InvalidFunctionId)?;
        let role = get_request_role(&request)?;

        if function.public || role == UserRole::PlatformAdmin || function.owner == user_id {
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
                user_allowlist: function.user_allowlist,
            };

            Ok(response)
        } else if !function.public && function.user_allowlist.contains(&user_id.into()) {
            let response = GetFunctionResponse {
                name: function.name,
                description: function.description,
                owner: function.owner,
                executor_type: function.executor_type,
                payload: vec![],
                public: function.public,
                arguments: function.arguments,
                inputs: function.inputs,
                outputs: function.outputs,
                user_allowlist: vec![],
            };

            Ok(response)
        } else {
            Err(ManagementServiceError::PermissionDenied.into())
        }
    }

    // access control:
    // function.public || request.role == PlatformAdmin || requested user_id in the user_allowlist
    fn get_function_usage_stats(
        &self,
        request: Request<GetFunctionUsageStatsRequest>,
    ) -> TeaclaveServiceResponseResult<GetFunctionUsageStatsResponse> {
        let user_id = get_request_user_id(&request)?;
        let role = get_request_role(&request)?;
        let request = request.message;
        let function: Function = self
            .read_from_db(&request.function_id)
            .map_err(|_| ManagementServiceError::InvalidFunctionId)?;

        ensure!(
            function.public
                || role == UserRole::PlatformAdmin
                || function.user_allowlist.contains(&user_id.to_string()),
            ManagementServiceError::PermissionDenied
        );

        let usage = FunctionUsage {
            function_id: function.id,
            ..Default::default()
        };
        let external_id = usage.external_id();
        let function_usage = self
            .read_from_db::<FunctionUsage>(&external_id)
            .map_err(|_| ManagementServiceError::InvalidFunctionId)?;
        let function_quota = function.usage_quota.unwrap_or(-1);
        let response = GetFunctionUsageStatsResponse {
            function_quota,
            current_usage: function_usage.use_numbers,
        };
        Ok(response)
    }

    // access control: function.owner == user_id
    fn delete_function(
        &self,
        request: Request<DeleteFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<DeleteFunctionResponse> {
        let user_id = get_request_user_id(&request)?;

        let function: Function = self
            .read_from_db(&request.message.function_id)
            .map_err(|_| ManagementServiceError::InvalidFunctionId)?;

        ensure!(
            function.owner == user_id,
            ManagementServiceError::PermissionDenied
        );
        self.delete_from_db(&request.message.function_id)
            .map_err(|_| ManagementServiceError::InvalidFunctionId)?;
        let response = DeleteFunctionResponse {};
        Ok(response)
    }

    // access control: function.owner == user_id
    // disable function
    // 1. `List functions` does not show this function
    // 2. `Create new task` with the function id fails
    fn disable_function(
        &self,
        request: Request<DisableFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<DisableFunctionResponse> {
        let user_id = get_request_user_id(&request)?;
        let role = get_request_role(&request)?;

        let mut function: Function = self
            .read_from_db(&request.message.function_id)
            .map_err(|_| ManagementServiceError::InvalidFunctionId)?;

        if role != UserRole::PlatformAdmin {
            ensure!(
                function.owner == user_id,
                ManagementServiceError::PermissionDenied
            );
        }
        let func_id = function.external_id().to_string();

        // Updated function owner
        let u = User {
            id: function.owner.clone(),
            ..Default::default()
        };
        let external_id = u.external_id();
        let user = self.read_from_db::<User>(&external_id);
        if let Ok(mut us) = user {
            us.allowed_functions.retain(|f| !f.eq(&func_id));
            us.registered_functions.retain(|f| !f.eq(&func_id));
            self.write_to_db(&us)?;
        } else {
            log::warn!("Invalid user id from functions");
        }

        // Update allowed function list for users
        for user_id in &function.user_allowlist {
            let u = User {
                id: user_id.into(),
                ..Default::default()
            };
            let external_id = u.external_id();
            let user = self.read_from_db::<User>(&external_id);
            if let Ok(mut us) = user {
                us.allowed_functions.retain(|f| !f.eq(&func_id));
                us.registered_functions.retain(|f| !f.eq(&func_id));
                self.write_to_db(&us)?;
            } else {
                log::warn!("Invalid user id from functions");
            }
        }

        function.user_allowlist.clear();
        self.write_to_db(&function)?;
        let response = DisableFunctionResponse {};
        Ok(response)
    }

    // access contro: user_id = request.user_id
    fn list_functions(
        &self,
        request: Request<ListFunctionsRequest>,
    ) -> TeaclaveServiceResponseResult<ListFunctionsResponse> {
        let mut request_user_id = request.message.user_id.clone();

        let current_user_id = get_request_user_id(&request)?;
        let role = get_request_role(&request)?;

        if role != UserRole::PlatformAdmin {
            ensure!(
                request_user_id == current_user_id,
                ManagementServiceError::PermissionDenied
            );
        }

        if let UserRole::DataOwner(s) = &role {
            request_user_id = s.into();
        }

        let u = User {
            id: request_user_id,
            ..Default::default()
        };
        let external_id = u.external_id();

        let user = self.read_from_db::<User>(&external_id);
        match user {
            Ok(us) => {
                let mut response = ListFunctionsResponse {
                    registered_functions: us.registered_functions,
                    allowed_functions: us.allowed_functions,
                };
                if role == UserRole::PlatformAdmin {
                    let allowed_functions =
                        self.get_keys_by_prefix_from_db(Function::key_prefix())?;
                    response.allowed_functions = allowed_functions;
                }

                Ok(response)
            }
            Err(_) => {
                let response = ListFunctionsResponse::default();
                Ok(response)
            }
        }
    }

    // access control: none
    // when a task is created, following rules will be verified:
    // 1) arugments match function definition
    // 2) input files match function definition
    // 3) output files match function definition
    // 4) requested user_id in the user_allowlist
    fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> TeaclaveServiceResponseResult<CreateTaskResponse> {
        let user_id = get_request_user_id(&request)?;
        let role = get_request_role(&request)?;

        let request = request.message;

        let function: Function = self
            .read_from_db(&request.function_id)
            .map_err(|_| ManagementServiceError::InvalidFunctionId)?;

        match role {
            UserRole::DataOwner(a) | UserRole::DataOwnerManager(a) => {
                ensure!(
                    (function.public || function.user_allowlist.contains(&a)),
                    ManagementServiceError::PermissionDenied
                );
            }
            UserRole::PlatformAdmin => (),
            _ => {
                return Err(ManagementServiceError::PermissionDenied.into());
            }
        }
        let task = Task::<Create>::new(
            user_id,
            request.executor,
            request.function_arguments,
            request.inputs_ownership,
            request.outputs_ownership,
            function,
        )
        .map_err(|_| ManagementServiceError::InvalidTask)?;

        log::debug!("CreateTask: {:?}", task);
        let ts: TaskState = task.into();
        self.write_to_db(&ts)?;

        let response = CreateTaskResponse::new(ts.external_id());
        Ok(response)
    }

    // access control: task.participants.contains(&user_id)
    fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> TeaclaveServiceResponseResult<GetTaskResponse> {
        let user_id = get_request_user_id(&request)?;

        let ts: TaskState = self
            .read_from_db(&request.message.task_id)
            .map_err(|_| ManagementServiceError::InvalidTaskId)?;

        ensure!(
            ts.has_participant(&user_id),
            ManagementServiceError::PermissionDenied
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
        let user_id = get_request_user_id(&request)?;

        let request = request.message;

        let ts: TaskState = self
            .read_from_db(&request.task_id)
            .map_err(|_| ManagementServiceError::InvalidTaskId)?;

        ensure!(
            ts.has_participant(&user_id),
            ManagementServiceError::PermissionDenied
        );

        let mut task: Task<Assign> = ts.try_into().map_err(|e| {
            log::warn!("Assign state error: {:?}", e);
            ManagementServiceError::TaskAssignDataError
        })?;

        for (data_name, data_id) in request.inputs.iter() {
            let file: TeaclaveInputFile = self
                .read_from_db(data_id)
                .map_err(|_| ManagementServiceError::InvalidDataId)?;
            task.assign_input(&user_id, data_name, file)
                .map_err(|_| ManagementServiceError::PermissionDenied)?;
        }

        for (data_name, data_id) in request.outputs.iter() {
            let file: TeaclaveOutputFile = self
                .read_from_db(data_id)
                .map_err(|_| ManagementServiceError::InvalidDataId)?;
            task.assign_output(&user_id, data_name, file)
                .map_err(|_| ManagementServiceError::PermissionDenied)?;
        }

        log::debug!("AssignData: {:?}", task);

        let ts: TaskState = task.into();
        self.write_to_db(&ts)?;

        Ok(AssignDataResponse)
    }

    // access_control:
    // 1) task status == Ready
    // 2) user_id in task.participants
    fn approve_task(
        &self,
        request: Request<ApproveTaskRequest>,
    ) -> TeaclaveServiceResponseResult<ApproveTaskResponse> {
        let user_id = get_request_user_id(&request)?;

        let request = request.message;
        let ts: TaskState = self
            .read_from_db(&request.task_id)
            .map_err(|_| ManagementServiceError::InvalidTaskId)?;

        let mut task: Task<Approve> = ts.try_into().map_err(|e| {
            log::warn!("Approve state error: {:?}", e);
            ManagementServiceError::TaskApproveError
        })?;

        task.approve(&user_id)
            .map_err(|_| ManagementServiceError::PermissionDenied)?;

        log::debug!("ApproveTask: approve:{:?}", task);

        let ts: TaskState = task.into();
        self.write_to_db(&ts)?;

        Ok(ApproveTaskResponse)
    }

    // access_control:
    // 1) task status == Approved
    // 2) user_id == task.creator
    fn invoke_task(
        &self,
        request: Request<InvokeTaskRequest>,
    ) -> TeaclaveServiceResponseResult<InvokeTaskResponse> {
        let user_id = get_request_user_id(&request)?;
        let request = request.message;

        let ts: TaskState = self
            .read_from_db(&request.task_id)
            .map_err(|_| ManagementServiceError::InvalidTaskId)?;

        // Early validation
        ensure!(
            ts.has_creator(&user_id),
            ManagementServiceError::PermissionDenied
        );

        let function: Function = self
            .read_from_db(&ts.function_id)
            .map_err(|_| ManagementServiceError::InvalidFunctionId)?;

        log::debug!("InvokeTask: get function: {:?}", function);

        let usage = FunctionUsage {
            function_id: function.id,
            ..Default::default()
        };
        let external_id = usage.external_id();
        let mut function_usage = self
            .read_from_db::<FunctionUsage>(&external_id)
            .map_err(|_| ManagementServiceError::InvalidFunctionId)?;
        let function_current_use_numbers = function_usage.use_numbers;

        if let Some(quota) = function.usage_quota {
            if quota <= function_current_use_numbers {
                return Err(ManagementServiceError::FunctionQuotaError.into());
            }
        }

        let mut task: Task<Stage> = ts.try_into().map_err(|e| {
            log::warn!("Stage state error: {:?}", e);
            ManagementServiceError::TaskInvokeError
        })?;

        log::debug!("InvokeTask: get task: {:?}", task);
        let staged_task = task
            .stage_for_running(&user_id, function)
            .map_err(|_| ManagementServiceError::PermissionDenied)?;
        log::debug!("InvokeTask: staged task: {:?}", staged_task);
        self.enqueue_to_db(StagedTask::get_queue_key().as_bytes(), &staged_task)?;

        let ts: TaskState = task.into();
        self.write_to_db(&ts)?;

        function_usage.use_numbers = function_current_use_numbers + 1;
        self.write_to_db(&function_usage)?;
        Ok(InvokeTaskResponse)
    }

    // access_control:
    // 1) user_id == task.creator
    // 2) user_role == admin
    fn cancel_task(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> TeaclaveServiceResponseResult<CancelTaskResponse> {
        let user_id = get_request_user_id(&request)?;
        let role = get_request_role(&request)?;
        let request = request.message;

        let ts: TaskState = self
            .read_from_db(&request.task_id)
            .map_err(|_| ManagementServiceError::InvalidTaskId)?;

        match role {
            UserRole::PlatformAdmin => {}
            _ => {
                ensure!(
                    ts.has_creator(&user_id),
                    ManagementServiceError::PermissionDenied
                );
            }
        }

        match ts.status {
            // need scheduler to cancel the task
            TaskStatus::Staged | TaskStatus::Running => {
                self.enqueue_to_db(CANCEL_QUEUE_KEY.as_bytes(), &ts)?;
            }
            _ => {
                // early cancelation
                // race will not affect correctness/privacy
                let mut task: Task<Cancel> = ts.try_into().map_err(|e| {
                    log::warn!("Cancel state error: {:?}", e);
                    ManagementServiceError::TaskCancelError(
                        "task has already been canceled".to_string(),
                    )
                })?;

                log::debug!("Canceled Task: {:?}", task);

                task.update_result(TaskResult::Err(TaskFailure {
                    reason: "Task canceled".to_string(),
                }))
                .map_err(|_| {
                    ManagementServiceError::TaskCancelError("cannot update result".to_string())
                })?;
                let ts: TaskState = task.into();
                self.write_to_db(&ts)?;

                log::warn!("Canceled Task: writtenback");
            }
        }

        Ok(CancelTaskResponse)
    }
}

impl TeaclaveManagementService {
    pub(crate) fn new(storage_service_endpoint: Endpoint) -> anyhow::Result<Self> {
        let mut i = 0;
        let channel = loop {
            match storage_service_endpoint.connect() {
                Ok(channel) => break channel,
                Err(_) => {
                    anyhow::ensure!(i < 10, "failed to connect to storage service");
                    log::warn!("Failed to connect to storage service, retry {}", i);
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

    fn write_to_db(&self, item: &impl Storable) -> Result<(), ManagementServiceError> {
        let k = item.key();
        let v = item.to_vec()?;
        let put_request = PutRequest::new(k.as_slice(), v.as_slice());
        let _put_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("cannot lock storage client"))?
            .put(put_request)
            .map_err(|e| ManagementServiceError::Service(e.into()))?;
        Ok(())
    }

    fn read_from_db<T: Storable>(&self, key: &ExternalID) -> Result<T, ManagementServiceError> {
        ensure!(
            T::match_prefix(&key.prefix),
            anyhow!("key prefix doesn't match")
        );

        let request = GetRequest::new(key.to_bytes());
        let response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("cannot lock storage client"))?
            .get(request)
            .map_err(|e| ManagementServiceError::Service(e.into()))?;
        T::from_slice(response.value.as_slice()).map_err(ManagementServiceError::Service)
    }

    fn get_keys_by_prefix_from_db(
        &self,
        prefix: impl Into<Vec<u8>>,
    ) -> Result<Vec<String>, ManagementServiceError> {
        let request = GetKeysByPrefixRequest::new(prefix.into());
        let response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("cannot lock storage client"))?
            .get_keys_by_prefix(request)
            .map_err(|e| ManagementServiceError::Service(e.into()))?;
        Ok(response
            .keys
            .into_iter()
            .map(String::from_utf8)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| anyhow!("cannot convert keys"))?)
    }

    fn delete_from_db(&self, key: &ExternalID) -> Result<(), ManagementServiceError> {
        let request = DeleteRequest::new(key.to_bytes());
        self.storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("cannot lock storage client"))?
            .delete(request)
            .map_err(|e| ManagementServiceError::Service(e.into()))?;
        Ok(())
    }

    fn enqueue_to_db(
        &self,
        key: &[u8],
        item: &impl Storable,
    ) -> Result<(), ManagementServiceError> {
        let value = item.to_vec()?;
        let enqueue_request = EnqueueRequest::new(key, value);
        let _enqueue_response = self
            .storage_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("cannot lock storage client"))?
            .enqueue(enqueue_request)
            .map_err(|e| ManagementServiceError::Service(e.into()))?;
        Ok(())
    }

    #[cfg(test_mode)]
    fn add_mock_data(&self) -> anyhow::Result<()> {
        let mut output_file = create_fusion_data(vec!["mock_user1", "frontend_user"])?;
        output_file.uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001")?;
        output_file.cmac = Some(FileAuthTag::mock());
        self.write_to_db(&output_file)?;

        let mut output_file = create_fusion_data(vec!["mock_user2", "mock_user3"])?;
        output_file.uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000002")?;
        output_file.cmac = Some(FileAuthTag::mock());
        self.write_to_db(&output_file)?;

        let mut input_file = TeaclaveInputFile::from_output(output_file)?;
        input_file.uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000002")?;
        self.write_to_db(&input_file)?;

        let function_input = FunctionInput::new("input", "input_desc", false);
        let function_output = FunctionOutput::new("output", "output_desc", false);
        let function_input2 = FunctionInput::new("input2", "input_desc", false);
        let function_output2 = FunctionOutput::new("output2", "output_desc", false);
        let function_arg1 = FunctionArgument::new("arg1", "", true);
        let function_arg2 = FunctionArgument::new("arg2", "", true);

        let function_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let function = FunctionBuilder::new()
            .id(function_id)
            .name("mock-func-1")
            .description("mock-desc")
            .payload(b"mock-payload".to_vec())
            .public(true)
            .arguments(vec![function_arg1, function_arg2])
            .inputs(vec![function_input, function_input2])
            .outputs(vec![function_output, function_output2])
            .owner("teaclave".to_string())
            .build();

        let function_usage = FunctionUsage {
            function_id,
            use_numbers: 0,
        };

        self.write_to_db(&function)?;
        self.write_to_db(&function_usage)?;

        let function_output = FunctionOutput::new("output", "output_desc", false);
        let function_arg1 = FunctionArgument::new("arg1", "", true);
        let function_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

        let function = FunctionBuilder::new()
            .id(function_id)
            .name("mock-func-2")
            .description("mock-desc")
            .payload(b"mock-payload".to_vec())
            .public(true)
            .arguments(vec![function_arg1.clone()])
            .outputs(vec![function_output])
            .owner("teaclave".to_string())
            .build();

        let function_usage = FunctionUsage {
            function_id,
            use_numbers: 0,
        };

        self.write_to_db(&function)?;
        self.write_to_db(&function_usage)?;

        let function_id = Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap();
        let function = FunctionBuilder::new()
            .id(function_id)
            .name("mock-func-3")
            .description("Private mock function")
            .payload(b"mock-payload".to_vec())
            .public(false)
            .arguments(vec![function_arg1])
            .owner("mock_user".to_string())
            .user_allowlist(vec!["mock_user".to_string(), "mock_user1".to_string()])
            .build();

        let function_usage = FunctionUsage {
            function_id,
            use_numbers: 0,
        };

        self.write_to_db(&function)?;
        self.write_to_db(&function_usage)?;

        Ok(())
    }
}

fn get_request_user_id<T>(request: &Request<T>) -> Result<UserID, ManagementServiceError> {
    let user_id = request
        .metadata()
        .get("id")
        .ok_or(ManagementServiceError::MissingUserId)?;
    Ok(user_id.to_string().into())
}

fn get_request_role<T>(request: &Request<T>) -> Result<UserRole, ManagementServiceError> {
    let role = request
        .metadata()
        .get("role")
        .ok_or(ManagementServiceError::MissingUserRole)?;
    Ok(UserRole::from_str(role))
}

fn create_fusion_data(owners: impl Into<OwnerList>) -> anyhow::Result<TeaclaveOutputFile> {
    let uuid = Uuid::new_v4();
    let url = format!("fusion:///TEACLAVE_FUSION_BASE/{}.fusion", uuid);
    let url = Url::parse(&url).map_err(|_| anyhow!("invalid url"))?;
    let crypto_info = FileCrypto::default();

    Ok(TeaclaveOutputFile::new(url, crypto_info, owners))
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
        let function_input = FunctionInput::new("input", "input_desc", false);
        let function_output = FunctionOutput::new("output", "output_desc", false);
        let function_arg = FunctionArgument::new("arg", "", true);
        let function = FunctionBuilder::new()
            .id(Uuid::new_v4())
            .name("mock_function")
            .description("mock function")
            .payload(b"python script".to_vec())
            .arguments(vec![function_arg])
            .inputs(vec![function_input])
            .outputs(vec![function_output])
            .public(true)
            .owner("mock_user")
            .build();
        assert!(Function::match_prefix(&function.key_string()));
        let value = function.to_vec().unwrap();
        let deserialized_function = Function::from_slice(&value).unwrap();
        debug!("function: {:?}", deserialized_function);
    }

    pub fn check_function_quota() {
        let function = FunctionBuilder::new().build();
        assert_eq!(function.usage_quota, None);

        let function = FunctionBuilder::new().usage_quota(Some(-5)).build();
        assert_eq!(function.usage_quota, None);

        let function = FunctionBuilder::new().usage_quota(Some(5)).build();
        assert_eq!(function.usage_quota, Some(5));
    }

    pub fn handle_task() {
        let function_arg = FunctionArgument::new("arg", "", true);
        let function = FunctionBuilder::new()
            .id(Uuid::new_v4())
            .name("mock_function")
            .description("mock function")
            .payload(b"python script".to_vec())
            .arguments(vec![function_arg])
            .public(true)
            .owner("mock_user")
            .build();
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
        let function = FunctionBuilder::new()
            .id(Uuid::new_v4())
            .name("mock_function")
            .description("mock function")
            .payload(b"python script".to_vec())
            .public(true)
            .owner("mock_user")
            .build();

        let url = Url::parse("s3://bucket_id/path?token=mock_token").unwrap();
        let cmac = FileAuthTag::mock();
        let input_data = FunctionInputFile::new(url.clone(), cmac, FileCrypto::default());
        let output_data = FunctionOutputFile::new(url, FileCrypto::default());

        let staged_task = StagedTaskBuilder::new()
            .task_id(Uuid::new_v4())
            .executor(Executor::MesaPy)
            .function_payload(function.payload)
            .function_arguments(hashmap!("arg" => "data"))
            .input_data(hashmap!("input" => input_data))
            .output_data(hashmap!("output" => output_data))
            .build();

        let value = staged_task.to_vec().unwrap();
        let deserialized_data = StagedTask::from_slice(&value).unwrap();
        debug!("staged task: {:?}", deserialized_data);
    }
}
