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
use futures::FutureExt;
use std::convert::TryFrom;
use teaclave_proto::teaclave_common::i32_from_task_status;
use teaclave_proto::teaclave_management_service::*;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_rpc::CredentialService;
use teaclave_test_utils::async_test_case;
use teaclave_types::*;
use url::Url;
use uuid::Uuid;

async fn authorized_client(user_id: &str) -> TeaclaveManagementClient<CredentialService> {
    get_management_client(user_id).await
}

#[async_test_case]
async fn test_register_input_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let cmac = FileAuthTag::mock();

    let request = RegisterInputFileRequest::new(url, cmac, FileCrypto::default());
    let mut client = authorized_client("mock_user").await;
    let response = client.register_input_file(request).await;
    assert!(response.is_ok());
}

#[async_test_case]
async fn test_register_output_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::new("aes-gcm-128", &[0x90u8; 16], &[0x89u8; 12]).unwrap();

    let request = RegisterOutputFileRequest::new(url, crypto_info);
    let mut client = authorized_client("mock_user").await;
    let response = client.register_output_file(request).await;
    assert!(response.is_ok());
}

#[async_test_case]
async fn test_register_fusion_output() {
    let request = RegisterFusionOutputRequest::new(vec!["mock_user", "mock_user_b"]);
    let mut client = authorized_client("mock_user").await;
    let response = client.register_fusion_output(request).await;
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest::new(vec!["mock_user_c", "mock_user_b"]);
    let response = client.register_fusion_output(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_register_input_from_output() {
    let user1_output_id =
        ExternalID::try_from("output-00000000-0000-0000-0000-000000000001").unwrap();

    // not a owner
    let request = RegisterInputFromOutputRequest::new(user1_output_id.clone());
    let mut client = authorized_client("mock_user_c").await;
    let response = client.register_input_from_output(request).await;
    assert!(response.is_err());

    // output not ready
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::default();
    let mut client = authorized_client("mock_user1").await;
    let request = RegisterOutputFileRequest::new(url, crypto_info);
    let response = client.register_output_file(request).await.unwrap();
    let request =
        RegisterInputFromOutputRequest::new(response.into_inner().data_id.try_into().unwrap());
    let response = client.register_input_from_output(request).await;
    assert!(response.is_err());

    let request = RegisterInputFromOutputRequest::new(user1_output_id);
    let response = client.register_input_from_output(request).await;
    assert!(response.is_ok());
}

#[async_test_case]
async fn test_get_output_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::default();
    let request = RegisterOutputFileRequest::new(url, crypto_info);

    let mut client = authorized_client("mock_user").await;
    let response = client.register_output_file(request).await.unwrap();
    let data_id = ExternalID::try_from(response.into_inner().data_id).unwrap();
    let request = GetOutputFileRequest::new(data_id.clone());
    let response = client.get_output_file(request).await;
    assert!(response.is_ok());

    let request = GetOutputFileRequest::new(data_id);
    let mut client = authorized_client("mock_another_user").await;
    let response = client.get_output_file(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_get_input_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let cmac = FileAuthTag::mock();
    let crypto_info = FileCrypto::default();

    let mut client = authorized_client("mock_user").await;
    let request = RegisterInputFileRequest::new(url, cmac, crypto_info);
    let response = client.register_input_file(request).await;
    let data_id = ExternalID::try_from(response.unwrap().into_inner().data_id).unwrap();
    let request = GetInputFileRequest::new(data_id.clone());
    let response = client.get_input_file(request).await;
    assert!(response.is_ok());

    let mut client = authorized_client("mock_another_user").await;
    let request = GetInputFileRequest::new(data_id);
    let response = client.get_input_file(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_register_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(true)
        .arguments(vec![FunctionArgument::new("arg", "", true)])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .usage_quota(Some(0))
        .build();

    let mut client = authorized_client("mock_user").await;
    let response = client.register_function(request).await;
    assert!(response.is_ok());
}

#[async_test_case]
async fn test_register_private_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(false)
        .arguments(vec![FunctionArgument::new("arg", "", true)])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .user_allowlist(vec!["mock_user".to_string()])
        .build();

    let mut client = authorized_client("mock_user").await;
    let response = client.register_function(request).await;

    assert!(response.is_ok());
}

#[async_test_case]
async fn test_delete_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(true)
        .arguments(vec![FunctionArgument::new("arg", "", true)])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .build();

    let mut client = authorized_client("mock_user").await;
    let response = client.register_function(request).await.unwrap();
    let function_id = ExternalID::try_from(response.into_inner().function_id).unwrap();

    let request = DeleteFunctionRequest::new(function_id);
    let response = client.delete_function(request).await;
    assert!(response.is_ok());
}

#[async_test_case]
async fn test_disable_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(true)
        .arguments(vec![FunctionArgument::new("arg", "", true)])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .build();

    let mut client = authorized_client("mock_user").await;
    let response = client
        .register_function(request)
        .await
        .unwrap()
        .into_inner();
    let function_id = ExternalID::try_from(response.function_id).unwrap();

    let request = DisableFunctionRequest::new(function_id);
    let response = client.disable_function(request).await;
    assert!(response.is_ok());
}

#[async_test_case]
async fn test_update_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(true)
        .arguments(vec![FunctionArgument::new("arg", "", true)])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .build();

    let mut client = authorized_client("mock_user").await;
    let response = client.register_function(request).await;
    let original_id = ExternalID::try_from(response.unwrap().into_inner().function_id).unwrap();

    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = UpdateFunctionRequestBuilder::new()
        .function_id(original_id.clone())
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(false)
        .arguments(vec![FunctionArgument::new("arg", "", true)])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .user_allowlist(vec!["mock_user".to_string()])
        .build();

    let mut client = authorized_client("mock_user").await;
    let response = client.update_function(request.clone()).await;
    assert!(response.is_ok());
    assert!(original_id.to_string() == response.unwrap().into_inner().function_id);

    let mock_id = ExternalID::try_from("function-00000000-0000-0000-0000-000000000006").unwrap();
    let mut request_mock_id = request.clone();
    request_mock_id.function_id = mock_id.to_string();
    let response = client.update_function(request_mock_id).await;
    assert!(response.is_err());

    let mut client = authorized_client("another_mock_user").await;
    let response = client.update_function(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_list_functions() {
    let request = ListFunctionsRequest {
        user_id: "mock_user".into(),
    };

    let mut client = authorized_client("mock_user").await;
    let response = client.list_functions(request).await;
    assert!(response.is_ok());
}

#[async_test_case]
async fn test_get_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(false)
        .arguments(vec![FunctionArgument::new("arg", "", true)])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .build();

    let mut client = authorized_client("mock_user").await;
    let response = client
        .register_function(request)
        .await
        .unwrap()
        .into_inner();
    let function_id = ExternalID::try_from(response.function_id).unwrap();

    let request = GetFunctionRequest::new(function_id.clone());
    let response = client.get_function(request).await;
    assert!(response.is_ok());

    let mut client = authorized_client("mock_unauthorized_user").await;
    let request = GetFunctionRequest::new(function_id);
    let response = client.get_function(request).await;
    // mock_unauthorized_user is PlatformAdmin
    assert!(response.is_ok());

    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000001").unwrap();
    let request = GetFunctionRequest::new(function_id);
    let response = client.get_function(request).await;
    assert!(response.is_ok());

    // private functions
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000003").unwrap();

    let mut client = authorized_client("mock_user").await;
    let request = GetFunctionRequest::new(function_id.clone());
    let response = client.get_function(request).await;
    assert!(response.is_ok());

    let mut client = authorized_client("mock_user1").await;
    let request = GetFunctionRequest::new(function_id.clone());
    let response = client.get_function(request).await;
    assert!(response.is_ok());

    let mut client = authorized_client("mock_unauthorized_user").await;
    let request = GetFunctionRequest::new(function_id);
    let response = client.get_function(request).await;
    // mock_unauthorized_user is PlatformAdmin
    assert!(response.is_ok());
}

fn create_valid_task_request() -> CreateTaskRequest {
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000001").unwrap();
    let input_owners = hashmap!(
            "input" => vec!["mock_user1"],
            "input2" => vec!["mock_user2", "mock_user3"]
    );
    let output_owners = hashmap!(
            "output" => vec!["mock_user1"],
            "output2" => vec!["mock_user2", "mock_user3"]
    );

    CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "data1", "arg2" => "data2"))
        .executor(Executor::MesaPy)
        .inputs_ownership(input_owners)
        .outputs_ownership(output_owners)
}

fn create_valid_task_request_private_function() -> CreateTaskRequest {
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000003").unwrap();

    CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("arg1" => "data1"))
        .executor(Executor::MesaPy)
}

#[async_test_case]
async fn test_create_task() {
    let mut client = authorized_client("mock_user").await;

    let request = CreateTaskRequest::new().executor(Executor::MesaPy);
    let response = client.create_task(request).await;
    assert!(response.is_err());

    let request = create_valid_task_request();
    let response = client.create_task(request).await;
    assert!(response.is_ok());

    let request = create_valid_task_request_private_function();
    let response = client.create_task(request).await;
    assert!(response.is_ok());

    let mut request = create_valid_task_request();
    let mut function_arguments: FunctionArguments = request.function_arguments.try_into().unwrap();
    function_arguments.inner_mut().remove("arg1");
    request.function_arguments = function_arguments.into_string();
    let response = client.create_task(request).await;
    assert!(response.is_err());

    let mut request = create_valid_task_request();
    request = request.inputs_ownership(hashmap!(
        "input2" => vec!["mock_user2", "mock_user3"]
    ));
    let response = client.create_task(request).await;
    assert!(response.is_err());

    let mut request = create_valid_task_request();
    request = request.outputs_ownership(hashmap!(
            "output2" => vec!["mock_user2", "mock_user3"]
    ));
    let response = client.create_task(request).await;
    assert!(response.is_err());

    let mut client = authorized_client("mock_user2").await;
    let request = create_valid_task_request_private_function();
    let response = client.create_task(request).await;
    // PlatformAdmin can access private function
    assert!(response.is_ok());
}

#[async_test_case]
async fn test_get_task() {
    let mut client = authorized_client("mock_user").await;

    let request = create_valid_task_request();
    let response = client.create_task(request).await.unwrap();
    let task_id = ExternalID::try_from(response.into_inner().task_id).unwrap();

    let request = GetTaskRequest::new(task_id);
    let response = client.get_task(request).await.unwrap().into_inner();
    assert!(response.participants.len() == 4);

    let participants = vec!["mock_user1", "mock_user3", "mock_user2", "mock_user"];
    for name in participants {
        assert!(response.participants.contains(&name.to_string()));
    }
}

#[async_test_case]
async fn test_assign_data() {
    let mut client = authorized_client("mock_user").await;
    let mut client1 = authorized_client("mock_user1").await;
    let mut client2 = authorized_client("mock_user2").await;
    let mut client3 = authorized_client("mock_user3").await;
    let request = create_valid_task_request();
    let response = client.create_task(request).await.unwrap().into_inner();
    let task_id = ExternalID::try_from(response.task_id).unwrap();

    // not a participant
    let request = AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!());

    let mut unknown_client = authorized_client("non-participant").await;
    let response = unknown_client.assign_data(request).await;
    assert!(response.is_err());

    // !input_file.owner.contains(user_id)
    let url = Url::parse("https://path").unwrap();
    let cmac = FileAuthTag::mock();
    let request = RegisterInputFileRequest::new(url, cmac, FileCrypto::default());
    let response = client2
        .register_input_file(request)
        .await
        .unwrap()
        .into_inner();
    let input_file_id_user2 = ExternalID::try_from(response.data_id).unwrap();

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => input_file_id_user2.clone()),
        hashmap!(),
    );
    let response = client1.assign_data(request).await;
    assert!(response.is_err());

    // !output_file.owner.contains(user_id)
    let url = Url::parse("https://output_file_path").unwrap();
    let request = RegisterOutputFileRequest::new(url, FileCrypto::default());
    let response = client2
        .register_output_file(request)
        .await
        .unwrap()
        .into_inner();
    let output_file_id_user2 = ExternalID::try_from(response.data_id).unwrap();

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output" => output_file_id_user2.clone()),
    );
    let response = client1.assign_data(request).await;
    assert!(response.is_err());

    let existing_outfile_id_user1 =
        ExternalID::try_from("output-00000000-0000-0000-0000-000000000001").unwrap();

    // output_file.cmac.is_some()
    let request = GetOutputFileRequest::new(existing_outfile_id_user1.clone());
    client1.get_output_file(request).await.unwrap();

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output" => existing_outfile_id_user1),
    );
    let response = client1.assign_data(request).await;
    assert!(response.is_err());

    // !fusion_data.owner_id_list.contains(user_id)
    let file_id2 = ExternalID::try_from("input-00000000-0000-0000-0000-000000000002").unwrap();
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => file_id2.clone()),
        hashmap!(),
    );
    let response = client1.assign_data(request).await;
    assert!(response.is_err());

    // inputs_ownership doesn't contain the name
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("none" => input_file_id_user2.clone()),
        hashmap!(),
    );
    let response = client2.assign_data(request).await;
    assert!(response.is_err());

    // outputs_ownership doesn't contain the name
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("none" => output_file_id_user2.clone()),
    );
    let response = client2.assign_data(request).await;
    assert!(response.is_err());

    //input file: OwnerList != user_id
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input2" => input_file_id_user2.clone()),
        hashmap!(),
    );
    let response = client2.assign_data(request).await;
    assert!(response.is_err());

    // input file: OwnerList != user_id
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => input_file_id_user2),
        hashmap!(),
    );
    let response = client2.assign_data(request).await;
    assert!(response.is_err());

    // output file OwnerList != uids
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output2" => output_file_id_user2.clone()),
    );
    let response = client2.assign_data(request).await;
    assert!(response.is_err());

    // output file: OwnerList != uids
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output1" => output_file_id_user2),
    );
    let response = client2.assign_data(request).await;
    assert!(response.is_err());

    // assign all the data
    let url = Url::parse("input://path").unwrap();
    let cmac = FileAuthTag::mock();
    let request = RegisterInputFileRequest::new(url, cmac, FileCrypto::default());
    let response = client1
        .register_input_file(request)
        .await
        .unwrap()
        .into_inner();
    let input_file_id_user1 = ExternalID::try_from(response.data_id).unwrap();

    let url = Url::parse("https://output_file_path").unwrap();
    let request = RegisterOutputFileRequest::new(url, FileCrypto::default());
    let response = client1
        .register_output_file(request)
        .await
        .unwrap()
        .into_inner();
    let output_file_id_user1 = ExternalID::try_from(response.data_id).unwrap();

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => input_file_id_user1.clone()),
        hashmap!("output" => output_file_id_user1),
    );
    let response = client1.assign_data(request).await;
    assert!(response.is_ok());

    let request =
        AssignDataRequest::new(task_id.clone(), hashmap!("input2" => file_id2), hashmap!());
    let response = client3.assign_data(request).await;
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest::new(vec!["mock_user2", "mock_user3"]);
    let response = client3
        .register_fusion_output(request)
        .await
        .unwrap()
        .into_inner();
    let fusion_output = ExternalID::try_from(response.data_id).unwrap();
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output2" => fusion_output),
    );
    let response = client3.assign_data(request).await;
    assert!(response.is_ok());

    let request = GetTaskRequest::new(task_id.clone());
    let response = client3.get_task(request).await.unwrap().into_inner();
    assert_eq!(
        response.status,
        i32_from_task_status(TaskStatus::DataAssigned)
    );

    // task.status != Created
    let request = AssignDataRequest::new(
        task_id,
        hashmap!("input" => input_file_id_user1),
        hashmap!(),
    );
    let response = client1.assign_data(request).await;
    assert!(response.is_err());
}

#[async_test_case]
async fn test_approve_task() {
    let mut client = authorized_client("mock_user").await;
    let mut client1 = authorized_client("mock_user1").await;
    let mut client2 = authorized_client("mock_user2").await;
    let mut client3 = authorized_client("mock_user3").await;
    let request = create_valid_task_request();
    let response = client.create_task(request).await.unwrap().into_inner();
    let task_id = ExternalID::try_from(response.task_id).unwrap();

    // task_status != ready
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client1.approve_task(request).await;
    assert!(response.is_err());

    // assign all the data
    let url = Url::parse("input://path").unwrap();
    let cmac = FileAuthTag::mock();
    let request = RegisterInputFileRequest::new(url, cmac, FileCrypto::default());
    let response = client1
        .register_input_file(request)
        .await
        .unwrap()
        .into_inner();
    let input_file_id_user1 = ExternalID::try_from(response.data_id).unwrap();
    let url = Url::parse("https://output_file_path").unwrap();
    let request = RegisterOutputFileRequest::new(url, FileCrypto::default());
    let response = client1
        .register_output_file(request)
        .await
        .unwrap()
        .into_inner();
    let output_file_id_user1 = ExternalID::try_from(response.data_id).unwrap();
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => input_file_id_user1),
        hashmap!("output" => output_file_id_user1),
    );
    let response = client1.assign_data(request).await;
    assert!(response.is_ok());

    let input_file_id_user2 =
        ExternalID::try_from("input-00000000-0000-0000-0000-000000000002").unwrap();
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input2" => input_file_id_user2),
        hashmap!(),
    );
    let response = client2.assign_data(request).await;
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest::new(vec!["mock_user2", "mock_user3"]);
    let response = client3.register_fusion_output(request).await;
    let fusion_output = ExternalID::try_from(response.unwrap().into_inner().data_id).unwrap();
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output2" => fusion_output),
    );
    let response = client3.assign_data(request).await;
    assert!(response.is_ok());

    let request = GetTaskRequest::new(task_id.clone());
    let response = client2.get_task(request).await.unwrap().into_inner();
    assert_eq!(
        response.status,
        i32_from_task_status(TaskStatus::DataAssigned)
    );

    // user_id not in task.participants
    let mut unknown_client = authorized_client("non-participant").await;
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = unknown_client.approve_task(request).await;
    assert!(response.is_err());

    //all participants approve the task
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client.approve_task(request).await;
    assert!(response.is_ok());
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client1.approve_task(request).await;
    assert!(response.is_ok());
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client2.approve_task(request).await;
    assert!(response.is_ok());
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client3.approve_task(request).await;
    assert!(response.is_ok());
    let request = GetTaskRequest::new(task_id);
    let response = client2.get_task(request).await.unwrap().into_inner();
    assert_eq!(response.status, i32_from_task_status(TaskStatus::Approved));
}

#[async_test_case]
async fn test_invoke_task() {
    let mut client = authorized_client("mock_user").await;
    let mut client1 = authorized_client("mock_user1").await;
    let mut client2 = authorized_client("mock_user2").await;
    let mut client3 = authorized_client("mock_user3").await;
    let request = create_valid_task_request();
    let response = client.create_task(request).await;
    assert!(response.is_ok());
    let task_id: ExternalID = response.unwrap().into_inner().task_id.try_into().unwrap();

    // assign all the data
    let url = Url::parse("input://path").unwrap();
    let cmac = FileAuthTag::mock();
    let request = RegisterInputFileRequest::new(url, cmac, FileCrypto::default());
    let response = client1
        .register_input_file(request)
        .await
        .unwrap()
        .into_inner();

    let input_file_id_user1: ExternalID = response.data_id.try_into().unwrap();

    let url = Url::parse("https://output_file_path").unwrap();
    let request = RegisterOutputFileRequest::new(url, FileCrypto::default());
    let response = client1
        .register_output_file(request)
        .await
        .unwrap()
        .into_inner();
    let output_file_id_user1: ExternalID = response.data_id.try_into().unwrap();

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => input_file_id_user1),
        hashmap!("output" => output_file_id_user1),
    );
    client1.assign_data(request).await.unwrap();

    let input_file_id_user2 =
        ExternalID::try_from("input-00000000-0000-0000-0000-000000000002").unwrap();
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input2" => input_file_id_user2),
        hashmap!(),
    );
    let response = client2.assign_data(request).await;
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest::new(vec!["mock_user2", "mock_user3"]);
    let response = client3.register_fusion_output(request).await;
    let fusion_output = ExternalID::try_from(response.unwrap().into_inner().data_id).unwrap();
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output2" => fusion_output),
    );
    let response = client3.assign_data(request).await;
    assert!(response.is_ok());

    // task status != Approved
    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client.invoke_task(request).await;
    assert!(response.is_err());

    //all participants approve the task
    let request = ApproveTaskRequest::new(task_id.clone());
    client.approve_task(request).await.unwrap();
    let request = ApproveTaskRequest::new(task_id.clone());
    client1.approve_task(request).await.unwrap();
    let request = ApproveTaskRequest::new(task_id.clone());
    client2.approve_task(request).await.unwrap();
    let request = ApproveTaskRequest::new(task_id.clone());
    client3.approve_task(request).await.unwrap();
    let request = GetTaskRequest::new(task_id.clone());
    let response = client2.get_task(request).await.unwrap().into_inner();
    assert_eq!(response.status, i32_from_task_status(TaskStatus::Approved));

    // user_id != task.creator
    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client2.invoke_task(request).await;
    assert!(response.is_err());

    // invoke task
    let request = InvokeTaskRequest::new(task_id.clone());
    client.invoke_task(request).await.unwrap();

    let request = GetTaskRequest::new(task_id);
    let response = client2.get_task(request).await.unwrap().into_inner();
    assert_eq!(response.status, i32_from_task_status(TaskStatus::Staged));

    let mut scheduler_client = get_scheduler_client().await;
    let executor_id = Uuid::new_v4().to_string();

    std::thread::sleep(std::time::Duration::from_secs(2));

    let pull_task_request = PullTaskRequest { executor_id };
    let response = scheduler_client.pull_task(pull_task_request).await;
    assert!(response.is_ok());
}
