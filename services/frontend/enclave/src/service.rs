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
use teaclave_proto::teaclave_access_control_service::{
    AuthorizeApiRequest, TeaclaveAccessControlClient,
};
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
use teaclave_types::{Entry, EntryBuilder, TeaclaveServiceResponseResult, UserAuthClaims};
use tokio::sync::Mutex;

macro_rules! authentication_and_forward_to_management {
    ($service: ident, $request: ident, $func: ident) => {{
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
                if $service
                    .check_api_privilege(
                        claims.get_role().to_string().split('-').next().unwrap(),
                        stringify!($func),
                    )
                    .await
                {
                    claims
                } else {
                    log::debug!(
                        "User is not authorized to access func: {}",
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
                    "User is not authenticated to access func: {}",
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

#[derive(Clone)]
pub(crate) struct TeaclaveFrontendService {
    authentication_client: Arc<Mutex<TeaclaveAuthenticationInternalClient<Channel>>>,
    management_client: Arc<Mutex<TeaclaveManagementClient<Channel>>>,
    access_control_client: Arc<Mutex<TeaclaveAccessControlClient<Channel>>>,
    audit_log_buffer: Arc<Mutex<Vec<Entry>>>,
}

impl TeaclaveFrontendService {
    pub(crate) async fn new(
        authentication_client: Arc<Mutex<TeaclaveAuthenticationInternalClient<Channel>>>,
        management_client: Arc<Mutex<TeaclaveManagementClient<Channel>>>,
        access_control_client: Arc<Mutex<TeaclaveAccessControlClient<Channel>>>,
        audit_log_buffer: Arc<Mutex<Vec<Entry>>>,
    ) -> Result<Self> {
        Ok(Self {
            authentication_client,
            management_client,
            access_control_client,
            audit_log_buffer,
        })
    }

    pub async fn push_log(&self, entry: Entry) {
        let mut buffer_lock = self.audit_log_buffer.lock().await;
        buffer_lock.push(entry);
    }

    async fn check_api_privilege(&self, user_role: &str, api: &str) -> bool {
        let request = AuthorizeApiRequest {
            user_role: user_role.to_owned(),
            api: api.to_owned(),
        };

        let mut acs_client = self.access_control_client.lock().await;
        let result = acs_client.authorize_api(request).await;
        result.map(|r| r.into_inner().accept).unwrap_or(false)
    }
}

#[teaclave_rpc::async_trait]
impl TeaclaveFrontend for TeaclaveFrontendService {
    async fn register_input_file(
        &self,
        request: Request<RegisterInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterInputFileResponse> {
        authentication_and_forward_to_management!(self, request, register_input_file)
    }

    async fn update_input_file(
        &self,
        request: Request<UpdateInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateInputFileResponse> {
        authentication_and_forward_to_management!(self, request, update_input_file)
    }

    async fn register_output_file(
        &self,
        request: Request<RegisterOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterOutputFileResponse> {
        authentication_and_forward_to_management!(self, request, register_output_file)
    }

    async fn update_output_file(
        &self,
        request: Request<UpdateOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateOutputFileResponse> {
        authentication_and_forward_to_management!(self, request, update_output_file)
    }

    async fn register_fusion_output(
        &self,
        request: Request<RegisterFusionOutputRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFusionOutputResponse> {
        authentication_and_forward_to_management!(self, request, register_fusion_output)
    }

    async fn register_input_from_output(
        &self,
        request: Request<RegisterInputFromOutputRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterInputFromOutputResponse> {
        authentication_and_forward_to_management!(self, request, register_input_from_output)
    }

    async fn get_output_file(
        &self,
        request: Request<GetOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetOutputFileResponse> {
        authentication_and_forward_to_management!(self, request, get_output_file)
    }

    async fn get_input_file(
        &self,
        request: Request<GetInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetInputFileResponse> {
        authentication_and_forward_to_management!(self, request, get_input_file)
    }

    async fn register_function(
        &self,
        request: Request<RegisterFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFunctionResponse> {
        authentication_and_forward_to_management!(self, request, register_function)
    }

    async fn update_function(
        &self,
        request: Request<UpdateFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateFunctionResponse> {
        authentication_and_forward_to_management!(self, request, update_function)
    }

    async fn get_function(
        &self,
        request: Request<GetFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<GetFunctionResponse> {
        authentication_and_forward_to_management!(self, request, get_function)
    }

    async fn get_function_usage_stats(
        &self,
        request: Request<GetFunctionUsageStatsRequest>,
    ) -> TeaclaveServiceResponseResult<GetFunctionUsageStatsResponse> {
        authentication_and_forward_to_management!(self, request, get_function_usage_stats)
    }

    async fn delete_function(
        &self,
        request: Request<DeleteFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(self, request, delete_function)
    }

    async fn disable_function(
        &self,
        request: Request<DisableFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(self, request, disable_function)
    }

    async fn list_functions(
        &self,
        request: Request<ListFunctionsRequest>,
    ) -> TeaclaveServiceResponseResult<ListFunctionsResponse> {
        authentication_and_forward_to_management!(self, request, list_functions)
    }

    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> TeaclaveServiceResponseResult<CreateTaskResponse> {
        authentication_and_forward_to_management!(self, request, create_task)
    }

    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> TeaclaveServiceResponseResult<GetTaskResponse> {
        authentication_and_forward_to_management!(self, request, get_task)
    }

    async fn assign_data(
        &self,
        request: Request<AssignDataRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(self, request, assign_data)
    }

    async fn approve_task(
        &self,
        request: Request<ApproveTaskRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(self, request, approve_task)
    }

    async fn invoke_task(
        &self,
        request: Request<InvokeTaskRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(self, request, invoke_task)
    }

    async fn cancel_task(
        &self,
        request: Request<CancelTaskRequest>,
    ) -> TeaclaveServiceResponseResult<()> {
        authentication_and_forward_to_management!(self, request, cancel_task)
    }

    async fn query_audit_logs(
        &self,
        request: Request<QueryAuditLogsRequest>,
    ) -> TeaclaveServiceResponseResult<QueryAuditLogsResponse> {
        authentication_and_forward_to_management!(self, request, query_audit_logs)
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
