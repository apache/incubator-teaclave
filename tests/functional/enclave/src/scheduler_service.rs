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
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_test_utils::test_case;
use teaclave_types::*;

use uuid::Uuid;

#[test_case]
fn test_pull_task() {
    let function_id = Uuid::new_v4();
    let staged_task = StagedTask::new()
        .task_id(Uuid::new_v4())
        .function_name("builtin-echo")
        .function_id(function_id)
        .executor(Executor::Builtin);

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

#[test_case]
fn test_update_task_status_result() {
    let task_id = Uuid::new_v4();
    let function_id = Uuid::new_v4();

    let staged_task = StagedTask::new()
        .task_id(task_id)
        .function_name("builtin-echo")
        .function_id(function_id)
        .executor(Executor::Builtin);

    let mut storage_client = get_storage_client();
    let enqueue_request = EnqueueRequest::new(
        StagedTask::get_queue_key().as_bytes(),
        staged_task.to_vec().unwrap(),
    );
    let _enqueue_response = storage_client.enqueue(enqueue_request).unwrap();

    let ts = TaskState {
        task_id,
        status: TaskStatus::Staged,
        ..Default::default()
    };
    let put_request = PutRequest::new(ts.key().as_slice(), ts.to_vec().unwrap().as_slice());
    let _put_response = storage_client.put(put_request).unwrap();

    let mut client = get_scheduler_client();
    let request = PullTaskRequest {};
    let response = client.pull_task(request).unwrap();
    log::debug!("response: {:?}", response);
    let task_id = response.staged_task.task_id;

    let request = UpdateTaskStatusRequest::new(task_id, TaskStatus::Running);
    let response = client.update_task_status(request);
    assert!(response.is_ok());

    let task_outputs = TaskOutputs::new("return value", hashmap!());
    let request = UpdateTaskResultRequest::new(task_id, Ok(task_outputs));
    let response = client.update_task_result(request);

    assert!(response.is_ok());
}
