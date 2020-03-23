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
use std::collections::HashMap;
use std::collections::HashSet;
use std::prelude::v1::*;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_types::*;

use uuid::Uuid;

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(test_pull_task, test_update_task_status_result)
}

fn test_pull_task() {
    let task_id = Uuid::new_v4();
    let function_id = Uuid::new_v4();
    let function_name = "echo";

    let staged_task = StagedTask::new()
        .task_id(task_id)
        .function_id(function_id.clone())
        .function_name(function_name);

    let mut storage_client = get_storage_client();
    let enqueue_request = EnqueueRequest::new(
        StagedTask::get_queue_key().as_bytes(),
        staged_task.to_vec().unwrap(),
    );
    let _enqueue_response = storage_client.enqueue(enqueue_request).unwrap();

    let mut client = get_scheduler_client();
    let request = PullTaskRequest {};
    let response = client.pull_task(request);
    log::debug!("response: {:?}", response);
    assert!(response.is_ok());
    assert_eq!(response.unwrap().staged_task.function_id, function_id);
}

fn test_update_task_status_result() {
    let task_id = Uuid::new_v4();

    let task = Task {
        task_id,
        creator: "".to_string(),
        function_id: "".to_string(),
        function_owner: "".to_string(),
        function_arguments: FunctionArguments::default(),
        input_data_owner_list: HashMap::new(),
        output_data_owner_list: HashMap::new(),
        participants: HashSet::new(),
        approved_user_list: HashSet::new(),
        input_map: HashMap::new(),
        output_map: HashMap::new(),
        return_value: None,
        output_file_hash: HashMap::new(),
        status: TaskStatus::Running,
    };

    let function_id = Uuid::new_v4();
    let function_name = "echo";

    let staged_task = StagedTask::new()
        .task_id(task_id.clone())
        .function_id(function_id)
        .function_name(function_name);

    let mut storage_client = get_storage_client();
    let enqueue_request = EnqueueRequest::new(
        StagedTask::get_queue_key().as_bytes(),
        staged_task.to_vec().unwrap(),
    );
    let _enqueue_response = storage_client.enqueue(enqueue_request).unwrap();

    let put_request = PutRequest::new(task.key().as_slice(), task.to_vec().unwrap().as_slice());
    let _put_response = storage_client.put(put_request).unwrap();

    let mut client = get_scheduler_client();
    let request = PullTaskRequest {};
    let response = client.pull_task(request).unwrap();
    log::debug!("response: {:?}", response);
    let task_id = response.staged_task.task_id;

    let request = UpdateTaskStatusRequest::new(task_id, TaskStatus::Finished);
    let response = client.update_task_status(request);
    assert!(response.is_ok());

    let request =
        UpdateTaskResultRequest::new(task_id, "return".to_string().as_bytes(), HashMap::new());
    let response = client.update_task_result(request);

    assert!(response.is_ok());
}
