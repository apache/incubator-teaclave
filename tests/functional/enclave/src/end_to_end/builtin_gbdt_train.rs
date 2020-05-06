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

use super::*;
use teaclave_crypto::TeaclaveFile128Key;
use teaclave_test_utils::test_case;

#[test_case]
pub fn test_gbdt_training_task() {
    let mut client = authorized_frontend_client();
    let function_id = register_gbdt_function(&mut client);
    let training_data_id = register_input_file(&mut client);

    let crypto = TeaclaveFile128Key::random();
    let output_model_id = register_output_file(&mut client, crypto);

    let task_id = create_gbdt_training_task(&mut client, &function_id);
    assign_data_to_task(&mut client, &task_id, training_data_id, output_model_id);
    approve_task(&mut client, &task_id).unwrap();
    invoke_task(&mut client, &task_id).unwrap();

    let ret_val = get_task_until(&mut client, &task_id, TaskStatus::Finished);
    assert_eq!(&ret_val, "Trained 120 lines of data.");
}

// Authenticate user before talking to frontend service
fn authorized_frontend_client() -> TeaclaveFrontendClient {
    let mut api_client =
        create_authentication_api_client(shared_enclave_info(), AUTH_SERVICE_ADDR).unwrap();
    let cred = login(&mut api_client, USERNAME, TEST_PASSWORD).unwrap();
    create_frontend_client(shared_enclave_info(), FRONTEND_SERVICE_ADDR, cred).unwrap()
}

fn register_gbdt_function(client: &mut TeaclaveFrontendClient) -> ExternalID {
    let fn_input = FunctionInput::new("training_data", "Input traning data file.");
    let fn_output = FunctionOutput::new("trained_model", "Output trained model.");
    let fn_args = vec![
        "feature_size",
        "max_depth",
        "iterations",
        "shrinkage",
        "feature_sample_ratio",
        "data_sample_ratio",
        "min_leaf_size",
        "loss",
        "training_optimization_level",
    ];

    // Register Function
    let request = RegisterFunctionRequest::new()
        .name("builtin-gbdt-train")
        .description("Native Gbdt Training Function")
        .arguments(fn_args)
        .inputs(vec![fn_input])
        .outputs(vec![fn_output]);

    let response = client.register_function(request).unwrap();
    log::info!("Register function: {:?}", response);
    response.function_id
}

fn register_input_file(client: &mut TeaclaveFrontendClient) -> ExternalID {
    let url =
        Url::parse("http://localhost:6789/fixtures/functions/gbdt_training/train.enc").unwrap();
    let crypto = TeaclaveFile128Key::new(&[0; 16]).unwrap();
    let crypto_info = FileCrypto::TeaclaveFile128(crypto);
    let cmac = FileAuthTag::from_hex("881adca6b0524472da0a9d0bb02b9af9").unwrap();

    let request = RegisterInputFileRequest::new(url, cmac, crypto_info);
    let response = client.register_input_file(request).unwrap();
    log::info!("Register input: {:?}", response);
    response.data_id
}

fn register_output_file(
    client: &mut TeaclaveFrontendClient,
    crypto: impl Into<FileCrypto>,
) -> ExternalID {
    let url =
        Url::parse("http://localhost:6789/fixtures/functions/gbdt_training/e2e_output_model.enc")
            .unwrap();
    let request = RegisterOutputFileRequest::new(url, crypto);
    let response = client.register_output_file(request).unwrap();
    log::info!("Register output: {:?}", response);
    response.data_id
}

fn create_gbdt_training_task(
    client: &mut TeaclaveFrontendClient,
    function_id: &ExternalID,
) -> ExternalID {
    let request = CreateTaskRequest::new()
        .executor(Executor::Builtin)
        .function_id(function_id.clone())
        .function_arguments(hashmap!(
            "feature_size"          => "4",
            "max_depth"             => "4",
            "iterations"            => "100",
            "shrinkage"             => "0.1",
            "feature_sample_ratio"  => "1.0",
            "data_sample_ratio"     => "1.0",
            "min_leaf_size"         => "1",
            "loss"                  => "LAD",
            "training_optimization_level" => "2"
        ))
        .inputs_ownership(hashmap!("training_data" => vec![USERNAME]))
        .outputs_ownership(hashmap!("trained_model" => vec![USERNAME]));

    let response = client.create_task(request).unwrap();
    log::info!("Create task: {:?}", response);

    response.task_id
}

fn assign_data_to_task(
    client: &mut TeaclaveFrontendClient,
    task_id: &ExternalID,
    training_data_id: ExternalID,
    out_model_id: ExternalID,
) {
    // Assign Data To Task
    let request = AssignDataRequest::new(
        task_id.clone(),
        hashmap!("training_data" => training_data_id),
        hashmap!("trained_model" => out_model_id),
    );
    let response = client.assign_data(request).unwrap();

    log::info!("Assign data: {:?}", response);
}
