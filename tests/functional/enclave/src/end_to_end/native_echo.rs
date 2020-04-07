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

pub fn test_echo_task_success() {
    // Authenticate user before talking to frontend service
    let mut api_client =
        create_authentication_api_client(shared_enclave_info(), AUTH_SERVICE_ADDR).unwrap();
    let cred = login(&mut api_client, USERNAME, PASSWORD).unwrap();
    let mut client =
        create_frontend_client(shared_enclave_info(), FRONTEND_SERVICE_ADDR, cred).unwrap();

    // Register Function
    let request = RegisterFunctionRequest::new()
        .name("native_echo_demo")
        .description("Native Echo Function")
        .executor_type(ExecutorType::Native)
        .public(true)
        .arguments(vec!["message"]);

    let response = client.register_function(request).unwrap();

    log::info!("Resgister function: {:?}", response);

    // Create Task
    let function_id = response.function_id;
    let request = CreateTaskRequest::new()
        .function_id(function_id)
        .function_arguments(hashmap!("message" => "Hello From Teaclave!"))
        .executor(Executor::Echo);

    let response = client.create_task(request).unwrap();

    log::info!("Create task: {:?}", response);

    // Assign Data To Task
    let task_id = response.task_id;
    let request = AssignDataRequest::new(task_id.clone(), hashmap!(), hashmap!());
    let response = client.assign_data(request).unwrap();

    log::info!("Assign data: {:?}", response);

    // Approve Task
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client.approve_task(request).unwrap();

    log::info!("Approve task: {:?}", response);

    // Invoke Task
    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client.invoke_task(request).unwrap();

    log::info!("Invoke task: {:?}", response);

    // Get Task
    loop {
        let request = GetTaskRequest::new(task_id.clone());
        let response = client.get_task(request).unwrap();
        log::info!("Get task: {:?}", response);
        std::thread::sleep(std::time::Duration::from_secs(1));
        if response.status != TaskStatus::Running {
            let ret_val = String::from_utf8(response.return_value).unwrap();
            log::info!("Task returns: {:?}", ret_val);
            assert_eq!(&ret_val, "Hello From Teaclave!");
            break;
        }
    }
}
