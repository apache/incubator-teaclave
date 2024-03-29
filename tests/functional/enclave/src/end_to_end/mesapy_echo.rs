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
use teaclave_test_utils::async_test_case;
#[async_test_case]
pub async fn test_echo_task_success() {
    // Authenticate user before talking to frontend service
    let mut api_client = create_authentication_api_client(shared_enclave_info(), AUTH_SERVICE_ADDR)
        .await
        .unwrap();
    let cred = login(&mut api_client, USERNAME, TEST_PASSWORD)
        .await
        .unwrap();
    let mut client = create_frontend_client(shared_enclave_info(), FRONTEND_SERVICE_ADDR, cred)
        .await
        .unwrap();

    let script = "
def entrypoint(argv):
    assert argv[0] == 'message'
    assert argv[1] is not None
    return argv[1]
";
    let arg = FunctionArgument::new("message", "", true);
    // Register Function
    let request = RegisterFunctionRequestBuilder::new()
        .name("mesapy_echo_demo")
        .description("Mesapy Echo Function")
        .payload(script.into())
        .executor_type(ExecutorType::Python)
        .public(true)
        .arguments(vec![arg])
        .build();

    let response = client
        .register_function(request)
        .await
        .unwrap()
        .into_inner();
    log::debug!("Resgister function: {:?}", response);

    // Create Task
    let function_id = response.function_id.try_into().unwrap();
    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("message" => "Hello From Teaclave!"))
        .executor(Executor::MesaPy);

    let response = client.create_task(request).await.unwrap().into_inner();
    log::debug!("Create task: {:?}", response);

    // Assign Data To Task
    let task_id: ExternalID = response.task_id.try_into().unwrap();
    let request = AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!());
    let response = client.assign_data(request).await.unwrap();

    log::debug!("Assign data: {:?}", response);

    // Approve Task
    approve_task(&mut client, &task_id).await.unwrap();

    // Invoke Task
    invoke_task(&mut client, &task_id).await.unwrap();

    // Get Task
    let ret_val = get_task_until(&mut client, &task_id, TaskStatus::Finished).await;
    assert_eq!(&ret_val, "Hello From Teaclave!");
}
