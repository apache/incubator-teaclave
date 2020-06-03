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

use anyhow::Result;
use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};
use thiserror::Error;

use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationInternalClient, UserAuthenticateRequest,
};
use teaclave_proto::teaclave_common::UserCredential;
use teaclave_proto::teaclave_frontend_service::{
    ApproveTaskRequest, ApproveTaskResponse, AssignDataRequest, AssignDataResponse,
    CreateTaskRequest, CreateTaskResponse, GetFunctionRequest, GetFunctionResponse,
    GetInputFileRequest, GetInputFileResponse, GetOutputFileRequest, GetOutputFileResponse,
    GetTaskRequest, GetTaskResponse, InvokeTaskRequest, InvokeTaskResponse,
    RegisterFunctionRequest, RegisterFunctionResponse, RegisterFusionOutputRequest,
    RegisterFusionOutputResponse, RegisterInputFileRequest, RegisterInputFileResponse,
    RegisterInputFromOutputRequest, RegisterInputFromOutputResponse, RegisterOutputFileRequest,
    RegisterOutputFileResponse, TeaclaveFrontend, UpdateInputFileRequest, UpdateInputFileResponse,
    UpdateOutputFileRequest, UpdateOutputFileResponse,
};
use teaclave_proto::teaclave_management_service::TeaclaveManagementClient;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::{bail, teaclave_service};
use teaclave_types::{TeaclaveServiceResponseError, TeaclaveServiceResponseResult};

#[derive(Error, Debug)]
enum TeaclaveFrontendError {
    #[error("authentication error")]
    AuthenticationError,
    #[error("lock error")]
    LockError,
}

impl From<TeaclaveFrontendError> for TeaclaveServiceResponseError {
    fn from(error: TeaclaveFrontendError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

#[teaclave_service(teaclave_frontend_service, TeaclaveFrontend, TeaclaveFrontendError)]
#[derive(Clone)]
pub(crate) struct TeaclaveFrontendService {
    authentication_client: Arc<Mutex<TeaclaveAuthenticationInternalClient>>,
    management_client: Arc<Mutex<TeaclaveManagementClient>>,
}

macro_rules! authentication_and_forward_to_management {
    ($service: ident, $request: ident, $func: ident) => {{
        match $service.authenticate(&$request) {
            Ok(true) => (),
            _ => bail!(TeaclaveFrontendError::AuthenticationError),
        }

        let client = $service.management_client.clone();
        let mut client = client
            .lock()
            .map_err(|_| TeaclaveFrontendError::LockError)?;
        client.metadata_mut().clear();
        client.metadata_mut().extend($request.metadata);

        let response = client.$func($request.message);

        client.metadata_mut().clear();
        let response = response?;
        Ok(response)
    }};
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
                    log::debug!("Failed to connect to authentication service, retry {}", i);
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
                    log::debug!("Failed to connect to management service, retry {}", i);
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
        authentication_and_forward_to_management!(self, request, register_input_file)
    }

    fn update_input_file(
        &self,
        request: Request<UpdateInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateInputFileResponse> {
        authentication_and_forward_to_management!(self, request, update_input_file)
    }

    fn register_output_file(
        &self,
        request: Request<RegisterOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterOutputFileResponse> {
        authentication_and_forward_to_management!(self, request, register_output_file)
    }

    fn update_output_file(
        &self,
        request: Request<UpdateOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateOutputFileResponse> {
        authentication_and_forward_to_management!(self, request, update_output_file)
    }

    fn register_fusion_output(
        &self,
        request: Request<RegisterFusionOutputRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFusionOutputResponse> {
        authentication_and_forward_to_management!(self, request, register_fusion_output)
    }

    fn register_input_from_output(
        &self,
        request: Request<RegisterInputFromOutputRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterInputFromOutputResponse> {
        authentication_and_forward_to_management!(self, request, register_input_from_output)
    }
    fn get_output_file(
        &self,
        request: Request<GetOutputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetOutputFileResponse> {
        authentication_and_forward_to_management!(self, request, get_output_file)
    }

    fn get_input_file(
        &self,
        request: Request<GetInputFileRequest>,
    ) -> TeaclaveServiceResponseResult<GetInputFileResponse> {
        authentication_and_forward_to_management!(self, request, get_input_file)
    }

    fn register_function(
        &self,
        request: Request<RegisterFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<RegisterFunctionResponse> {
        authentication_and_forward_to_management!(self, request, register_function)
    }

    fn get_function(
        &self,
        request: Request<GetFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<GetFunctionResponse> {
        authentication_and_forward_to_management!(self, request, get_function)
    }

    fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> TeaclaveServiceResponseResult<CreateTaskResponse> {
        authentication_and_forward_to_management!(self, request, create_task)
    }

    fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> TeaclaveServiceResponseResult<GetTaskResponse> {
        authentication_and_forward_to_management!(self, request, get_task)
    }

    fn assign_data(
        &self,
        request: Request<AssignDataRequest>,
    ) -> TeaclaveServiceResponseResult<AssignDataResponse> {
        authentication_and_forward_to_management!(self, request, assign_data)
    }

    fn approve_task(
        &self,
        request: Request<ApproveTaskRequest>,
    ) -> TeaclaveServiceResponseResult<ApproveTaskResponse> {
        authentication_and_forward_to_management!(self, request, approve_task)
    }

    fn invoke_task(
        &self,
        request: Request<InvokeTaskRequest>,
    ) -> TeaclaveServiceResponseResult<InvokeTaskResponse> {
        authentication_and_forward_to_management!(self, request, invoke_task)
    }
}

impl TeaclaveFrontendService {
    fn authenticate<T>(&self, request: &Request<T>) -> anyhow::Result<bool> {
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
        let auth_response = self
            .authentication_client
            .clone()
            .lock()
            .map_err(|_| anyhow!("Cannot lock authentication client"))?
            .user_authenticate(auth_request);
        Ok(auth_response?.accept)
    }
}
