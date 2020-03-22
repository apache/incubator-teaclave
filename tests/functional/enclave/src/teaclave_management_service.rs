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

use std::collections::HashMap;
use std::prelude::v1::*;
use teaclave_attestation::verifier;
use teaclave_config::RuntimeConfig;
use teaclave_config::BUILD_CONFIG;
use teaclave_proto::teaclave_management_service::*;
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
        test_get_input_file,
        test_get_output_file,
        test_register_function,
        test_get_function,
        test_create_task,
        test_get_task,
        test_assign_data,
        test_approve_task,
        test_invoke_task,
    )
}

fn get_client(user_id: &str) -> TeaclaveManagementClient {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let enclave_info =
        EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes.as_ref().unwrap());
    let enclave_attr = enclave_info
        .get_enclave_attr("teaclave_management_service")
        .expect("management");
    let config = SgxTrustedTlsClientConfig::new().attestation_report_verifier(
        vec![enclave_attr],
        BUILD_CONFIG.as_root_ca_cert,
        verifier::universal_quote_verifier,
    );

    let channel = Endpoint::new(
        &runtime_config
            .internal_endpoints
            .management
            .advertised_address,
    )
    .config(config)
    .connect()
    .unwrap();

    let mut metadata = HashMap::new();
    metadata.insert("id".to_string(), user_id.to_string());
    metadata.insert("token".to_string(), "".to_string());

    TeaclaveManagementClient::new_with_metadata(channel, metadata).unwrap()
}

fn test_register_input_file() {
    let url = Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap();
    let crypto_info =
        TeaclaveFileCryptoInfo::new("aes_gcm_128", &[0x90u8; 16], &[0x89u8; 12]).unwrap();
    let request = RegisterInputFileRequest::new(url, "deadbeefdeadbeef", crypto_info);

    let mut client = get_client("mock_user");
    let response = client.register_input_file(request);

    assert!(response.is_ok());
}

fn test_register_output_file() {
    let crypto_info =
        TeaclaveFileCryptoInfo::new("aes_gcm_128", &[0x90u8; 16], &[0x89u8; 12]).unwrap();
    let url = Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap();
    let request = RegisterOutputFileRequest::new(url, crypto_info);

    let mut client = get_client("mock_user");
    let response = client.register_output_file(request);

    assert!(response.is_ok());
}

fn test_register_fusion_output() {
    let request = RegisterFusionOutputRequest {
        owner_list: vec!["mock_user", "mock_user_b"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    };

    let mut client = get_client("mock_user");
    let response = client.register_fusion_output(request);

    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest {
        owner_list: vec!["mock_user_c", "mock_user_b"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    };

    let mut client = get_client("mock_user");
    let response = client.register_fusion_output(request);
    assert!(response.is_err());
}

fn test_register_input_from_output() {
    // not a owner
    let mut client = get_client("mock_user_c");
    let data_id = "output-file-00000000-0000-0000-0000-000000000001";
    let request = RegisterInputFromOutputRequest::new(data_id);
    let response = client.register_input_from_output(request);
    assert!(response.is_err());

    // output not ready
    let url = Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap();
    let crypto_info = TeaclaveFileCryptoInfo::default();
    let mut client = get_client("mock_user1");
    let request = RegisterOutputFileRequest::new(url, crypto_info);
    let response = client.register_output_file(request);
    assert!(response.is_ok());
    let request = RegisterInputFromOutputRequest::new(response.unwrap().data_id);
    let response = client.register_input_from_output(request);
    assert!(response.is_err());

    let request = RegisterInputFromOutputRequest::new(data_id);
    let response = client.register_input_from_output(request);
    assert!(response.is_ok());
    info!("{:?}", response);
}

fn test_get_output_file() {
    let url = Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap();
    let crypto_info = TeaclaveFileCryptoInfo::default();
    let request = RegisterOutputFileRequest::new(url, crypto_info);

    let mut client = get_client("mock_user");
    let response = client.register_output_file(request).unwrap();
    let data_id = response.data_id;
    let request = GetOutputFileRequest::new(&data_id);
    let response = client.get_output_file(request);
    assert!(response.is_ok());
    let mut client = get_client("mock_another_user");
    let request = GetOutputFileRequest::new(&data_id);
    let response = client.get_output_file(request);
    assert!(response.is_err());
}

fn test_get_input_file() {
    let url = Url::parse("s3://s3.us-west-2.amazonaws.com/mybucket/puppy.jpg.enc?key-id=deadbeefdeadbeef&key=deadbeefdeadbeef").unwrap();
    let crypto_info = TeaclaveFileCryptoInfo::default();
    let request = RegisterInputFileRequest::new(url, "deadbeef", crypto_info);

    let mut client = get_client("mock_user");
    let response = client.register_input_file(request).unwrap();
    let data_id = response.data_id;
    let request = GetInputFileRequest::new(&data_id);
    let response = client.get_input_file(request);
    assert!(response.is_ok());
    let mut client = get_client("mock_another_user");
    let request = GetInputFileRequest::new(&data_id);
    let response = client.get_input_file(request);
    assert!(response.is_err());
}

fn test_register_function() {
    let function_input = FunctionInput::new("input", "input_desc");
    let function_output = FunctionOutput::new("output", "output_desc");
    let request = RegisterFunctionRequest {
        name: "mock_function".to_string(),
        description: "mock function".to_string(),
        payload: b"python script".to_vec(),
        is_public: true,
        arg_list: vec!["arg".to_string()],
        input_list: vec![function_input],
        output_list: vec![function_output],
    };

    let mut client = get_client("mock_user");
    let response = client.register_function(request);

    assert!(response.is_ok());
}

fn test_get_function() {
    let function_input = FunctionInput::new("input", "input_desc");
    let function_output = FunctionOutput::new("output", "output_desc");
    let request = RegisterFunctionRequest {
        name: "mock_function".to_string(),
        description: "mock function".to_string(),
        payload: b"python script".to_vec(),
        is_public: false,
        arg_list: vec!["arg".to_string()],
        input_list: vec![function_input],
        output_list: vec![function_output],
    };

    let mut client = get_client("mock_user");
    let response = client.register_function(request);
    let function_id = response.unwrap().function_id;

    let request = GetFunctionRequest {
        function_id: function_id.clone(),
    };
    let response = client.get_function(request);
    assert!(response.is_ok());
    info!("{:?}", response.unwrap());
    let mut client = get_client("mock_unauthorized_user");
    let request = GetFunctionRequest::new(function_id);
    let response = client.get_function(request);
    assert!(response.is_err());
    let request = GetFunctionRequest::new("function-00000000-0000-0000-0000-000000000001");
    let response = client.get_function(request);
    assert!(response.is_ok());
    info!("{:?}", response.unwrap());
}

fn get_correct_create_task() -> CreateTaskRequest {
    let function_arguments = FunctionArguments::new(hashmap!(
        "arg1" => "data1",
        "arg2" => "data2",
    ));
    let data_owner_id_list = DataOwnerList {
        user_id_list: vec!["mock_user1".to_string()].into_iter().collect(),
    };
    let data_owner_id_list2 = DataOwnerList {
        user_id_list: vec!["mock_user2".to_string(), "mock_user3".to_string()]
            .into_iter()
            .collect(),
    };
    let mut input_data_owner_list = HashMap::new();
    input_data_owner_list.insert("input".to_string(), data_owner_id_list.clone());
    input_data_owner_list.insert("input2".to_string(), data_owner_id_list2.clone());
    let mut output_data_owner_list = HashMap::new();
    output_data_owner_list.insert("output".to_string(), data_owner_id_list);
    output_data_owner_list.insert("output2".to_string(), data_owner_id_list2);

    CreateTaskRequest {
        function_id: "function-00000000-0000-0000-0000-000000000001".to_string(),
        function_arguments,
        input_data_owner_list,
        output_data_owner_list,
    }
}

fn test_create_task() {
    let request = CreateTaskRequest {
        function_id: "invalid_function".to_string(),
        function_arguments: HashMap::new().into(),
        input_data_owner_list: HashMap::new(),
        output_data_owner_list: HashMap::new(),
    };
    let mut client = get_client("mock_user");
    let response = client.create_task(request);
    assert!(response.is_err());

    let request = get_correct_create_task();
    let response = client.create_task(request);
    assert!(response.is_ok());

    let mut request = get_correct_create_task();
    request.function_arguments.inner_mut().remove("arg1");
    let response = client.create_task(request);
    assert!(response.is_err());

    let mut request = get_correct_create_task();
    request.input_data_owner_list.remove("input");
    let response = client.create_task(request);
    assert!(response.is_err());

    let mut request = get_correct_create_task();
    request.output_data_owner_list.remove("output");
    let response = client.create_task(request);
    assert!(response.is_err());
}

fn test_get_task() {
    let mut client = get_client("mock_user");
    let request = get_correct_create_task();
    let response = client.create_task(request);
    assert!(response.is_ok());
    let task_id = response.unwrap().task_id;

    let request = GetTaskRequest { task_id };
    let response = client.get_task(request);
    assert!(response.is_ok());
    let response = response.unwrap();
    info!("{:?}", response);
    let participants = vec!["mock_user1", "mock_user3", "mock_user2", "mock_user"];
    for name in participants {
        assert!(response.participants.contains(&name.to_string()));
    }
    assert!(response.participants.len() == 4);
}

fn test_assign_data() {
    let mut client = get_client("mock_user");
    let mut client1 = get_client("mock_user1");
    let mut client2 = get_client("mock_user2");
    let mut client3 = get_client("mock_user3");
    let request = get_correct_create_task();
    let response = client.create_task(request);
    assert!(response.is_ok());
    let task_id = response.unwrap().task_id;

    // not a participant
    let request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    let mut unknown_client = get_client("non-participant");
    let response = unknown_client.assign_data(request);
    assert!(response.is_err());

    // !input_file.owner.contains(user_id)
    let request = RegisterInputFileRequest {
        url: Url::parse("input://path").unwrap(),
        hash: "deadbeefdeadbeef".to_string(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client2.register_input_file(request);
    let input_file_id_user2 = response.unwrap().data_id;

    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .input_map
        .insert("input".to_string(), input_file_id_user2.clone());
    let response = client1.assign_data(request);
    assert!(response.is_err());

    // !output_file.owner.contains(user_id)
    let request = RegisterOutputFileRequest {
        url: Url::parse("output://path").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client2.register_output_file(request);
    let output_file_id_user2 = response.unwrap().data_id;

    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .output_map
        .insert("output".to_string(), output_file_id_user2.clone());
    let response = client1.assign_data(request);
    assert!(response.is_err());

    let existing_outfile_id_user1 = "output-file-00000000-0000-0000-0000-000000000001".to_string();

    // output_file.hash.is_some()
    let request = GetOutputFileRequest {
        data_id: existing_outfile_id_user1.clone(),
    };
    let response = client1.get_output_file(request);
    assert!(response.is_ok());
    assert!(!response.unwrap().hash.is_empty());
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .output_map
        .insert("output".to_string(), existing_outfile_id_user1);
    let response = client1.assign_data(request);
    assert!(response.is_err());

    // !fusion_data.owner_id_list.contains(user_id)
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request.input_map.insert(
        "input".to_string(),
        "input-file-00000000-0000-0000-0000-000000000002".to_string(),
    );
    let response = client1.assign_data(request);
    assert!(response.is_err());

    // input_data_owner_list doesn't contain the name
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .input_map
        .insert("none".to_string(), input_file_id_user2.clone());
    let response = client2.assign_data(request);
    assert!(response.is_err());

    // output_data_owner_list doesn't contain the name
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .output_map
        .insert("none".to_string(), output_file_id_user2.clone());
    let response = client2.assign_data(request);
    assert!(response.is_err());

    //input file: DataOwnerList != user_id
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .input_map
        .insert("input2".to_string(), input_file_id_user2.clone());
    let response = client2.assign_data(request);
    assert!(response.is_err());

    // input file: DataOwnerList != user_id
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .input_map
        .insert("input".to_string(), input_file_id_user2);
    let response = client2.assign_data(request);
    assert!(response.is_err());

    // output file DataOwnerList != user_id_list
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .output_map
        .insert("output2".to_string(), output_file_id_user2.clone());
    let response = client2.assign_data(request);
    assert!(response.is_err());

    // output file: DataOwnerList != user_id_list
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .output_map
        .insert("output1".to_string(), output_file_id_user2);
    let response = client2.assign_data(request);
    assert!(response.is_err());

    // assign all the data
    let request = RegisterInputFileRequest {
        url: Url::parse("input://path").unwrap(),
        hash: "deadbeefdeadbeef".to_string(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client1.register_input_file(request);
    let input_file_id_user1 = response.unwrap().data_id;

    let request = RegisterOutputFileRequest {
        url: Url::parse("input://path").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client1.register_output_file(request);
    let output_file_id_user1 = response.unwrap().data_id;

    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .input_map
        .insert("input".to_string(), input_file_id_user1.clone());
    request
        .output_map
        .insert("output".to_string(), output_file_id_user1);
    let response = client1.assign_data(request);
    assert!(response.is_ok());

    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request.input_map.insert(
        "input2".to_string(),
        "input-file-00000000-0000-0000-0000-000000000002".to_string(),
    );
    let response = client3.assign_data(request);
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest {
        owner_list: vec!["mock_user2", "mock_user3"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    };
    let response = client3.register_fusion_output(request);
    let fusion_output = response.unwrap().data_id;
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .output_map
        .insert("output2".to_string(), fusion_output);
    let response = client3.assign_data(request);
    assert!(response.is_ok());

    let request = GetTaskRequest {
        task_id: task_id.clone(),
    };
    let response = client3.get_task(request);
    assert_eq!(response.unwrap().status, TaskStatus::Ready);

    // task.status != Created
    let mut request = AssignDataRequest {
        task_id,
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .input_map
        .insert("input".to_string(), input_file_id_user1);
    let response = client1.assign_data(request);
    assert!(response.is_err());
}

fn test_approve_task() {
    let mut client = get_client("mock_user");
    let mut client1 = get_client("mock_user1");
    let mut client2 = get_client("mock_user2");
    let mut client3 = get_client("mock_user3");
    let request = get_correct_create_task();
    let response = client.create_task(request);
    assert!(response.is_ok());
    let task_id = response.unwrap().task_id;

    // task_status != ready
    let request = ApproveTaskRequest {
        task_id: task_id.clone(),
    };
    let response = client1.approve_task(request);
    assert!(response.is_err());

    // assign all the data
    let request = RegisterInputFileRequest {
        url: Url::parse("input://path").unwrap(),
        hash: "deadbeefdeadbeef".to_string(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client1.register_input_file(request);
    let input_file_id_user1 = response.unwrap().data_id;
    let request = RegisterOutputFileRequest {
        url: Url::parse("input://path").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client1.register_output_file(request);
    let output_file_id_user1 = response.unwrap().data_id;
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .input_map
        .insert("input".to_string(), input_file_id_user1);
    request
        .output_map
        .insert("output".to_string(), output_file_id_user1);
    let response = client1.assign_data(request);
    assert!(response.is_ok());

    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request.input_map.insert(
        "input2".to_string(),
        "input-file-00000000-0000-0000-0000-000000000002".to_string(),
    );
    let response = client2.assign_data(request);
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest {
        owner_list: vec!["mock_user2", "mock_user3"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    };
    let response = client3.register_fusion_output(request);
    let fusion_output = response.unwrap().data_id;
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .output_map
        .insert("output2".to_string(), fusion_output);
    let response = client3.assign_data(request);
    assert!(response.is_ok());

    let request = GetTaskRequest::new(&task_id);
    let response = client2.get_task(request);
    assert_eq!(response.unwrap().status, TaskStatus::Ready);

    // user_id not in task.participants
    let mut unknown_client = get_client("non-participant");
    let request = ApproveTaskRequest::new(&task_id);
    let response = unknown_client.approve_task(request);
    assert!(response.is_err());

    //all participants approve the task
    let request = ApproveTaskRequest::new(&task_id);
    let response = client.approve_task(request);
    assert!(response.is_ok());
    let request = ApproveTaskRequest::new(&task_id);
    let response = client1.approve_task(request);
    assert!(response.is_ok());
    let request = ApproveTaskRequest::new(&task_id);
    let response = client2.approve_task(request);
    assert!(response.is_ok());
    let request = ApproveTaskRequest::new(&task_id);
    let response = client3.approve_task(request);
    assert!(response.is_ok());
    let request = GetTaskRequest::new(&task_id);
    let response = client2.get_task(request);
    assert_eq!(response.unwrap().status, TaskStatus::Approved);
}

fn test_invoke_task() {
    let mut client = get_client("mock_user");
    let mut client1 = get_client("mock_user1");
    let mut client2 = get_client("mock_user2");
    let mut client3 = get_client("mock_user3");
    let request = get_correct_create_task();
    let response = client.create_task(request);
    assert!(response.is_ok());
    let task_id = response.unwrap().task_id;

    // assign all the data
    let request = RegisterInputFileRequest {
        url: Url::parse("input://path").unwrap(),
        hash: "deadbeefdeadbeef".to_string(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client1.register_input_file(request);
    let input_file_id_user1 = response.unwrap().data_id;
    let request = RegisterOutputFileRequest {
        url: Url::parse("input://path").unwrap(),
        crypto_info: TeaclaveFileCryptoInfo::default(),
    };
    let response = client1.register_output_file(request);
    let output_file_id_user1 = response.unwrap().data_id;
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .input_map
        .insert("input".to_string(), input_file_id_user1);
    request
        .output_map
        .insert("output".to_string(), output_file_id_user1);
    client1.assign_data(request).unwrap();
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request.input_map.insert(
        "input2".to_string(),
        "input-file-00000000-0000-0000-0000-000000000002".to_string(),
    );
    let response = client2.assign_data(request);
    assert!(response.is_ok());

    let request = RegisterFusionOutputRequest {
        owner_list: vec!["mock_user2", "mock_user3"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    };
    let response = client3.register_fusion_output(request);
    let fusion_output = response.unwrap().data_id;
    let mut request = AssignDataRequest {
        task_id: task_id.clone(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
    };
    request
        .output_map
        .insert("output2".to_string(), fusion_output);
    let response = client3.assign_data(request);
    assert!(response.is_ok());

    // task status != Approved
    let request = InvokeTaskRequest::new(&task_id);
    let response = client.invoke_task(request);
    assert!(response.is_err());

    //all participants approve the task
    let request = ApproveTaskRequest::new(&task_id);
    client.approve_task(request).unwrap();
    let request = ApproveTaskRequest::new(&task_id);
    client1.approve_task(request).unwrap();
    let request = ApproveTaskRequest::new(&task_id);
    client2.approve_task(request).unwrap();
    let request = ApproveTaskRequest::new(&task_id);
    client3.approve_task(request).unwrap();
    let request = GetTaskRequest::new(&task_id);
    let response = client2.get_task(request).unwrap();
    assert_eq!(response.status, TaskStatus::Approved);

    // user_id != task.creator
    let request = InvokeTaskRequest::new(&task_id);
    let response = client2.invoke_task(request);
    assert!(response.is_err());

    // invoke task
    let request = InvokeTaskRequest::new(&task_id);
    client.invoke_task(request).unwrap();

    let request = GetTaskRequest::new(&task_id);
    let response = client2.get_task(request).unwrap();
    assert_eq!(response.status, TaskStatus::Running);
}
