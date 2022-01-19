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

use crate::error::TeaclaveFrontendError;

use anyhow::Result;
use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};

use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationInternalClient, UserAuthenticateRequest,
};
use teaclave_proto::teaclave_common::UserCredential;
use teaclave_proto::teaclave_frontend_service::{
    ApproveTaskRequest, ApproveTaskResponse, AssignDataRequest, AssignDataResponse,
    CancelTaskRequest, CancelTaskResponse, CreateTaskRequest, CreateTaskResponse,
    DeleteFunctionRequest, DeleteFunctionResponse, GetFunctionRequest, GetFunctionResponse,
    GetInputFileRequest, GetInputFileResponse, GetOutputFileRequest, GetOutputFileResponse,
    GetTaskRequest, GetTaskResponse, InvokeTaskRequest, InvokeTaskResponse, ListFunctionsRequest,
    ListFunctionsResponse, RegisterFunctionRequest, RegisterFunctionResponse,
    RegisterFusionOutputRequest, RegisterFusionOutputResponse, RegisterInputFileRequest,
    RegisterInputFileResponse, RegisterInputFromOutputRequest, RegisterInputFromOutputResponse,
    RegisterOutputFileRequest, RegisterOutputFileResponse, TeaclaveFrontend, UpdateFunctionRequest,
    UpdateFunctionResponse, UpdateInputFileRequest, UpdateInputFileResponse,
    UpdateOutputFileRequest, UpdateOutputFileResponse,
};
use teaclave_proto::teaclave_management_service::TeaclaveManagementClient;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::{bail, teaclave_service};
use teaclave_types::{TeaclaveServiceResponseResult, UserAuthClaims, UserRole};

#[teaclave_service(teaclave_frontend_service, TeaclaveFrontend, TeaclaveFrontendError)]
#[derive(Clone)]
pub(crate) struct TeaclaveFrontendService {
    authentication_client: Arc<Mutex<TeaclaveAuthenticationInternalClient>>,
    management_client: Arc<Mutex<TeaclaveManagementClient>>,
}

macro_rules! authentication_and_forward_to_management {
    ($service: ident, $request: ident, $func: ident, $endpoint: expr) => {{
        let claims = match $service.authenticate(&$request) {
            Ok(claims) => {
                if authorize(&claims, $endpoint) {
                    claims
                } else {
                    log::debug!(
                        "User is not authorized to access endpoint: {}, func: {}",
                        stringify!($endpoint),
                        stringify!($func)
                    );
                    bail!(TeaclaveFrontendError::AuthenticationError);
                }
            }
            _ => {
                log::debug!(
                    "User is not authenticated to access endpoint: {}, func: {}",
                    stringify!($endpoint),
                    stringify!($func)
                );
                bail!(TeaclaveFrontendError::AuthenticationError)
            }
        };

        let client = $service.management_client.clone();
        let mut client = client
            .lock()
            .map_err(|_| TeaclaveFrontendError::LockError)?;
        client.metadata_mut().clear();
        client.metadata_mut().extend($request.metadata);
        client
            .metadata_mut()
            .insert("role".to_string(), claims.role);

        let response = client.$func($request.message);

        client.metadata_mut().clear();
        let response = response?;
        Ok(response)
    }};
}

enum Endpoints {
    RegisterInputFile,
    RegisterOutputFile,
    UpdateInputFile,
    UpdateOutputFile,
    RegisterFusionOutput,
    RegisterInputFromOutput,
    GetOutputFile,
    GetInputFile,
    RegisterFunction,
    GetFunction,
    UpdateFunction,
    ListFunctions,
    DeleteFunction,
    CreateTask,
    GetTask,
    AssignData,
    ApproveTask,
    InvokeTask,
    CancelTask,
}

fn authorize(claims: &UserAuthClaims, request: Endpoints) -> bool {
    let role = claims.get_role();

    if role == UserRole::Invalid {
        return false;
    }
    if role == UserRole::PlatformAdmin {
        return true;
    }

    match request {
        Endpoints::RegisterFunction | Endpoints::UpdateFunction | Endpoints::DeleteFunction => {
            role.is_function_owner()
        }
        Endpoints::RegisterInputFile
        | Endpoints::RegisterOutputFile
        | Endpoints::UpdateInputFile
        | Endpoints::UpdateOutputFile
        | Endpoints::RegisterFusionOutput
        | Endpoints::RegisterInputFromOutput
        | Endpoints::GetOutputFile
        | Endpoints::GetInputFile
        | Endpoints::CreateTask
        | Endpoints::GetTask
        | Endpoints::AssignData
        | Endpoints::ApproveTask
        | Endpoints::InvokeTask
        | Endpoints::CancelTask => role.is_data_owner(),
        Endpoints::GetFunction | Endpoints::ListFunctions => {
            role.is_function_owner() || role.is_data_owner()
        }
    }
}

impl TeaclaveFrontendService {
    pub(crate) fn new(
        authentication_service_endpoint: Endpoint,
        management_service_endpoint: Endpoint,
    ) -> Result<Self> {
        let mut i = 0;
        let authentication_channel = loop {
            match authentication_service_endpoint.connect() {
                Ok(channel) => break channel,
                Err(_) => {
                    anyhow::ensure!(i < 10, "failed to connect to authentication service");
                    log::warn!("Failed to connect to authentication service, retry {}", i);
                    i += 1;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(3));
        };
        let authentication_client = Arc::new(Mutex::new(
            TeaclaveAuthenticationInternalClient::new(authentication_channel)?,
        ));

        let mut i = 0;
        let management_channel = loop {
            match management_service_endpoint.connect() {
                Ok(channel) => break channel,
                Err(_) => {
                    anyhow::ensure!(i < 10, "failed to connect to management service");
                    log::warn!("Failed to connect to management service, retry {}", i);
                    i += 1;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(3));
        };
        let management_client = Arc::new(Mutex::new(TeaclaveManagementClient::new(
            management_channel,
        )?));

        Ok(Self {
            authentication_client,
            management_client,
        })
    }
}

impl TeaclaveFrontend for TeaclaveFrontendService {
    fn register_input_file(
        &self,
        request: Request<RegisterInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterInputFileResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            register_input_file,
            Endpoints::RegisterInputFile
        )
    }

    fn update_input_file(
        &self,
        request: Request<UpdateInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateInputFileResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            update_input_file,
            Endpoints::UpdateInputFile
        )
    }

    fn register_output_file(
        &self,
        request: Request<RegisterOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterOutputFileResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            register_output_file,
            Endpoints::RegisterOutputFile
        )
    }

    fn update_output_file(
        &self,
        request: Request<UpdateOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateOutputFileResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            update_output_file,
            Endpoints::UpdateOutputFile
        )
    }

    fn register_fusion_output(
        &self,
        request: Request<RegisterFusionOutputRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFusionOutputResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            register_fusion_output,
            Endpoints::RegisterFusionOutput
        )
    }

    fn register_input_from_output(
        &self,
        request: Request<RegisterInputFromOutputRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterInputFromOutputResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            register_input_from_output,
            Endpoints::RegisterInputFromOutput
        )
    }
    fn get_output_file(
        &self,
        request: Request<GetOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetOutputFileResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            get_output_file,
            Endpoints::GetOutputFile
        )
    }

    fn get_input_file(
        &self,
        request: Request<GetInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetInputFileResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            get_input_file,
            Endpoints::GetInputFile
        )
    }

    fn register_function(
        &self,
        request: Request<RegisterFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFunctionResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            register_function,
            Endpoints::RegisterFunction
        )
    }

    fn update_function(
        &self,
        request: Request<UpdateFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateFunctionResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            update_function,
            Endpoints::UpdateFunction
        )
    }

    fn get_function(
        &self,
        request: Request<GetFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<GetFunctionResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            get_function,
            Endpoints::GetFunction
        )
    }

    fn delete_function(
        &self,
        request: Request<DeleteFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<DeleteFunctionResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            delete_function,
            Endpoints::DeleteFunction
        )
    }

    fn list_functions(
        &self,
        request: Request<ListFunctionsRequest>,
    ) -> TeaclaveServiceResponseResult<ListFunctionsResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            list_functions,
            Endpoints::ListFunctions
        )
    }

    fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> TeaclaveServiceResponseResult<CreateTaskResponse> {
        authentication_and_forward_to_management!(self, request, create_task, Endpoints::CreateTask)
    }

    fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> TeaclaveServiceResponseResult<GetTaskResponse> {
        authentication_and_forward_to_management!(self, request, get_task, Endpoints::GetTask)
    }

    fn assign_data(
        &self,
        request: Request<AssignDataRequest>,
    ) -> TeaclaveServiceResponseResult<AssignDataResponse> {
        authentication_and_forward_to_management!(self, request, assign_data, Endpoints::AssignData)
    }

    fn approve_task(
        &self,
        request: Request<ApproveTaskRequest>,
    ) -> TeaclaveServiceResponseResult<ApproveTaskResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            approve_task,
            Endpoints::ApproveTask
        )
    }

    fn invoke_task(
        &self,
        request: Request<InvokeTaskRequest>,
    ) -> TeaclaveServiceResponseResult<InvokeTaskResponse> {
        authentication_and_forward_to_management!(self, request, invoke_task, Endpoints::InvokeTask)
    }

    fn cancel_task(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> TeaclaveServiceResponseResult<CancelTaskResponse> {
        authentication_and_forward_to_management!(self, request, cancel_task, Endpoints::CancelTask)
    }
}

impl TeaclaveFrontendService {
    fn authenticate<T>(&self, request: &Request<T>) -> anyhow::Result<UserAuthClaims> {
        use anyhow::anyhow;
        let id = request
            .metadata
            .get("id")
            .ok_or_else(|| anyhow!("Missing credential"))?;
        let token = request
            .metadata
            .get("token")
            .ok_or_else(|| anyhow!("Missing credential"))?;
        let credential = UserCredential::new(id, token);
        let auth_request = UserAuthenticateRequest { credential };
        let claims = self
            .authentication_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("Cannot lock authentication client"))?
            .user_authenticate(auth_request)?
            .claims;

        Ok(claims)
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;

    pub fn test_authorize_platform_admin() {
        let mut claims = UserAuthClaims::default();
        claims.role = "PlatformAdmin".to_string();
        let result = authorize(&claims, Endpoints::GetFunction);
        assert!(result);
    }

    pub fn test_authorize_function_owner() {
        let mut claims = UserAuthClaims::default();
        claims.role = "FunctionOwner".to_string();
        let result = authorize(&claims, Endpoints::GetFunction);
        assert!(result);
        let result = authorize(&claims, Endpoints::RegisterFunction);
        assert!(result);
        let result = authorize(&claims, Endpoints::UpdateFunction);
        assert!(result);
        let result = authorize(&claims, Endpoints::InvokeTask);
        assert!(!result);
    }

    pub fn test_authorize_data_owner() {
        let mut claims = UserAuthClaims::default();
        claims.role = "DataOwnerManager-Attribute".to_string();
        let result = authorize(&claims, Endpoints::GetFunction);
        assert!(result);
        let result = authorize(&claims, Endpoints::InvokeTask);
        assert!(result);

        let mut claims = UserAuthClaims::default();
        claims.role = "DataOwner-Attribute".to_string();
        let result = authorize(&claims, Endpoints::GetFunction);
        assert!(result);
        let result = authorize(&claims, Endpoints::InvokeTask);
        assert!(result);
    }
}
