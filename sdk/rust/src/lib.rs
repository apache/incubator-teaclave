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
use std::collections::HashMap;
use std::convert::TryInto;
use teaclave_attestation::verifier;
use teaclave_proto::teaclave_authentication_service::TeaclaveAuthenticationApiClient;
use teaclave_proto::teaclave_authentication_service_proto as authentication_proto;
use teaclave_proto::teaclave_frontend_service::TeaclaveFrontendClient;
use teaclave_proto::teaclave_frontend_service_proto as frontend_proto;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::FileAuthTag;
use url::Url;

pub use teaclave_proto::teaclave_authentication_service::{
    UserLoginRequest, UserLoginResponse, UserRegisterRequest, UserRegisterResponse,
};
pub use teaclave_proto::teaclave_frontend_service::GetFunctionResponse as Function;
pub use teaclave_proto::teaclave_frontend_service::{
    ApproveTaskRequest, ApproveTaskResponse, AssignDataRequest, AssignDataResponse,
    CancelTaskRequest, CancelTaskResponse, CreateTaskRequest, CreateTaskResponse,
    GetFunctionRequest, GetFunctionResponse, GetTaskRequest, GetTaskResponse, InvokeTaskRequest,
    InvokeTaskResponse, RegisterFunctionRequest, RegisterFunctionRequestBuilder,
    RegisterFunctionResponse, RegisterInputFileRequest, RegisterInputFileResponse,
    RegisterOutputFileRequest, RegisterOutputFileResponse,
};
pub use teaclave_types::{
    EnclaveInfo, Executor, FileCrypto, FunctionArgument, FunctionInput, FunctionOutput, TaskResult,
};

pub mod bindings;

pub struct AuthenticationClient {
    api_client: TeaclaveAuthenticationApiClient,
}

pub struct AuthenticationService;

impl AuthenticationClient {
    pub fn new(api_client: TeaclaveAuthenticationApiClient) -> Self {
        Self { api_client }
    }

    pub fn set_credential(&mut self, id: &str, token: &str) {
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), id.to_string());
        metadata.insert("token".to_string(), token.to_string());
        self.api_client.set_metadata(metadata);
    }

    pub fn user_register_with_request(
        &mut self,
        request: UserRegisterRequest,
    ) -> Result<UserRegisterResponse> {
        let response = self.api_client.user_register(request)?;

        Ok(response)
    }

    pub fn user_register_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: authentication_proto::UserRegisterRequest =
            serde_json::from_str(serialized_request)?;
        let response: authentication_proto::UserRegisterResponse =
            self.user_register_with_request(request.try_into()?)?.into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn user_register(
        &mut self,
        user_id: &str,
        user_password: &str,
        role: &str,
        attribute: &str,
    ) -> Result<()> {
        let request = UserRegisterRequest::new(user_id, user_password, role, attribute);
        let _response = self.user_register_with_request(request)?;

        Ok(())
    }

    pub fn user_login_with_request(
        &mut self,
        request: UserLoginRequest,
    ) -> Result<UserLoginResponse> {
        let response = self.api_client.user_login(request)?;

        Ok(response)
    }

    pub fn user_login_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: authentication_proto::UserLoginRequest =
            serde_json::from_str(serialized_request)?;
        let response: authentication_proto::UserLoginResponse =
            self.user_login_with_request(request.try_into()?)?.into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn user_login(&mut self, user_id: &str, user_password: &str) -> Result<String> {
        let request = UserLoginRequest::new(user_id, user_password);
        let response = self.user_login_with_request(request)?;

        Ok(response.token)
    }
}

impl AuthenticationService {
    pub fn connect(
        url: &str,
        enclave_info: &EnclaveInfo,
        as_root_ca_cert: &[u8],
    ) -> Result<AuthenticationClient> {
        let enclave_attr = enclave_info
            .get_enclave_attr("teaclave_authentication_service")
            .expect("enclave attr");
        let config = SgxTrustedTlsClientConfig::new().attestation_report_verifier(
            vec![enclave_attr],
            as_root_ca_cert,
            verifier::universal_quote_verifier,
        );
        let channel = Endpoint::new(url).config(config).connect()?;
        let client = TeaclaveAuthenticationApiClient::new(channel)?;

        Ok(AuthenticationClient::new(client))
    }
}

#[repr(C)]
pub struct FrontendService;

impl FrontendService {
    pub fn connect(
        url: &str,
        enclave_info: &EnclaveInfo,
        as_root_ca_cert: &[u8],
    ) -> Result<FrontendClient> {
        let enclave_attr = enclave_info
            .get_enclave_attr("teaclave_frontend_service")
            .expect("enclave attr");
        let config = SgxTrustedTlsClientConfig::new().attestation_report_verifier(
            vec![enclave_attr],
            as_root_ca_cert,
            verifier::universal_quote_verifier,
        );
        let channel = Endpoint::new(url).config(config).connect()?;
        let client = TeaclaveFrontendClient::new(channel)?;

        Ok(FrontendClient::new(client))
    }
}

pub struct FrontendClient {
    api_client: TeaclaveFrontendClient,
}

impl FrontendClient {
    pub fn new(api_client: TeaclaveFrontendClient) -> Self {
        Self { api_client }
    }

    pub fn set_credential(&mut self, id: &str, token: &str) {
        let mut metadata = HashMap::new();
        metadata.insert("id".to_string(), id.to_string());
        metadata.insert("token".to_string(), token.to_string());
        self.api_client.set_metadata(metadata);
    }

    pub fn register_function_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: frontend_proto::RegisterFunctionRequest =
            serde_json::from_str(serialized_request)?;
        let response: frontend_proto::RegisterFunctionResponse = self
            .register_function_with_request(request.try_into()?)?
            .into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn register_function_with_request(
        &mut self,
        request: RegisterFunctionRequest,
    ) -> Result<RegisterFunctionResponse> {
        let response = self.api_client.register_function(request)?;

        Ok(response)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_function(
        &mut self,
        name: &str,
        description: &str,
        executor_type: &str,
        payload: Option<&[u8]>,
        arguments: Option<Vec<FunctionArgument>>,
        inputs: Option<Vec<FunctionInput>>,
        outputs: Option<Vec<FunctionOutput>>,
    ) -> Result<String> {
        let executor_type = executor_type.try_into()?;
        let mut builder = RegisterFunctionRequestBuilder::new()
            .name(name)
            .description(description)
            .executor_type(executor_type);

        if let Some(payload) = payload {
            builder = builder.payload(payload.into());
        }
        if let Some(arguments) = arguments {
            builder = builder.arguments(arguments);
        }
        if let Some(inputs) = inputs {
            builder = builder.inputs(inputs);
        }
        if let Some(outputs) = outputs {
            builder = builder.outputs(outputs);
        }

        let request = builder.build();
        let response = self.register_function_with_request(request)?;

        Ok(response.function_id.to_string())
    }

    pub fn get_function_with_request(
        &mut self,
        request: GetFunctionRequest,
    ) -> Result<GetFunctionResponse> {
        let response = self.api_client.get_function(request)?;

        Ok(response)
    }

    pub fn get_function_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: frontend_proto::GetFunctionRequest = serde_json::from_str(serialized_request)?;
        let response: frontend_proto::GetFunctionResponse =
            self.get_function_with_request(request.try_into()?)?.into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn get_function(&mut self, function_id: &str) -> Result<Function> {
        let function_id = function_id.try_into()?;
        let request = GetFunctionRequest::new(function_id);
        let response = self.get_function_with_request(request)?;

        Ok(response)
    }

    pub fn register_input_file_with_request(
        &mut self,
        request: RegisterInputFileRequest,
    ) -> Result<RegisterInputFileResponse> {
        let response = self.api_client.register_input_file(request)?;

        Ok(response)
    }

    pub fn register_input_file_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: frontend_proto::RegisterInputFileRequest =
            serde_json::from_str(serialized_request)?;
        let response: frontend_proto::RegisterInputFileResponse = self
            .register_input_file_with_request(request.try_into()?)?
            .into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn register_input_file(
        &mut self,
        url: &str,
        cmac: &[u8],
        file_crypto: FileCrypto,
    ) -> Result<String> {
        let url = Url::parse(url)?;
        let cmac = FileAuthTag::from_bytes(cmac)?;
        let request = RegisterInputFileRequest::new(url, cmac, file_crypto);
        let response = self.register_input_file_with_request(request)?;

        Ok(response.data_id.to_string())
    }

    pub fn register_output_file_with_request(
        &mut self,
        request: RegisterOutputFileRequest,
    ) -> Result<RegisterOutputFileResponse> {
        let response = self.api_client.register_output_file(request)?;

        Ok(response)
    }

    pub fn register_output_file_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: frontend_proto::RegisterOutputFileRequest =
            serde_json::from_str(serialized_request)?;
        let response: frontend_proto::RegisterOutputFileResponse = self
            .register_output_file_with_request(request.try_into()?)?
            .into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn register_output_file(&mut self, url: &str, file_crypto: FileCrypto) -> Result<String> {
        let url = Url::parse(url)?;
        let request = RegisterOutputFileRequest::new(url, file_crypto);
        let response = self.register_output_file_with_request(request)?;

        Ok(response.data_id.to_string())
    }

    pub fn create_task_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: frontend_proto::CreateTaskRequest = serde_json::from_str(serialized_request)?;
        let response: frontend_proto::CreateTaskResponse =
            self.create_task_with_request(request.try_into()?)?.into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn create_task_with_request(
        &mut self,
        request: CreateTaskRequest,
    ) -> Result<CreateTaskResponse> {
        let response = self.api_client.create_task(request)?;

        Ok(response)
    }

    pub fn create_task(
        &mut self,
        function_id: &str,
        function_arguments: Option<HashMap<String, String>>,
        executor: &str,
        inputs_ownership: Option<HashMap<String, Vec<String>>>,
        outputs_ownership: Option<HashMap<String, Vec<String>>>,
    ) -> Result<String> {
        use teaclave_types::OwnerList;
        let function_id = function_id.try_into()?;
        let executor = executor.try_into()?;

        let mut request = CreateTaskRequest::new()
            .function_id(function_id)
            .executor(executor);

        if let Some(function_arguments) = function_arguments {
            request = request.function_arguments(function_arguments);
        }

        if let Some(inputs_ownership) = inputs_ownership {
            let mut inputs_task_file_owners: HashMap<String, OwnerList> = HashMap::new();
            for (k, v) in inputs_ownership.iter() {
                inputs_task_file_owners.insert(k.into(), v.clone().into());
            }
            request = request.inputs_ownership(inputs_task_file_owners);
        }

        if let Some(outputs_ownership) = outputs_ownership {
            let mut outputs_task_file_owners: HashMap<String, OwnerList> = HashMap::new();
            for (k, v) in outputs_ownership.iter() {
                outputs_task_file_owners.insert(k.into(), v.clone().into());
            }
            request = request.outputs_ownership(outputs_task_file_owners);
        }

        let response = self.create_task_with_request(request)?;

        Ok(response.task_id.to_string())
    }

    pub fn assign_data_with_request(
        &mut self,
        request: AssignDataRequest,
    ) -> Result<AssignDataResponse> {
        let response = self.api_client.assign_data(request)?;

        Ok(response)
    }

    pub fn assign_data_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: frontend_proto::AssignDataRequest = serde_json::from_str(serialized_request)?;
        let response: frontend_proto::AssignDataResponse =
            self.assign_data_with_request(request.try_into()?)?.into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn assign_data(
        &mut self,
        task_id: &str,
        inputs: Option<HashMap<String, String>>,
        outputs: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let mut input_data = HashMap::new();
        let mut output_data = HashMap::new();
        if let Some(inputs) = inputs {
            for (k, v) in inputs.iter() {
                input_data.insert(k.into(), v.clone().try_into()?);
            }
        }

        if let Some(outputs) = outputs {
            for (k, v) in outputs.iter() {
                output_data.insert(k.into(), v.clone().try_into()?);
            }
        }
        let request = AssignDataRequest::new(task_id.try_into()?, input_data, output_data);
        let _ = self.assign_data_with_request(request)?;

        Ok(())
    }

    pub fn approve_task_with_request(
        &mut self,
        request: ApproveTaskRequest,
    ) -> Result<ApproveTaskResponse> {
        let response = self.api_client.approve_task(request)?;

        Ok(response)
    }

    pub fn approve_task(&mut self, task_id: &str) -> Result<()> {
        let request = ApproveTaskRequest::new(task_id.try_into()?);
        let _ = self.approve_task_with_request(request)?;

        Ok(())
    }

    pub fn approve_task_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: frontend_proto::ApproveTaskRequest = serde_json::from_str(serialized_request)?;
        let response: frontend_proto::ApproveTaskResponse =
            self.approve_task_with_request(request.try_into()?)?.into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn invoke_task_with_request(
        &mut self,
        request: InvokeTaskRequest,
    ) -> Result<InvokeTaskResponse> {
        let response = self.api_client.invoke_task(request)?;

        Ok(response)
    }

    pub fn invoke_task_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: frontend_proto::InvokeTaskRequest = serde_json::from_str(serialized_request)?;
        let response: frontend_proto::InvokeTaskResponse =
            self.invoke_task_with_request(request.try_into()?)?.into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn invoke_task(&mut self, task_id: &str) -> Result<()> {
        let request = InvokeTaskRequest::new(task_id.try_into()?);
        let _ = self.invoke_task_with_request(request)?;

        Ok(())
    }

    pub fn get_task_with_request(&mut self, request: GetTaskRequest) -> Result<GetTaskResponse> {
        let response = self.api_client.get_task(request)?;

        Ok(response)
    }

    pub fn get_task_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: frontend_proto::GetTaskRequest = serde_json::from_str(serialized_request)?;
        let response: frontend_proto::GetTaskResponse =
            self.get_task_with_request(request.try_into()?)?.into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn get_task_result(&mut self, task_id: &str) -> Result<(Vec<u8>, Vec<String>)> {
        loop {
            let request = GetTaskRequest::new(task_id.try_into()?);
            let response = self.get_task_with_request(request)?;
            match response.result {
                TaskResult::NotReady => {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                TaskResult::Ok(task_outputs) => {
                    return Ok((task_outputs.return_value, task_outputs.log));
                }
                TaskResult::Err(task_error) => {
                    return Err(anyhow::anyhow!(task_error.reason));
                }
            }
        }
    }

    pub fn cancel_task_with_request(
        &mut self,
        request: CancelTaskRequest,
    ) -> Result<CancelTaskResponse> {
        let response = self.api_client.cancel_task(request)?;

        Ok(response)
    }

    pub fn cancel_task_serialized(&mut self, serialized_request: &str) -> Result<String> {
        let request: frontend_proto::CancelTaskRequest = serde_json::from_str(serialized_request)?;
        let response: frontend_proto::CancelTaskResponse =
            self.cancel_task_with_request(request.try_into()?)?.into();
        let serialized_response = serde_json::to_string(&response)?;

        Ok(serialized_response)
    }

    pub fn cancel_task(&mut self, task_id: &str) -> Result<()> {
        let request = CancelTaskRequest::new(task_id.try_into()?);
        let _ = self.cancel_task_with_request(request)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;
    use std::fs;
    use teaclave_types::hashmap;

    const ENCLAVE_INFO_PATH: &str = "../../release/services/enclave_info.toml";
    #[cfg(dcap)]
    const AS_ROOT_CA_CERT_PATH: &str = "../../keys/dcap_root_ca_cert.pem";
    #[cfg(not(dcap))]
    const AS_ROOT_CA_CERT_PATH: &str = "../../keys/ias_root_ca_cert.pem";
    const USER_ID: &str = "rust_client_sdk_test_user";
    const USER_PASSWORD: &str = "test_password";
    const ADMIN_ID: &str = "admin";
    const ADMIN_PASSWORD: &str = "teaclave";

    #[test]
    fn test_authentication_service() {
        let enclave_info = EnclaveInfo::from_file(ENCLAVE_INFO_PATH).unwrap();
        let bytes = fs::read(AS_ROOT_CA_CERT_PATH).unwrap();
        let as_root_ca_cert = pem::parse(bytes).unwrap().contents;
        let mut client =
            AuthenticationService::connect("localhost:7776", &enclave_info, &as_root_ca_cert)
                .unwrap();
        let token = client.user_login(ADMIN_ID, ADMIN_PASSWORD).unwrap();
        client.set_credential(ADMIN_ID, &token);
        let _ = client.user_register(USER_ID, USER_PASSWORD, "PlatformAdmin", "");
        client.user_login(USER_ID, USER_PASSWORD).unwrap();
    }

    #[test]
    fn test_frontend_service() {
        let enclave_info = EnclaveInfo::from_file(ENCLAVE_INFO_PATH).unwrap();
        let bytes = fs::read(AS_ROOT_CA_CERT_PATH).unwrap();
        let as_root_ca_cert = pem::parse(bytes).unwrap().contents;
        let mut client =
            AuthenticationService::connect("localhost:7776", &enclave_info, &as_root_ca_cert)
                .unwrap();
        let token = client.user_login(ADMIN_ID, ADMIN_PASSWORD).unwrap();
        client.set_credential(ADMIN_ID, &token);
        let _ = client.user_register(USER_ID, USER_PASSWORD, "PlatformAdmin", "");
        let token = client.user_login(USER_ID, USER_PASSWORD).unwrap();

        let mut client =
            FrontendService::connect("localhost:7777", &enclave_info, &as_root_ca_cert).unwrap();
        client.set_credential(USER_ID, &token);
        let function_id = client
            .register_function(
                "builtin-echo",
                "An native echo function.",
                "builtin",
                None,
                Some(vec![FunctionArgument::new("message", "", true)]),
                None,
                None,
            )
            .unwrap();
        let _ = client.get_function(&function_id).unwrap();
        let function_arguments = hashmap!("message" => "Hello, Teaclave!");
        let task_id = client
            .create_task(
                &function_id,
                Some(function_arguments),
                "builtin",
                None,
                None,
            )
            .unwrap();

        let _ = client.invoke_task(&task_id).unwrap();
        let (result, log) = client.get_task_result(&task_id).unwrap();
        assert_eq!(result, b"Hello, Teaclave!");
        assert!(log.is_empty());
    }

    #[test]
    fn test_frontend_service_with_request() {
        let enclave_info = EnclaveInfo::from_file(ENCLAVE_INFO_PATH).unwrap();
        let bytes = fs::read(AS_ROOT_CA_CERT_PATH).unwrap();
        let as_root_ca_cert = pem::parse(bytes).unwrap().contents;
        let mut client =
            AuthenticationService::connect("localhost:7776", &enclave_info, &as_root_ca_cert)
                .unwrap();
        let token = client.user_login(ADMIN_ID, ADMIN_PASSWORD).unwrap();
        client.set_credential(ADMIN_ID, &token);
        let _ = client.user_register(USER_ID, USER_PASSWORD, "PlatformAdmin", "");
        let token = client.user_login(USER_ID, USER_PASSWORD).unwrap();

        let mut client =
            FrontendService::connect("localhost:7777", &enclave_info, &as_root_ca_cert).unwrap();
        client.set_credential(USER_ID, &token);
        let request = RegisterFunctionRequest::default();
        let function_id = client
            .register_function_with_request(request)
            .unwrap()
            .function_id;

        let request = GetFunctionRequest::new(function_id);
        let response = client.get_function_with_request(request);
        assert!(response.is_ok());

        let function_id = "function-00000000-0000-0000-0000-000000000002"
            .to_string()
            .try_into()
            .unwrap();
        let request = CreateTaskRequest::new()
            .function_id(function_id)
            .function_arguments(hashmap!("arg1" => "arg1_value"))
            .executor(Executor::MesaPy)
            .outputs_ownership(hashmap!("output" =>  vec!["frontend_user", "mock_user"]));
        let response = client.create_task_with_request(request);
        assert!(response.is_ok());
        let task_id = response.unwrap().task_id;

        let request = GetTaskRequest::new(task_id);
        let response = client.get_task_with_request(request);
        assert!(response.is_ok());
    }

    #[test]
    fn test_assign_data() {
        let enclave_info = EnclaveInfo::from_file(ENCLAVE_INFO_PATH).unwrap();
        let bytes = fs::read(AS_ROOT_CA_CERT_PATH).unwrap();
        let as_root_ca_cert = pem::parse(bytes).unwrap().contents;
        let mut client =
            AuthenticationService::connect("localhost:7776", &enclave_info, &as_root_ca_cert)
                .unwrap();
        let token = client.user_login(ADMIN_ID, ADMIN_PASSWORD).unwrap();
        client.set_credential(ADMIN_ID, &token);
        let _ = client.user_register(USER_ID, USER_PASSWORD, "PlatformAdmin", "");
        let token = client.user_login(USER_ID, USER_PASSWORD).unwrap();

        let mut client =
            FrontendService::connect("localhost:7777", &enclave_info, &as_root_ca_cert).unwrap();
        client.set_credential(USER_ID, &token);
        let function_id = "function-00000000-0000-0000-0000-000000000002";
        let function_arguments = hashmap!("arg1" => "arg1_value");
        let outputs_ownership = hashmap!("output" => vec![USER_ID.to_string()]);
        let task_id = client
            .create_task(
                &function_id,
                Some(function_arguments),
                "mesapy",
                None,
                Some(outputs_ownership),
            )
            .unwrap();
        let data_id = client
            .register_output_file(
                "https://external-storage.com/filepath?presigned_token",
                FileCrypto::default(),
            )
            .unwrap();
        let outputs = hashmap!("output" => data_id);
        client.assign_data(&task_id, None, Some(outputs)).unwrap();
    }

    #[test]
    fn test_assign_data_err() {
        let enclave_info = EnclaveInfo::from_file(ENCLAVE_INFO_PATH).unwrap();
        let bytes = fs::read(AS_ROOT_CA_CERT_PATH).unwrap();
        let as_root_ca_cert = pem::parse(bytes).unwrap().contents;
        let mut client =
            AuthenticationService::connect("localhost:7776", &enclave_info, &as_root_ca_cert)
                .unwrap();
        let token = client.user_login(ADMIN_ID, ADMIN_PASSWORD).unwrap();
        client.set_credential(ADMIN_ID, &token);
        let _ = client.user_register(USER_ID, USER_PASSWORD, "PlatformAdmin", "");
        let token = client.user_login(USER_ID, USER_PASSWORD).unwrap();

        let mut client =
            FrontendService::connect("localhost:7777", &enclave_info, &as_root_ca_cert).unwrap();
        client.set_credential(USER_ID, &token);
        let function_id = "function-00000000-0000-0000-0000-000000000002";
        let function_arguments = hashmap!("arg1" => "arg1_value");
        let outputs_ownership = hashmap!("output" => vec!["incorrect_user".to_string()]);
        let task_id = client
            .create_task(
                &function_id,
                Some(function_arguments),
                "mesapy",
                None,
                Some(outputs_ownership),
            )
            .unwrap();
        let data_id = client
            .register_output_file(
                "https://external-storage.com/filepath?presigned_token",
                FileCrypto::default(),
            )
            .unwrap();
        let outputs = hashmap!("output" => data_id);
        let result = client.assign_data(&task_id, None, Some(outputs));
        assert!(result.is_err());
    }

    #[test]
    fn test_approve_task() {
        let enclave_info = EnclaveInfo::from_file(ENCLAVE_INFO_PATH).unwrap();
        let bytes = fs::read(AS_ROOT_CA_CERT_PATH).unwrap();
        let as_root_ca_cert = pem::parse(bytes).unwrap().contents;
        let mut client =
            AuthenticationService::connect("localhost:7776", &enclave_info, &as_root_ca_cert)
                .unwrap();
        let token = client.user_login(ADMIN_ID, ADMIN_PASSWORD).unwrap();
        client.set_credential(ADMIN_ID, &token);
        let _ = client.user_register(USER_ID, USER_PASSWORD, "PlatformAdmin", "");
        let token = client.user_login(USER_ID, USER_PASSWORD).unwrap();

        let mut client =
            FrontendService::connect("localhost:7777", &enclave_info, &as_root_ca_cert).unwrap();
        client.set_credential(USER_ID, &token);
        let function_id = "function-00000000-0000-0000-0000-000000000002";
        let function_arguments = hashmap!("arg1" => "arg1_value");
        let outputs_ownership = hashmap!("output" => vec![USER_ID.to_string()]);
        let task_id = client
            .create_task(
                &function_id,
                Some(function_arguments),
                "mesapy",
                None,
                Some(outputs_ownership),
            )
            .unwrap();
        let data_id = client
            .register_output_file(
                "https://external-storage.com/filepath?presigned_token",
                FileCrypto::default(),
            )
            .unwrap();
        let outputs = hashmap!("output" => data_id);
        client.assign_data(&task_id, None, Some(outputs)).unwrap();
        client.approve_task(&task_id).unwrap();
    }

    #[test]
    fn test_cancel_task() {
        let enclave_info = EnclaveInfo::from_file(ENCLAVE_INFO_PATH).unwrap();
        let bytes = fs::read(AS_ROOT_CA_CERT_PATH).unwrap();
        let as_root_ca_cert = pem::parse(bytes).unwrap().contents;
        let mut client =
            AuthenticationService::connect("localhost:7776", &enclave_info, &as_root_ca_cert)
                .unwrap();
        let token = client.user_login(ADMIN_ID, ADMIN_PASSWORD).unwrap();
        client.set_credential(ADMIN_ID, &token);
        let _ = client.user_register(USER_ID, USER_PASSWORD, "PlatformAdmin", "");
        let token = client.user_login(USER_ID, USER_PASSWORD).unwrap();

        let mut client =
            FrontendService::connect("localhost:7777", &enclave_info, &as_root_ca_cert).unwrap();
        client.set_credential(USER_ID, &token);
        let function_id = "function-00000000-0000-0000-0000-000000000002";
        let function_arguments = hashmap!("arg1" => "arg1_value");
        let outputs_ownership = hashmap!("output" => vec![USER_ID.to_string()]);
        let task_id = client
            .create_task(
                &function_id,
                Some(function_arguments),
                "mesapy",
                None,
                Some(outputs_ownership),
            )
            .unwrap();

        print!("SDK DEBUG: task created");

        let result = client.cancel_task(&task_id);
        print!("SDK DEBUG: canceled, {:?}", result);

        let task = client.get_task_result(&task_id);
        assert!(task.is_err());
    }
}
