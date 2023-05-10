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
use futures::FutureExt;
use teaclave_crypto::TeaclaveFile128Key;
use teaclave_rpc::CredentialService;
use teaclave_test_utils::async_test_case;
#[async_test_case]
pub async fn test_gbdt_training_task() {
    let mut client = authorized_frontend_client().await;
    let function_id = register_gbdt_function(&mut client).await;
    let training_data_id = register_input_file(&mut client).await;

    let crypto = TeaclaveFile128Key::random();
    let output_model_id = register_output_file(&mut client, crypto).await;

    let task_id = create_gbdt_training_task(&mut client, &function_id).await;
    assign_data_to_task(&mut client, &task_id, training_data_id, output_model_id).await;
    approve_task(&mut client, &task_id).await.unwrap();
    invoke_task(&mut client, &task_id).await.unwrap();

    let ret_val = get_task_until(&mut client, &task_id, TaskStatus::Finished).await;
    assert_eq!(&ret_val, "Trained 120 lines of data.");
}

// Authenticate user before talking to frontend service
async fn authorized_frontend_client() -> TeaclaveFrontendClient<CredentialService> {
    let mut api_client = create_authentication_api_client(shared_enclave_info(), AUTH_SERVICE_ADDR)
        .await
        .unwrap();
    let cred = login(&mut api_client, USERNAME, TEST_PASSWORD)
        .await
        .unwrap();
    create_frontend_client(shared_enclave_info(), FRONTEND_SERVICE_ADDR, cred)
        .await
        .unwrap()
}

async fn register_gbdt_function(
    client: &mut TeaclaveFrontendClient<CredentialService>,
) -> ExternalID {
    let fn_input = FunctionInput::new("training_data", "Input traning data file.", false);
    let fn_output = FunctionOutput::new("trained_model", "Output trained model.", false);
    let fn_args = vec![
        FunctionArgument::new("feature_size", "", true),
        FunctionArgument::new("max_depth", "", true),
        FunctionArgument::new("iterations", "", true),
        FunctionArgument::new("shrinkage", "", true),
        FunctionArgument::new("feature_sample_ratio", "", true),
        FunctionArgument::new("data_sample_ratio", "", true),
        FunctionArgument::new("min_leaf_size", "", true),
        FunctionArgument::new("loss", "", true),
        FunctionArgument::new("training_optimization_level", "", true),
    ];

    // Register Function
    let request = RegisterFunctionRequestBuilder::new()
        .name("builtin-gbdt-train")
        .description("Native Gbdt Training Function")
        .arguments(fn_args)
        .inputs(vec![fn_input])
        .outputs(vec![fn_output])
        .build();
    let response = client
        .register_function(request)
        .await
        .unwrap()
        .into_inner();
    log::debug!("Register function: {:?}", response);
    response.function_id.try_into().unwrap()
}

async fn register_input_file(client: &mut TeaclaveFrontendClient<CredentialService>) -> ExternalID {
    let url =
        Url::parse("http://localhost:6789/fixtures/functions/gbdt_training/train.enc").unwrap();
    let crypto = TeaclaveFile128Key::new(&[0; 16]).unwrap();
    let crypto_info = FileCrypto::TeaclaveFile128(crypto);
    let cmac = FileAuthTag::from_hex("860030495909b84864b991865e9ad94f").unwrap();

    let request = RegisterInputFileRequest::new(url, cmac, crypto_info);
    let response = client
        .register_input_file(request)
        .await
        .unwrap()
        .into_inner();
    log::debug!("Register input: {:?}", response);
    response.data_id.try_into().unwrap()
}

async fn register_output_file(
    client: &mut TeaclaveFrontendClient<CredentialService>,
    crypto: impl Into<FileCrypto>,
) -> ExternalID {
    let url =
        Url::parse("http://localhost:6789/fixtures/functions/gbdt_training/e2e_output_model.enc")
            .unwrap();
    let request = RegisterOutputFileRequest::new(url, crypto);
    let response = client
        .register_output_file(request)
        .await
        .unwrap()
        .into_inner();
    log::debug!("Register output: {:?}", response);
    response.data_id.try_into().unwrap()
}

async fn create_gbdt_training_task(
    client: &mut TeaclaveFrontendClient<CredentialService>,
    function_id: &ExternalID,
) -> ExternalID {
    let arguments = FunctionArguments::from_json(serde_json::json!({
        "feature_size": 4,
        "max_depth": 4,
        "iterations": 100,
        "shrinkage": 0.1,
        "feature_sample_ratio": 1.0,
        "data_sample_ratio": 1.0,
        "min_leaf_size": 1,
        "loss": "LAD",
        "training_optimization_level": 2
    }))
    .unwrap();
    let request = CreateTaskRequest::new()
        .executor(Executor::Builtin)
        .function_id(function_id.clone())
        .function_arguments(arguments)
        .inputs_ownership(hashmap!("training_data" => vec![USERNAME]))
        .outputs_ownership(hashmap!("trained_model" => vec![USERNAME]));

    let response = client.create_task(request).await.unwrap().into_inner();
    log::debug!("Create task: {:?}", response);

    response.task_id.try_into().unwrap()
}

async fn assign_data_to_task(
    client: &mut TeaclaveFrontendClient<CredentialService>,
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
    let response = client.assign_data(request).await.unwrap();

    log::debug!("Assign data: {:?}", response);
}
