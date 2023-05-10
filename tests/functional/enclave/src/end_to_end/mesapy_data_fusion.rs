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
use std::collections::HashMap;
use teaclave_crypto::{AesGcm128Key, AesGcm256Key};
use teaclave_rpc::CredentialService;
use teaclave_test_utils::async_test_case;
type FrontendClient = TeaclaveFrontendClient<CredentialService>;

async fn setup_client() -> anyhow::Result<(FrontendClient, FrontendClient)> {
    // Authenticate user before talking to frontend service
    let mut api_client =
        create_authentication_api_client(shared_enclave_info(), AUTH_SERVICE_ADDR).await?;
    let cred = login(&mut api_client, USERNAME1, TEST_PASSWORD).await?;
    let client1 =
        create_frontend_client(shared_enclave_info(), FRONTEND_SERVICE_ADDR, cred).await?;
    let cred = login(&mut api_client, USERNAME2, TEST_PASSWORD).await?;
    let client2 =
        create_frontend_client(shared_enclave_info(), FRONTEND_SERVICE_ADDR, cred).await?;
    Ok((client1, client2))
}

async fn register_data_fusion_function(client: &mut FrontendClient) -> ExternalID {
    let script = r#"
def readlines(fid):
    lines = None
    with teaclave_open(fid, "rb") as f:
        lines = f.readlines()
    return lines

def entrypoint(argv):
    outfile = "OutFusionData"
    infiles = ["InPartyA", "InPartyB"]
    cnt = 0
    with teaclave_open(outfile, "wb") as of:
        for fid in infiles:
            for line in readlines(fid):
                of.write(line)
                cnt += 1
    summary = "Mixed %d lines of data" % cnt
    return summary
"#;

    let input1 = FunctionInput::new("InPartyA", "Input from party A", false);
    let input2 = FunctionInput::new("InPartyB", "Input from party B", false);
    let fusion_output = FunctionOutput::new("OutFusionData", "Output fusion data", false);
    let request = RegisterFunctionRequestBuilder::new()
        .name("mesapy_data_fusion_demo")
        .description("Mesapy Data Fusion Function")
        .payload(script.into())
        .executor_type(ExecutorType::Python)
        .public(true)
        .inputs(vec![input1, input2])
        .outputs(vec![fusion_output])
        .build();
    let response = client
        .register_function(request)
        .await
        .unwrap()
        .into_inner();

    log::debug!("Resgister function: {:?}", response);
    response.function_id.try_into().unwrap()
}

async fn register_input_file(
    client: &mut FrontendClient,
    url: impl AsRef<str>,
    crypto: impl Into<FileCrypto>,
    auth_tag: impl AsRef<str>,
) -> ExternalID {
    let url = Url::parse(url.as_ref()).unwrap();
    let cmac = FileAuthTag::from_hex(auth_tag.as_ref()).unwrap();
    let request = RegisterInputFileRequest::new(url, cmac, crypto);
    let response = client
        .register_input_file(request)
        .await
        .unwrap()
        .into_inner();
    log::debug!("Register input: {:?}", response);
    response.data_id.try_into().unwrap()
}

async fn register_fusion_output(
    client: &mut FrontendClient,
    file_owners: impl Into<OwnerList>,
) -> ExternalID {
    let request = RegisterFusionOutputRequest::new(file_owners);
    let response = client
        .register_fusion_output(request)
        .await
        .unwrap()
        .into_inner();
    response.data_id.try_into().unwrap()
}

async fn create_data_fusion_task(
    client: &mut FrontendClient,
    function_id: &ExternalID,
) -> ExternalID {
    let request = CreateTaskRequest::new()
        .function_id(function_id.to_owned())
        .inputs_ownership(hashmap!(
            "InPartyA" => vec![USERNAME1],
            "InPartyB" => vec![USERNAME2]
        ))
        .outputs_ownership(hashmap!("OutFusionData" => vec![USERNAME1, USERNAME2]))
        .executor(Executor::MesaPy);
    let response = client.create_task(request).await.unwrap().into_inner();
    log::debug!("Create task: {:?}", response);
    response.task_id.try_into().unwrap()
}

async fn assign_data_for_task(
    client: &mut FrontendClient,
    task_id: &ExternalID,
    input_map: HashMap<String, ExternalID>,
    output_map: HashMap<String, ExternalID>,
) {
    let request = AssignDataRequest::new(task_id.clone(), input_map, output_map);
    let response = client.assign_data(request).await.unwrap();
    log::debug!("Assign data: {:?}", response);
}

#[async_test_case]
pub async fn test_data_fusion_success() {
    let (mut c1, mut c2) = setup_client().await.unwrap();

    let function_id = register_data_fusion_function(&mut c1).await;

    // Create Task
    let task_id = create_data_fusion_task(&mut c1, &function_id).await;

    // Register Data and Assign Data To Task
    // input1 is owned by user1
    let path = "http://localhost:6789/fixtures/fusion/input1.enc";
    let key = "00000000000000000000000000000001";
    let iv = "123456781234567812345678";
    let cmac = "e8afd048b339fc835733e16c761a301c";
    let crypto = AesGcm128Key::from_hex(key, iv).unwrap();
    let input1 = register_input_file(&mut c1, path, crypto, cmac).await;

    // fusion_output is owned by user1 and user2
    let fusion_output = register_fusion_output(&mut c1, vec![USERNAME1, USERNAME2]).await;

    assign_data_for_task(
        &mut c1,
        &task_id,
        hashmap!("InPartyA" => input1),
        hashmap!("OutFusionData" => fusion_output),
    )
    .await;

    // input2 is owned by user2
    let path = "http://localhost:6789/fixtures/fusion/input2.enc";
    let key = "0000000000000000000000000000000000000000000000000000000000000002";
    let iv = "012345670123456701234567";
    let cmac = "75d7cf7a7843dee709e29ba0dcb865d2";
    let crypto = AesGcm256Key::from_hex(key, iv).unwrap();
    let input2 = register_input_file(&mut c2, path, crypto, cmac).await;
    assign_data_for_task(
        &mut c2,
        &task_id,
        hashmap!("InPartyB" => input2),
        hashmap!(),
    )
    .await;

    // Approve Task
    approve_task(&mut c1, &task_id).await.unwrap();
    approve_task(&mut c2, &task_id).await.unwrap();

    // Invoke Task by the creator
    invoke_task(&mut c1, &task_id).await.unwrap();

    // Get Task
    let ret_val = get_task_until(&mut c1, &task_id, TaskStatus::Finished).await;
    assert_eq!(&ret_val, "Mixed 5 lines of data");

    let task = get_task(&mut c2, &task_id).await;
    assert!(task.status == i32_from_task_status(TaskStatus::Finished));

    let outputs = from_proto_file_ids(task.assigned_outputs).unwrap();
    let fusion_id = outputs.get("OutFusionData").unwrap();
    let owners = from_proto_ownership(task.outputs_ownership);
    let fusion_owners = owners.get("OutFusionData").unwrap();

    let fusion_input = register_fusion_input_from_output(&mut c2, fusion_id).await;
    let function_id = register_word_count_function(&mut c2).await;

    let task_id = create_wlc_task(&mut c2, &function_id, fusion_owners).await;
    assign_data_for_task(
        &mut c2,
        &task_id,
        hashmap!("InputData" => fusion_input),
        hashmap!(),
    )
    .await;

    approve_task(&mut c2, &task_id).await.unwrap();

    // Invoke Task by the creator
    assert!(invoke_task(&mut c2, &task_id).await.is_err());

    approve_task(&mut c1, &task_id).await.unwrap();
    invoke_task(&mut c2, &task_id).await.unwrap();
    let ret_val = get_task_until(&mut c2, &task_id, TaskStatus::Finished).await;
    assert_eq!(&ret_val, "2");
}

async fn register_fusion_input_from_output(
    client: &mut FrontendClient,
    fusion_id: &ExternalID,
) -> ExternalID {
    let request = RegisterInputFromOutputRequest::new(fusion_id.clone());
    let response = client
        .register_input_from_output(request)
        .await
        .unwrap()
        .into_inner();
    response.data_id.try_into().unwrap()
}

async fn register_word_count_function(client: &mut FrontendClient) -> ExternalID {
    let script = r#"
def readlines(fid):
    lines = None
    with teaclave_open(fid, "rb") as f:
        lines = f.readlines()
    return lines

def entrypoint(argv):
    fid = "InputData"
    assert len(argv) == 2
    assert argv[0] == "query"
    word = argv[1]
    cnt = 0
    for line in readlines(fid):
        if word in line:
            cnt += 1
    return "%s" % cnt
"#;

    let input_spec = FunctionInput::new("InputData", "Lines of Data", false);
    let arg = FunctionArgument::new("query", "", true);
    let request = RegisterFunctionRequestBuilder::new()
        .name("wlc")
        .description("Mesapy Word Line Count Function")
        .arguments(vec![arg])
        .payload(script.into())
        .executor_type(ExecutorType::Python)
        .public(true)
        .inputs(vec![input_spec])
        .build();
    let response = client
        .register_function(request)
        .await
        .unwrap()
        .into_inner();
    log::debug!("Resgister function: {:?}", response);
    response.function_id.try_into().unwrap()
}

async fn create_wlc_task(
    client: &mut FrontendClient,
    function_id: &ExternalID,
    owners: &OwnerList,
) -> ExternalID {
    let request = CreateTaskRequest::new()
        .function_id(function_id.to_owned())
        .function_arguments(hashmap!("query" => "teaclave"))
        .inputs_ownership(hashmap!(
            "InputData" => owners.to_owned()
        ))
        .executor(Executor::MesaPy);
    let response = client.create_task(request).await.unwrap().into_inner();
    log::debug!("Create task: {:?}", response);
    response.task_id.try_into().unwrap()
}
