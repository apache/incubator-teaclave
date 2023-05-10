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
    let arg = FunctionArgument::new("message", "", true);

    // Register Function
    let request = RegisterFunctionRequestBuilder::new()
        .name("builtin-echo")
        .description("Native Echo Function")
        .arguments(vec![arg])
        .build();

    let response = client
        .register_function(request)
        .await
        .unwrap()
        .into_inner();

    log::debug!("Register function: {:?}", response);

    // Create Task
    let function_id = response.function_id.try_into().unwrap();
    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("message" => "Hello From Teaclave!"))
        .executor(Executor::Builtin);

    let response = client.create_task(request).await.unwrap().into_inner();

    log::debug!("Create task: {:?}", response);

    let task_id = response.task_id.try_into().unwrap();

    // Assign Data To Task
    // This task does not have any input/output files, we can opt to skip the assignment process.

    // Approve Task
    // This task is a single user task, we can opt to skip the approvement process.
    // approve_task(&mut client, &task_id).unwrap();

    // Invoke Task
    invoke_task(&mut client, &task_id).await.unwrap();

    // Get Task
    let ret_val = get_task_until(&mut client, &task_id, TaskStatus::Finished).await;
    assert_eq!(&ret_val, "Hello From Teaclave!");
}
