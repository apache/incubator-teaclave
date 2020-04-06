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

use crate::utils::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::prelude::v1::*;
use teaclave_attestation::verifier;
use teaclave_config::RuntimeConfig;
use teaclave_config::BUILD_CONFIG;
use teaclave_proto::teaclave_authentication_service::*;
use teaclave_proto::teaclave_common::*;
use teaclave_proto::teaclave_frontend_service::*;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::*;
use url::Url;

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(
        test_register_input_file,
        test_register_output_file,
        test_register_fusion_output,
        test_register_input_from_output,
        test_get_output_file,
        test_get_input_file,
        test_register_function,
        test_get_function,
        test_create_task,
        test_get_task,
        test_assign_data,
        test_approve_task,
        test_invoke_task,
    )
}

fn get_credential() -> UserCredential {
    // register user and login
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let enclave_info =
        EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes.as_ref().unwrap());
    let enclave_attr = enclave_info
        .get_enclave_attr("teaclave_authentication_service")
        .expect("authentication");
    let config = SgxTrustedTlsClientConfig::new().attestation_report_verifier(
        vec![enclave_attr],
        BUILD_CONFIG.as_root_ca_cert,
        verifier::universal_quote_verifier,
    );
    let channel = Endpoint::new("localhost:7776")
        .config(config)
        .connect()
        .unwrap();
    let mut api_client = TeaclaveAuthenticationApiClient::new(channel).unwrap();

    let request = UserRegisterRequest::new("frontend_user", "test_password");
    let _response_result = api_client.user_register(request);

    let request = UserLoginRequest::new("frontend_user", "test_password");
    let response_result = api_client.user_login(request);
    assert!(response_result.is_ok());
    UserCredential::new("frontend_user", response_result.unwrap().token)
}

fn get_client() -> TeaclaveFrontendClient {
    let user_credential = get_credential();
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let port = &runtime_config.api_endpoints.frontend.listen_address.port();
    let channel = Endpoint::new(&format!("localhost:{}", port))
        .connect()
        .unwrap();

    let mut metadata = HashMap::new();
    metadata.insert("id".to_string(), user_credential.id);
    metadata.insert("token".to_string(), user_credential.token);

    TeaclaveFrontendClient::new_with_metadata(channel, metadata).unwrap()
}

fn test_register_input_file() {
    let mut client = get_client();

    let request = RegisterInputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        hash: "deadbeefdeadbeef".to_string(),
        crypto_info: FileCrypto::default(),
    };
    let response = client.register_input_file(request);
    assert!(response.is_ok());

    let request = RegisterInputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        hash: "deadbeefdeadbeef".to_string(),
        crypto_info: FileCrypto::default(),
    };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.register_input_file(request);
    assert_eq!(
        response,
        Err(TeaclaveServiceResponseError::RequestError(
            "authentication error".to_string()
        ))
    );
}

fn test_register_output_file() {
    let mut client = get_client();

    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: FileCrypto::default(),
    };
    let response = client.register_output_file(request);
    assert!(response.is_ok());

    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: FileCrypto::default(),
    };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.register_output_file(request);
    assert!(response.is_err());
}

fn test_register_fusion_output() {
    let mut client = get_client();
    let request = RegisterFusionOutputRequest {
        owner_list: vec!["frontend_user", "mock_user"].into(),
    };
    let response = client.register_fusion_output(request);
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest {
        owner_list: vec!["frontend_user", "mock_user"].into(),
    };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.register_fusion_output(request);
    assert!(response.is_err());
}

fn test_register_input_from_output() {
    let mut client = get_client();
    let data_id = ExternalID::try_from("output-00000000-0000-0000-0000-000000000001").unwrap();
    let request = RegisterInputFromOutputRequest {
        data_id: data_id.clone(),
    };
    let response = client.register_input_from_output(request);
    assert!(response.is_ok());

    let request = RegisterInputFromOutputRequest { data_id };
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.register_input_from_output(request);
    assert!(response.is_err());
}

fn test_get_output_file() {
    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: FileCrypto::default(),
    };

    let mut client = get_client();
    let response = client.register_output_file(request).unwrap();
    let data_id = response.data_id;

    let request = GetOutputFileRequest::new(data_id.clone());
    let response = client.get_output_file(request).unwrap();
    assert!(response.hash.is_empty());

    let request = GetOutputFileRequest::new(data_id);
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.get_output_file(request);
    assert!(response.is_err());
}

fn test_get_input_file() {
    let request = RegisterInputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        hash: "deadbeef".to_string(),
        crypto_info: FileCrypto::default(),
    };

    let mut client = get_client();
    let response = client.register_input_file(request).unwrap();
    let data_id = response.data_id;

    let request = GetInputFileRequest::new(data_id.clone());
    let response = client.get_input_file(request).unwrap();
    assert!(!response.hash.is_empty());

    let request = GetInputFileRequest::new(data_id);
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.get_input_file(request);
    assert!(response.is_err());
}

fn test_register_function() {
    let mut client = get_client();

    let request = RegisterFunctionRequest::default();
    let response = client.register_function(request);
    assert!(response.is_ok());

    let request = RegisterFunctionRequest::default();
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.register_function(request);
    assert!(response.is_err());
}

fn test_get_function() {
    let mut client = get_client();

    let registered_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000001").unwrap();
    let request = GetFunctionRequest::new(registered_id.clone());
    let response = client.get_function(request).unwrap();
    assert!(!response.name.is_empty());

    let request = GetFunctionRequest::new(registered_id);
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.get_function(request);
    assert!(response.is_err());
}

fn test_create_task() {
    let mut client = get_client();

    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();

    let request = CreateTaskRequest::new()
        .function_id(function_id.clone())
        .function_arguments(hashmap!("arg1" => "data1"))
        .executor(Executor::MesaPy)
        .output_owners_map(hashmap!("output" =>  vec!["frontend_user", "mock_user"]));
    let response = client.create_task(request);
    assert!(response.is_ok());

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "data1"))
        .executor(Executor::MesaPy)
        .output_owners_map(hashmap!("output" => vec!["frontend_user", "mock_user"]));
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.create_task(request);
    assert!(response.is_err());
}

fn test_get_task() {
    let mut client = get_client();
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "data1"))
        .executor(Executor::MesaPy)
        .output_owners_map(hashmap!("output" => vec!["frontend_user", "mock_user"]));
    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    let request = GetTaskRequest::new(task_id.clone());
    let response = client.get_task(request);
    assert!(response.is_ok());

    let request = GetTaskRequest::new(task_id);
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.get_task(request);
    assert!(response.is_err());
}

fn test_assign_data() {
    let mut client = get_client();

    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();

    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "data1"))
        .executor(Executor::MesaPy)
        .output_owners_map(hashmap!("output" => vec!["frontend_user"]));

    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: FileCrypto::default(),
    };
    let response = client.register_output_file(request).unwrap();
    let output_id = response.data_id;

    let request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: hashmap!("output" => output_id.clone()),
    };
    let correct_token = client.metadata().get("token").unwrap().to_string();
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.assign_data(request);
    assert!(response.is_err());

    let request = AssignDataRequest {
        task_id,
        input_map: HashMap::new(),
        output_map: hashmap!("output" => output_id),
    };
    client
        .metadata_mut()
        .insert("token".to_string(), correct_token);
    let response = client.assign_data(request);
    assert!(response.is_ok());
}

fn test_approve_task() {
    let mut client = get_client();

    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "data1"))
        .executor(Executor::MesaPy)
        .output_owners_map(hashmap!("output" => vec!["frontend_user"]));

    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: FileCrypto::default(),
    };

    let response = client.register_output_file(request).unwrap();
    let output_id = response.data_id;

    let request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: hashmap!("output" => output_id),
    };

    client.assign_data(request).unwrap();

    let request = ApproveTaskRequest::new(task_id.clone());
    let correct_token = client.metadata().get("token").unwrap().to_string();
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.approve_task(request);
    assert!(response.is_err());

    let request = ApproveTaskRequest::new(task_id);
    client
        .metadata_mut()
        .insert("token".to_string(), correct_token);
    let response = client.approve_task(request);
    assert!(response.is_ok());
}

fn test_invoke_task() {
    let mut client = get_client();
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000002").unwrap();
    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "data1"))
        .executor(Executor::MesaPy)
        .output_owners_map(hashmap!("output" => vec!["frontend_user"]));

    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    let request = RegisterOutputFileRequest {
        url: Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap(),
        crypto_info: FileCrypto::default(),
    };
    let response = client.register_output_file(request).unwrap();
    let output_id = response.data_id;

    let request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: hashmap!("output" => output_id),
    };

    client.assign_data(request).unwrap();

    let request = ApproveTaskRequest::new(task_id.clone());
    client.approve_task(request).unwrap();

    let request = InvokeTaskRequest::new(task_id.clone());
    let correct_token = client.metadata().get("token").unwrap().to_string();
    client
        .metadata_mut()
        .insert("token".to_string(), "wrong token".to_string());
    let response = client.invoke_task(request);
    assert!(response.is_err());

    let request = InvokeTaskRequest::new(task_id.clone());
    client
        .metadata_mut()
        .insert("token".to_string(), correct_token);
    let response = client.invoke_task(request);
    assert!(response.is_ok());

    let request = GetTaskRequest::new(task_id);
    let response = client.get_task(request).unwrap();
    assert_eq!(response.status, TaskStatus::Running);

    let request = PullTaskRequest {};
    let mut scheduler_client = get_scheduler_client();
    let response = scheduler_client.pull_task(request);
    assert!(response.is_ok());
}
