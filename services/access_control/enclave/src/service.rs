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

use crate::acs::{AccessControlModule, EnforceRequest};
use std::prelude::v1::*;
use teaclave_proto::teaclave_access_control_service::{
    AuthorizeDataRequest, AuthorizeDataResponse, AuthorizeFunctionRequest,
    AuthorizeFunctionResponse, AuthorizeStagedTaskRequest, AuthorizeStagedTaskResponse,
    AuthorizeTaskRequest, AuthorizeTaskResponse, TeaclaveAccessControl,
};
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::{bail, teaclave_service};
use teaclave_types::{TeaclaveServiceResponseError, TeaclaveServiceResponseResult};
use thiserror::Error;

#[derive(Error, Debug)]
enum TeaclavAccessControlError {
    #[error("access control error")]
    AccessControlError,
}

impl From<TeaclavAccessControlError> for TeaclaveServiceResponseError {
    fn from(error: TeaclavAccessControlError) -> Self {
        TeaclaveServiceResponseError::RequestError(error.to_string())
    }
}

#[teaclave_service(teaclave_access_control_service, TeaclaveAccessControl)]
#[derive(Clone)]
pub(crate) struct TeaclaveAccessControlService {
    access_control_module: AccessControlModule,
}

impl TeaclaveAccessControlService {
    pub(crate) fn new() -> Self {
        TeaclaveAccessControlService {
            access_control_module: AccessControlModule::new(),
        }
    }
}

impl TeaclaveAccessControl for TeaclaveAccessControlService {
    fn authorize_data(
        &self,
        request: Request<AuthorizeDataRequest>,
    ) -> TeaclaveServiceResponseResult<AuthorizeDataResponse> {
        let request = request.message;
        let request =
            EnforceRequest::UserAccessData(request.subject_user_id, request.object_data_id);
        match self.access_control_module.enforce_request(request) {
            Ok(accept) => {
                let response = AuthorizeDataResponse::new(accept);
                Ok(response)
            }
            Err(_) => Err(TeaclavAccessControlError::AccessControlError.into()),
        }
    }

    fn authorize_function(
        &self,
        request: Request<AuthorizeFunctionRequest>,
    ) -> TeaclaveServiceResponseResult<AuthorizeFunctionResponse> {
        let request = request.message;
        let request =
            EnforceRequest::UserAccessFunction(request.subject_user_id, request.object_function_id);
        match self.access_control_module.enforce_request(request) {
            Ok(accept) => {
                let response = AuthorizeFunctionResponse::new(accept);
                Ok(response)
            }
            Err(_) => Err(TeaclavAccessControlError::AccessControlError.into()),
        }
    }

    fn authorize_task(
        &self,
        request: Request<AuthorizeTaskRequest>,
    ) -> TeaclaveServiceResponseResult<AuthorizeTaskResponse> {
        let request = request.message;
        let request =
            EnforceRequest::UserAccessTask(request.subject_user_id, request.object_task_id);
        match self.access_control_module.enforce_request(request) {
            Ok(accept) => {
                let response = AuthorizeTaskResponse::new(accept);
                Ok(response)
            }
            Err(_) => Err(TeaclavAccessControlError::AccessControlError.into()),
        }
    }

    fn authorize_staged_task(
        &self,
        request: Request<AuthorizeStagedTaskRequest>,
    ) -> TeaclaveServiceResponseResult<AuthorizeStagedTaskResponse> {
        let request = request.message;
        let enforce_access_function_request = EnforceRequest::TaskAccessFunction(
            request.subject_task_id.clone(),
            request.object_function_id,
        );
        match self
            .access_control_module
            .enforce_request(enforce_access_function_request)
        {
            Ok(accept) => {
                if !accept {
                    return Ok(AuthorizeStagedTaskResponse::new(false));
                }
            }
            Err(_) => bail!(TeaclavAccessControlError::AccessControlError),
        }
        for object_data_id in request.object_input_data_id_list.iter() {
            let enforce_access_data_request = EnforceRequest::TaskAccessData(
                request.subject_task_id.clone(),
                object_data_id.to_string(),
            );
            match self
                .access_control_module
                .enforce_request(enforce_access_data_request)
            {
                Ok(accept) => {
                    if !accept {
                        return Ok(AuthorizeStagedTaskResponse::new(false));
                    }
                }
                Err(_) => bail!(TeaclavAccessControlError::AccessControlError),
            }
        }
        for object_data_id in request.object_output_data_id_list.iter() {
            let enforce_access_data_request = EnforceRequest::TaskAccessData(
                request.subject_task_id.clone(),
                object_data_id.to_string(),
            );
            match self
                .access_control_module
                .enforce_request(enforce_access_data_request)
            {
                Ok(accept) => {
                    if !accept {
                        return Ok(AuthorizeStagedTaskResponse::new(false));
                    }
                }
                Err(_) => bail!(TeaclavAccessControlError::AccessControlError),
            }
        }
        Ok(AuthorizeStagedTaskResponse { accept: true })
    }
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_rpc::IntoRequest;

    pub fn user_access_data() {
        let service = TeaclaveAccessControlService::new();
        let request = AuthorizeDataRequest::new("mock_user_a", "mock_data").into_request();
        let response = service.authorize_data(request);
        assert!(response.is_ok());
        assert!(response.unwrap().accept);

        let request = AuthorizeDataRequest::new("mock_user_b", "mock_data").into_request();
        let response = service.authorize_data(request);
        assert!(response.is_ok());
        assert!(response.unwrap().accept);

        let request = AuthorizeDataRequest::new("mock_user_c", "mock_data").into_request();
        let response = service.authorize_data(request);
        assert!(response.is_ok());
        assert!(response.unwrap().accept);

        let request = AuthorizeDataRequest::new("mock_user_d", "mock_data").into_request();
        let response = service.authorize_data(request);
        assert!(response.is_ok());
        assert!(!response.unwrap().accept);

        let request = AuthorizeDataRequest::new("mock_user_a", "mock_data_b").into_request();
        let response = service.authorize_data(request);
        assert!(response.is_ok());
        assert!(!response.unwrap().accept);
    }

    pub fn user_access_function() {
        let service = TeaclaveAccessControlService::new();
        let request =
            AuthorizeFunctionRequest::new("mock_public_function_owner", "mock_public_function")
                .into_request();
        let response = service.authorize_function(request);
        assert!(response.is_ok());
        assert!(response.unwrap().accept);

        let request =
            AuthorizeFunctionRequest::new("mock_private_function_owner", "mock_private_function")
                .into_request();
        let response = service.authorize_function(request);
        assert!(response.is_ok());
        assert!(response.unwrap().accept);

        let request =
            AuthorizeFunctionRequest::new("mock_private_function_owner", "mock_public_function")
                .into_request();
        let response = service.authorize_function(request);
        assert!(response.is_ok());
        assert!(response.unwrap().accept);

        let request =
            AuthorizeFunctionRequest::new("mock_public_function_owner", "mock_private_function")
                .into_request();
        let response = service.authorize_function(request);
        assert!(response.is_ok());
        assert!(!response.unwrap().accept);
    }

    pub fn user_access_task() {
        let service = TeaclaveAccessControlService::new();
        let request = AuthorizeTaskRequest::new("mock_participant_a", "mock_task").into_request();
        let response = service.authorize_task(request);
        assert!(response.is_ok());
        assert!(response.unwrap().accept);

        let request = AuthorizeTaskRequest::new("mock_participant_b", "mock_task").into_request();
        let response = service.authorize_task(request);
        assert!(response.is_ok());
        assert!(response.unwrap().accept);

        let request = AuthorizeTaskRequest::new("mock_participant_c", "mock_task").into_request();
        let response = service.authorize_task(request);
        assert!(response.is_ok());
        assert!(!response.unwrap().accept);
    }

    pub fn task_access_function() {
        let service = TeaclaveAccessControlService::new();
        let mut request = get_correct_authorized_stage_task_req();
        request.object_function_id = "mock_staged_allowed_private_function".to_string();
        let response = service.authorize_staged_task(request.into_request());
        assert!(response.is_ok());
        assert!(response.unwrap().accept);

        let mut request = get_correct_authorized_stage_task_req();
        request.object_function_id = "mock_staged_public_function".to_string();
        let response = service.authorize_staged_task(request.into_request());
        assert!(response.is_ok());
        assert!(response.unwrap().accept);

        let mut request = get_correct_authorized_stage_task_req();
        request.object_function_id = "mock_staged_disallowed_private_function".to_string();
        let response = service.authorize_staged_task(request.into_request());
        assert!(response.is_ok());
        assert!(!response.unwrap().accept);
    }

    fn get_correct_authorized_stage_task_req() -> AuthorizeStagedTaskRequest {
        AuthorizeStagedTaskRequest {
            subject_task_id: "mock_staged_task".to_string(),
            object_function_id: "mock_staged_allowed_private_function".to_string(),
            object_input_data_id_list: vec![
                "mock_staged_allowed_data1".to_string(),
                "mock_staged_allowed_data2".to_string(),
                "mock_staged_allowed_data3".to_string(),
            ],
            object_output_data_id_list: vec![
                "mock_staged_allowed_data1".to_string(),
                "mock_staged_allowed_data2".to_string(),
                "mock_staged_allowed_data3".to_string(),
            ],
        }
    }
    pub fn task_access_data() {
        let service = TeaclaveAccessControlService::new();
        let request = get_correct_authorized_stage_task_req().into_request();
        let response = service.authorize_staged_task(request);
        assert!(response.is_ok());
        assert!(response.unwrap().accept);

        let mut request = get_correct_authorized_stage_task_req();
        request
            .object_input_data_id_list
            .push("mock_staged_disallowed_data1".to_string());
        let response = service.authorize_staged_task(request.into_request());
        assert!(response.is_ok());
        assert!(!response.unwrap().accept);

        let mut request = get_correct_authorized_stage_task_req();
        request
            .object_input_data_id_list
            .push("mock_staged_disallowed_data2".to_string());
        let response = service.authorize_staged_task(request.into_request());
        assert!(response.is_ok());
        assert!(!response.unwrap().accept);

        let mut request = get_correct_authorized_stage_task_req();
        request
            .object_output_data_id_list
            .push("mock_staged_disallowed_data1".to_string());
        let response = service.authorize_staged_task(request.into_request());
        assert!(response.is_ok());
        assert!(!response.unwrap().accept);

        let mut request = get_correct_authorized_stage_task_req();
        request
            .object_output_data_id_list
            .push("mock_staged_disallowed_data2".to_string());
        let response = service.authorize_staged_task(request.into_request());
        assert!(response.is_ok());
        assert!(!response.unwrap().accept);
    }
}
