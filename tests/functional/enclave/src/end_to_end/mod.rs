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
use std::prelude::v1::*;
use teaclave_proto::teaclave_frontend_service::*;
use teaclave_types::*;
use url::Url;

mod mesapy_echo;
mod native_echo;
mod native_gbdt_training;

const USERNAME: &str = "alice";
const PASSWORD: &str = "daHosldOdker0sS";

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;
    setup();
    run_tests!(
        native_gbdt_training::test_gbdt_training_task,
        native_echo::test_echo_task_success,
        mesapy_echo::test_echo_task_success,
    )
}

fn setup() {
    // Register user for the first time
    let mut api_client =
        create_authentication_api_client(shared_enclave_info(), AUTH_SERVICE_ADDR).unwrap();
    register_new_account(&mut api_client, USERNAME, PASSWORD).unwrap();
}

fn get_task_until(
    client: &mut TeaclaveFrontendClient,
    task_id: &ExternalID,
    status: TaskStatus,
) -> String {
    loop {
        let request = GetTaskRequest::new(task_id.clone());
        let response = client.get_task(request).unwrap();
        log::info!("Get task: {:?}", response);

        std::thread::sleep(std::time::Duration::from_secs(1));

        if response.status == status {
            match response.result {
                TaskResult::Ok(outputs) => {
                    let ret_val = String::from_utf8(outputs.return_value).unwrap();
                    log::info!("Task returns: {:?}", ret_val);
                    return ret_val;
                }
                TaskResult::Err(failure) => {
                    log::error!("Task failed, reason: {:?}", failure);
                    return failure.to_string();
                }
                TaskResult::NotReady => unreachable!(),
            }
        }
    }
}

fn approve_task(client: &mut TeaclaveFrontendClient, task_id: &ExternalID) {
    let request = ApproveTaskRequest::new(task_id.clone());
    let response = client.approve_task(request).unwrap();
    log::info!("Approve task: {:?}", response);
}

fn invoke_task(client: &mut TeaclaveFrontendClient, task_id: &ExternalID) {
    let request = InvokeTaskRequest::new(task_id.clone());
    let response = client.invoke_task(request).unwrap();
    log::info!("Invoke task: {:?}", response);
}
