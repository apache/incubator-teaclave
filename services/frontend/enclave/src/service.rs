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

use crate::error::AuthenticationError;
use crate::error::FrontendServiceError;

use anyhow::Result;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::Arc;
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationInternalClient, UserAuthenticateRequest,
};
use teaclave_proto::teaclave_common::UserCredential;
use teaclave_proto::teaclave_frontend_service::{
    ApproveTaskRequest, AssignDataRequest, CancelTaskRequest, CreateTaskRequest,
    CreateTaskResponse, DeleteFunctionRequest, DisableFunctionRequest, GetFunctionRequest,
    GetFunctionResponse, GetFunctionUsageStatsRequest, GetFunctionUsageStatsResponse,
    GetInputFileRequest, GetInputFileResponse, GetOutputFileRequest, GetOutputFileResponse,
    GetTaskRequest, GetTaskResponse, InvokeTaskRequest, ListFunctionsRequest,
    ListFunctionsResponse, QueryAuditLogsRequest, QueryAuditLogsResponse, RegisterFunctionRequest,
    RegisterFunctionResponse, RegisterFusionOutputRequest, RegisterFusionOutputResponse,
    RegisterInputFileRequest, RegisterInputFileResponse, RegisterInputFromOutputRequest,
    RegisterInputFromOutputResponse, RegisterOutputFileRequest, RegisterOutputFileResponse,
    TeaclaveFrontend, UpdateFunctionRequest, UpdateFunctionResponse, UpdateInputFileRequest,
    UpdateInputFileResponse, UpdateOutputFileRequest, UpdateOutputFileResponse,
};
use teaclave_proto::teaclave_management_service::TeaclaveManagementClient;
use teaclave_rpc::transport::Channel;
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::bail;
use teaclave_types::{
    Entry, EntryBuilder, TeaclaveServiceResponseResult, UserAuthClaims, UserRole,
};
use tokio::sync::Mutex;

macro_rules! authentication_and_forward_to_management {
    ($service: ident, $request: ident, $func: ident, $endpoint: expr) => {{
        let function_name = stringify!($func).to_owned();
        let ip_option = $request.remote_addr().map(|s| s.ip());
        let ip = match ip_option {
            Some(IpAddr::V4(ip_v4)) => ip_v4.to_ipv6_compatible(),
            Some(IpAddr::V6(ip_v6)) => ip_v6,
            None => Ipv6Addr::UNSPECIFIED,
        };

        let builder = EntryBuilder::new().ip(ip);

        let claims = match $service.authenticate(&$request).await {
            Ok(claims) => {
                if authorize(&claims, $endpoint) {
                    claims
                } else {
                    log::debug!(
                        "User is not authorized to access endpoint: {}, func: {}",
                        stringify!($endpoint),
                        stringify!($func)
                    );

                    let entry = builder
                        .message(String::from("authenticate to ") + &function_name)
                        .result(false)
                        .build();
                    $service.push_log(entry).await;

                    bail!(FrontendServiceError::PermissionDenied);
                }
            }
            Err(e) => {
                log::debug!(
                    "User is not authenticated to access endpoint: {}, func: {}",
                    stringify!($endpoint),
                    stringify!($func)
                );

                let entry = builder
                    .message(
                        String::from("authenticate to ") + &function_name + ": " + &e.to_string(),
                    )
                    .result(false)
                    .build();
                $service.push_log(entry).await;

                bail!(e);
            }
        };

        let user = claims.to_string();
        let builder = builder.user(user);

        let client = $service.management_client.clone();
        let mut client = client.lock().await;
        let meta = $request.metadata().clone();
        let message = $request.get_ref().to_owned();

        let mut request = Request::new(message);
        let metadata = request.metadata_mut();
        *metadata = meta;
        metadata.insert("role", claims.role.parse().unwrap());

        let response = match client.$func(request).await {
            Err(e) => {
                let entry = builder
                    .clone()
                    .message(function_name.clone() + ":" + &e.to_string())
                    .result(false)
                    .build();
                $service.push_log(entry).await;
                return Err(e);
            }
            Ok(r) => r,
        };

        let entry = builder.message(function_name).result(true).build();
        $service.push_log(entry).await;
        Ok(response)
    }};
}

// TODO: remove this structure as it is the same with RPC interface
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
    GetFunctionUsageStats,
    UpdateFunction,
    ListFunctions,
    DeleteFunction,
    DisableFunction,
    CreateTask,
    GetTask,
    AssignData,
    ApproveTask,
    InvokeTask,
    CancelTask,
    QueryAuditLogs,
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
        Endpoints::RegisterFunction
        | Endpoints::UpdateFunction
        | Endpoints::DeleteFunction
        | Endpoints::DisableFunction => role.is_function_owner(),
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
        Endpoints::GetFunction | Endpoints::ListFunctions | Endpoints::GetFunctionUsageStats => {
            role.is_function_owner() || role.is_data_owner()
        }
        Endpoints::QueryAuditLogs => false,
    }
}

#[derive(Clone)]
pub(crate) struct TeaclaveFrontendService {
    authentication_client: Arc<Mutex<TeaclaveAuthenticationInternalClient<Channel>>>,
    management_client: Arc<Mutex<TeaclaveManagementClient<Channel>>>,
    audit_log_buffer: Arc<Mutex<Vec<Entry>>>,
}

impl TeaclaveFrontendService {
    pub(crate) async fn new(
        authentication_client: Arc<Mutex<TeaclaveAuthenticationInternalClient<Channel>>>,
        management_client: Arc<Mutex<TeaclaveManagementClient<Channel>>>,
        audit_log_buffer: Arc<Mutex<Vec<Entry>>>,
    ) -> Result<Self> {
        Ok(Self {
            authentication_client,
            management_client,
            audit_log_buffer,
        })
    }

    pub async fn push_log(&self, entry: Entry) {
        let mut buffer_lock = self.audit_log_buffer.lock().await;
        buffer_lock.push(entry);
    }
}

#[teaclave_rpc::async_trait]
impl TeaclaveFrontend for TeaclaveFrontendService {
    async fn register_input_file(
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

    async fn update_input_file(
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

    async fn register_output_file(
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

    async fn update_output_file(
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

    async fn register_fusion_output(
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

    async fn register_input_from_output(
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

    async fn get_output_file(
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

    async fn get_input_file(
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

    async fn register_function(
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

    async fn update_function(
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

    async fn get_function(
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

    async fn get_function_usage_stats(
        &self,
        request: Request<GetFunctionUsageStatsRequest>,
    ) -> TeaclaveServiceResponseResult<GetFunctionUsageStatsResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            get_function_usage_stats,
            Endpoints::GetFunctionUsageStats
        )
    }

    async fn delete_function(
        &self,
        request: Request<DeleteFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(
            self,
            request,
            delete_function,
            Endpoints::DeleteFunction
        )
    }

    async fn disable_function(
        &self,
        request: Request<DisableFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(
            self,
            request,
            disable_function,
            Endpoints::DisableFunction
        )
    }

    async fn list_functions(
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

    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> TeaclaveServiceResponseResult<CreateTaskResponse> {
        authentication_and_forward_to_management!(self, request, create_task, Endpoints::CreateTask)
    }

    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> TeaclaveServiceResponseResult<GetTaskResponse> {
        authentication_and_forward_to_management!(self, request, get_task, Endpoints::GetTask)
    }

    async fn assign_data(
        &self,
        request: Request<AssignDataRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(self, request, assign_data, Endpoints::AssignData)
    }

    async fn approve_task(
        &self,
        request: Request<ApproveTaskRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(
            self,
            request,
            approve_task,
            Endpoints::ApproveTask
        )
    }

    async fn invoke_task(
        &self,
        request: Request<InvokeTaskRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(self, request, invoke_task, Endpoints::InvokeTask)
    }

    async fn cancel_task(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(self, request, cancel_task, Endpoints::CancelTask)
    }

    async fn query_audit_logs(
        &self,
        request: Request<QueryAuditLogsRequest>,
    ) -> TeaclaveServiceResponseResult<QueryAuditLogsResponse> {
        authentication_and_forward_to_management!(
            self,
            request,
            query_audit_logs,
            Endpoints::QueryAuditLogs
        )
    }
}

impl TeaclaveFrontendService {
    async fn authenticate<T>(
        &self,
        request: &Request<T>,
    ) -> Result<UserAuthClaims, FrontendServiceError> {
        let id = request
            .metadata()
            .get("id")
            .and_then(|x| x.to_str().ok())
            .ok_or(AuthenticationError::MissingUserId)?;
        let token = request
            .metadata()
            .get("token")
            .and_then(|x| x.to_str().ok())
            .ok_or(AuthenticationError::MissingToken)?;
        let credential = Some(UserCredential::new(id, token));
        let auth_request = UserAuthenticateRequest { credential };
        let claims = self
            .authentication_client
            .clone()
            .lock()
            .await
            .user_authenticate(auth_request)
            .await
            .map_err(|_| AuthenticationError::IncorrectCredential)?
            .into_inner()
            .claims
            .and_then(|x| x.try_into().ok())
            .ok_or(AuthenticationError::IncorrectCredential)?;

        Ok(claims)
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;

    pub fn test_authorize_platform_admin() {
        let claims = UserAuthClaims {
            role: "PlatformAdmin".to_string(),
            ..Default::default()
        };
        let result = authorize(&claims, Endpoints::GetFunction);
        assert!(result);
    }

    pub fn test_authorize_function_owner() {
        let claims = UserAuthClaims {
            role: "FunctionOwner".to_string(),
            ..Default::default()
        };
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
        let claims = UserAuthClaims {
            role: "DataOwnerManager-Attribute".to_string(),
            ..Default::default()
        };
        let result = authorize(&claims, Endpoints::GetFunction);
        assert!(result);
        let result = authorize(&claims, Endpoints::InvokeTask);
        assert!(result);

        let claims = UserAuthClaims {
            role: "DataOwner-Attribute".to_string(),
            ..Default::default()
        };
        let result = authorize(&claims, Endpoints::GetFunction);
        assert!(result);
        let result = authorize(&claims, Endpoints::InvokeTask);
        assert!(result);
    }
}
