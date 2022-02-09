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
use std::convert::TryFrom;
use std::prelude::v1::*;
use teaclave_proto::teaclave_management_service::*;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_test_utils::test_case;
use teaclave_types::*;
use url::Url;
use uuid::Uuid;

fn authorized_client(user_id: &str) -> TeaclaveManagementClient {
    get_management_client(user_id)
}

#[test_case]
fn test_register_input_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let cmac = FileAuthTag::mock();

    let request = RegisterInputFileRequest::new(url, cmac, FileCrypto::default());
    let response = authorized_client("mock_user").register_input_file(request);
    assert!(response.is_ok());
}

#[test_case]
fn test_register_output_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::new("aes-gcm-128", &[0x90u8; 16], &[0x89u8; 12]).unwrap();

    let request = RegisterOutputFileRequest::new(url, crypto_info);
    let response = authorized_client("mock_user").register_output_file(request);

    assert!(response.is_ok());
}

#[test_case]
fn test_register_fusion_output() {
    let request = RegisterFusionOutputRequest::new(vec!["mock_user", "mock_user_b"]);
    let response = authorized_client("mock_user").register_fusion_output(request);
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest::new(vec!["mock_user_c", "mock_user_b"]);
    let response = authorized_client("mock_user").register_fusion_output(request);
    assert!(response.is_err());
}

#[test_case]
fn test_register_input_from_output() {
    let user1_output_id =
        ExternalID::try_from("output-00000000-0000-0000-0000-000000000001").unwrap();

    // not a owner
    let request = RegisterInputFromOutputRequest::new(user1_output_id.clone());
    let response = authorized_client("mock_user_c").register_input_from_output(request);
    assert!(response.is_err());

    // output not ready
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::default();
    let mut client = authorized_client("mock_user1");
    let request = RegisterOutputFileRequest::new(url, crypto_info);
    let response = client.register_output_file(request).unwrap();
    let request = RegisterInputFromOutputRequest::new(response.data_id);
    let response = client.register_input_from_output(request);
    assert!(response.is_err());

    let request = RegisterInputFromOutputRequest::new(user1_output_id);
    let response = client.register_input_from_output(request);
    assert!(response.is_ok());
}

#[test_case]
fn test_get_output_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let crypto_info = FileCrypto::default();
    let request = RegisterOutputFileRequest::new(url, crypto_info);

    let mut client = authorized_client("mock_user");
    let response = client.register_output_file(request).unwrap();
    let data_id = response.data_id;
    let request = GetOutputFileRequest::new(data_id.clone());
    let response = client.get_output_file(request);
    assert!(response.is_ok());

    let request = GetOutputFileRequest::new(data_id);
    let response = authorized_client("mock_another_user").get_output_file(request);
    assert!(response.is_err());
}

#[test_case]
fn test_get_input_file() {
    let url = Url::parse("https://external-storage.com/filepath?presigned_token").unwrap();
    let cmac = FileAuthTag::mock();
    let crypto_info = FileCrypto::default();

    let mut client = authorized_client("mock_user");
    let request = RegisterInputFileRequest::new(url, cmac, crypto_info);
    let response = client.register_input_file(request).unwrap();
    let data_id = response.data_id;
    let request = GetInputFileRequest::new(data_id.clone());
    let response = client.get_input_file(request);
    assert!(response.is_ok());

    let mut client = authorized_client("mock_another_user");
    let request = GetInputFileRequest::new(data_id);
    let response = client.get_input_file(request);
    assert!(response.is_err());
}

#[test_case]
fn test_register_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(true)
        .arguments(vec!["arg"])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .build();

    let mut client = authorized_client("mock_user");
    let response = client.register_function(request);

    assert!(response.is_ok());
}

#[test_case]
fn test_register_private_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(false)
        .arguments(vec!["arg"])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .user_allowlist(vec!["mock_user".to_string()])
        .build();

    let mut client = authorized_client("mock_user");
    let response = client.register_function(request);

    assert!(response.is_ok());
}

#[test_case]
fn test_delete_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(true)
        .arguments(vec!["arg"])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .build();

    let mut client = authorized_client("mock_user");
    let response = client.register_function(request);
    let function_id = response.unwrap().function_id;

    let request = DeleteFunctionRequest::new(function_id);
    let response = client.delete_function(request);
    assert!(response.is_ok());
}

#[test_case]
fn test_disable_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(true)
        .arguments(vec!["arg"])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .build();

    let mut client = authorized_client("mock_user");
    let response = client.register_function(request);
    let function_id = response.unwrap().function_id;

    let request = DeleteFunctionRequest::new(function_id);
    let response = client.disable_function(request);
    assert!(response.is_ok());
}

#[test_case]
fn test_update_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(true)
        .arguments(vec!["arg"])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .build();

    let mut client = authorized_client("mock_user");
    let response = client.register_function(request);
    let original_id = response.unwrap().function_id;

    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = UpdateFunctionRequestBuilder::new()
        .function_id(original_id.clone())
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(false)
        .arguments(vec!["arg"])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .user_allowlist(vec!["mock_user".to_string()])
        .build();

    let mut client = authorized_client("mock_user");
    let response = client.update_function(request);

    assert!(response.is_ok());
    assert!(original_id == response.unwrap().function_id);
}

#[test_case]
fn test_list_functions() {
    let request = ListFunctionsRequest {
        user_id: "mock_user".into(),
    };

    let mut client = authorized_client("mock_user");
    let response = client.list_functions(request);

    assert!(response.is_ok());
}

#[test_case]
fn test_get_function() {
    let function_input = FunctionInput::new("input", "input_desc", false);
    let function_output = FunctionOutput::new("output", "output_desc", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mock_function")
        .executor_type(ExecutorType::Python)
        .payload(b"def entrypoint:\n\treturn".to_vec())
        .public(false)
        .arguments(vec!["arg"])
        .inputs(vec![function_input])
        .outputs(vec![function_output])
        .build();

    let mut client = authorized_client("mock_user");
    let response = client.register_function(request).unwrap();
    let function_id = response.function_id;

    let request = GetFunctionRequest::new(function_id.clone());
    let response = client.get_function(request);
    assert!(response.is_ok());

    let mut client = authorized_client("mock_unauthorized_user");
    let request = GetFunctionRequest::new(function_id);
    let response = client.get_function(request);
    assert!(response.is_err());

    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000001").unwrap();
    let request = GetFunctionRequest::new(function_id);
    let response = client.get_function(request);
    assert!(response.is_ok());

    // private functions
    let function_id =
        ExternalID::try_from("function-00000000-0000-0000-0000-000000000003").unwrap();

    let mut client = authorized_client("mock_user");
    let request = GetFunctionRequest::new(function_id.clone());
    let response = client.get_function(request);
    assert!(response.is_ok());

    let mut client = authorized_client("mock_user1");
    let request = GetFunctionRequest::new(function_id.clone());
    let response = client.get_function(request);
    assert!(response.is_ok());

    let mut client = authorized_client("mock_unauthorized_user");
    let request = GetFunctionRequest::new(function_id);
    let response = client.get_function(request);
    assert!(response.is_err());
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

#[test_case]
fn test_create_task() {
    let mut client = authorized_client("mock_user");

    let request = CreateTaskRequest::new().executor(Executor::MesaPy);
    let response = client.create_task(request);
    assert!(response.is_err());

    let request = create_valid_task_request();
    let response = client.create_task(request);
    assert!(response.is_ok());

    let request = create_valid_task_request_private_function();
    let response = client.create_task(request);
    assert!(response.is_ok());

    let mut request = create_valid_task_request();
    request.function_arguments.inner_mut().remove("arg1");
    let response = client.create_task(request);
    assert!(response.is_err());

    let mut request = create_valid_task_request();
    request = request.inputs_ownership(hashmap!(
        "input2" => vec!["mock_user2", "mock_user3"]
    ));
    let response = client.create_task(request);
    assert!(response.is_err());

    let mut request = create_valid_task_request();
    request = request.outputs_ownership(hashmap!(
            "output2" => vec!["mock_user2", "mock_user3"]
    ));
    let response = client.create_task(request);
    assert!(response.is_err());

    let mut client = authorized_client("mock_user2");
    let request = create_valid_task_request_private_function();
    let response = client.create_task(request);
    // PlatformAdmin can access private function
    assert!(response.is_ok());
}

#[test_case]
fn test_get_task() {
    let mut client = authorized_client("mock_user");

    let request = create_valid_task_request();
    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    let request = GetTaskRequest::new(task_id);
    let response = client.get_task(request).unwrap();
    assert!(response.participants.len() == 4);

    let participants = vec!["mock_user1", "mock_user3", "mock_user2", "mock_user"];
    for name in participants {
        assert!(response.participants.contains(&UserID::from(name)));
    }
}

#[test_case]
fn test_assign_data() {
    let mut client = authorized_client("mock_user");
    let mut client1 = authorized_client("mock_user1");
    let mut client2 = authorized_client("mock_user2");
    let mut client3 = authorized_client("mock_user3");
    let request = create_valid_task_request();
    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    // not a participant
    let request = AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!());

    let mut unknown_client = authorized_client("non-participant");
    let response = unknown_client.assign_data(request);
    assert!(response.is_err());

    // !input_file.owner.contains(user_id)
    let url = Url::parse("https://path").unwrap();
    let cmac = FileAuthTag::mock();
    let request = RegisterInputFileRequest::new(url, cmac, FileCrypto::default());
    let response = client2.register_input_file(request).unwrap();
    let input_file_id_user2 = response.data_id;

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => input_file_id_user2.clone()),
        hashmap!(),
    );
    let response = client1.assign_data(request);
    assert!(response.is_err());

    // !output_file.owner.contains(user_id)
    let url = Url::parse("https://output_file_path").unwrap();
    let request = RegisterOutputFileRequest::new(url, FileCrypto::default());
    let response = client2.register_output_file(request).unwrap();
    let output_file_id_user2 = response.data_id;

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output" => output_file_id_user2.clone()),
    );
    let response = client1.assign_data(request);
    assert!(response.is_err());

    let existing_outfile_id_user1 =
        ExternalID::try_from("output-00000000-0000-0000-0000-000000000001").unwrap();

    // output_file.cmac.is_some()
    let request = GetOutputFileRequest::new(existing_outfile_id_user1.clone());
    client1.get_output_file(request).unwrap();

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output" => existing_outfile_id_user1),
    );
    let response = client1.assign_data(request);
    assert!(response.is_err());

    // !fusion_data.owner_id_list.contains(user_id)
    let file_id2 = ExternalID::try_from("input-00000000-0000-0000-0000-000000000002").unwrap();
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => file_id2.clone()),
        hashmap!(),
    );
    let response = client1.assign_data(request);
    assert!(response.is_err());

    // inputs_ownership doesn't contain the name
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("none" => input_file_id_user2.clone()),
        hashmap!(),
    );
    let response = client2.assign_data(request);
    assert!(response.is_err());

    // outputs_ownership doesn't contain the name
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("none" => output_file_id_user2.clone()),
    );
    let response = client2.assign_data(request);
    assert!(response.is_err());

    //input file: OwnerList != user_id
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input2" => input_file_id_user2.clone()),
        hashmap!(),
    );
    let response = client2.assign_data(request);
    assert!(response.is_err());

    // input file: OwnerList != user_id
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => input_file_id_user2),
        hashmap!(),
    );
    let response = client2.assign_data(request);
    assert!(response.is_err());

    // output file OwnerList != uids
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output2" => output_file_id_user2.clone()),
    );
    let response = client2.assign_data(request);
    assert!(response.is_err());

    // output file: OwnerList != uids
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output1" => output_file_id_user2),
    );
    let response = client2.assign_data(request);
    assert!(response.is_err());

    // assign all the data
    let url = Url::parse("input://path").unwrap();
    let cmac = FileAuthTag::mock();
    let request = RegisterInputFileRequest::new(url, cmac, FileCrypto::default());
    let response = client1.register_input_file(request);
    let input_file_id_user1 = response.unwrap().data_id;

    let url = Url::parse("https://output_file_path").unwrap();
    let request = RegisterOutputFileRequest::new(url, FileCrypto::default());
    let response = client1.register_output_file(request);
    let output_file_id_user1 = response.unwrap().data_id;

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => input_file_id_user1.clone()),
        hashmap!("output" => output_file_id_user1),
    );
    let response = client1.assign_data(request);
    assert!(response.is_ok());

    let request =
        AssignDataRequest::new(task_id.clone(), hashmap!("input2" => file_id2), hashmap!());
    let response = client3.assign_data(request);
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest::new(vec!["mock_user2", "mock_user3"]);
    let response = client3.register_fusion_output(request).unwrap();
    let fusion_output = response.data_id;
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output2" => fusion_output),
    );
    let response = client3.assign_data(request);
    assert!(response.is_ok());

    let request = GetTaskRequest::new(task_id.clone());
    let response = client3.get_task(request).unwrap();
    assert_eq!(response.status, TaskStatus::DataAssigned);

    // task.status != Created
    let request = AssignDataRequest::new(
        task_id,
        hashmap!("input" => input_file_id_user1),
        hashmap!(),
    );
    let response = client1.assign_data(request);
    assert!(response.is_err());
}

#[test_case]
fn test_approve_task() {
    let mut client = authorized_client("mock_user");
    let mut client1 = authorized_client("mock_user1");
    let mut client2 = authorized_client("mock_user2");
    let mut client3 = authorized_client("mock_user3");
    let request = create_valid_task_request();
    let response = client.create_task(request).unwrap();
    let task_id = response.task_id;

    // task_status != ready
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client1.approve_task(request);
    assert!(response.is_err());

    // assign all the data
    let url = Url::parse("input://path").unwrap();
    let cmac = FileAuthTag::mock();
    let request = RegisterInputFileRequest::new(url, cmac, FileCrypto::default());
    let response = client1.register_input_file(request).unwrap();

    let input_file_id_user1 = response.data_id;
    let url = Url::parse("https://output_file_path").unwrap();
    let request = RegisterOutputFileRequest::new(url, FileCrypto::default());
    let response = client1.register_output_file(request).unwrap();

    let output_file_id_user1 = response.data_id;
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => input_file_id_user1),
        hashmap!("output" => output_file_id_user1),
    );
    let response = client1.assign_data(request);
    assert!(response.is_ok());

    let input_file_id_user2 =
        ExternalID::try_from("input-00000000-0000-0000-0000-000000000002").unwrap();
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input2" => input_file_id_user2),
        hashmap!(),
    );
    let response = client2.assign_data(request);
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest::new(vec!["mock_user2", "mock_user3"]);
    let response = client3.register_fusion_output(request);
    let fusion_output = response.unwrap().data_id;
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output2" => fusion_output),
    );
    let response = client3.assign_data(request);
    assert!(response.is_ok());

    let request = GetTaskRequest::new(task_id.clone());
    let response = client2.get_task(request).unwrap();
    assert_eq!(response.status, TaskStatus::DataAssigned);

    // user_id not in task.participants
    let mut unknown_client = authorized_client("non-participant");
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = unknown_client.approve_task(request);
    assert!(response.is_err());

    //all participants approve the task
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client.approve_task(request);
    assert!(response.is_ok());
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client1.approve_task(request);
    assert!(response.is_ok());
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client2.approve_task(request);
    assert!(response.is_ok());
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client3.approve_task(request);
    assert!(response.is_ok());
    let request = GetTaskRequest::new(task_id);
    let response = client2.get_task(request).unwrap();
    assert_eq!(response.status, TaskStatus::Approved);
}

#[test_case]
fn test_invoke_task() {
    let mut client = authorized_client("mock_user");
    let mut client1 = authorized_client("mock_user1");
    let mut client2 = authorized_client("mock_user2");
    let mut client3 = authorized_client("mock_user3");
    let request = create_valid_task_request();
    let response = client.create_task(request);
    assert!(response.is_ok());
    let task_id = response.unwrap().task_id;

    // assign all the data
    let url = Url::parse("input://path").unwrap();
    let cmac = FileAuthTag::mock();
    let request = RegisterInputFileRequest::new(url, cmac, FileCrypto::default());
    let response = client1.register_input_file(request).unwrap();

    let input_file_id_user1 = response.data_id;

    let url = Url::parse("https://output_file_path").unwrap();
    let request = RegisterOutputFileRequest::new(url, FileCrypto::default());
    let response = client1.register_output_file(request).unwrap();
    let output_file_id_user1 = response.data_id;

    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input" => input_file_id_user1),
        hashmap!("output" => output_file_id_user1),
    );
    client1.assign_data(request).unwrap();

    let input_file_id_user2 =
        ExternalID::try_from("input-00000000-0000-0000-0000-000000000002").unwrap();
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("input2" => input_file_id_user2),
        hashmap!(),
    );
    let response = client2.assign_data(request);
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest::new(vec!["mock_user2", "mock_user3"]);
    let response = client3.register_fusion_output(request);
    let fusion_output = response.unwrap().data_id;
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!(),
        hashmap!("output2" => fusion_output),
    );
    let response = client3.assign_data(request);
    assert!(response.is_ok());

    // task status != Approved
    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client.invoke_task(request);
    assert!(response.is_err());

    //all participants approve the task
    let request = ApproveTaskRequest::new(task_id.clone());
    client.approve_task(request).unwrap();
    let request = ApproveTaskRequest::new(task_id.clone());
    client1.approve_task(request).unwrap();
    let request = ApproveTaskRequest::new(task_id.clone());
    client2.approve_task(request).unwrap();
    let request = ApproveTaskRequest::new(task_id.clone());
    client3.approve_task(request).unwrap();
    let request = GetTaskRequest::new(task_id.clone());
    let response = client2.get_task(request).unwrap();
    assert_eq!(response.status, TaskStatus::Approved);

    // user_id != task.creator
    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client2.invoke_task(request);
    assert!(response.is_err());

    // invoke task
    let request = InvokeTaskRequest::new(task_id.clone());
    client.invoke_task(request).unwrap();

    let request = GetTaskRequest::new(task_id);
    let response = client2.get_task(request).unwrap();
    assert_eq!(response.status, TaskStatus::Staged);

    let mut scheduler_client = get_scheduler_client();
    let executor_id = Uuid::new_v4();

    std::thread::sleep(std::time::Duration::from_secs(2));

    let pull_task_request = PullTaskRequest { executor_id };
    let response = scheduler_client.pull_task(pull_task_request);
    assert!(response.is_ok());
}
