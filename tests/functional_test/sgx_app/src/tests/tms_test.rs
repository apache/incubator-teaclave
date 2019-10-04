// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::common_setup::{
    save_file_for_user, setup_tms_external_client, USER_ERR, USER_FAKE, USER_FOUR, USER_ONE,
    USER_THREE, USER_TWO,
};
use tms_external_proto::{FunctionType, TaskStatus};

pub fn api_create_task() {
    trace!("Test tms: create task.");
    let mut client = setup_tms_external_client(&USER_ONE);

    let function_name = "abc";
    let collaborator_list = vec!["collaborator_id"];

    let response = client
        .request_create_task(function_name, &collaborator_list, &[])
        .unwrap();
    println!("{:?}", response);

    // invalid user_token
    let mut client = setup_tms_external_client(&USER_ERR);
    let function_name = "abc";
    let response = client.request_create_task(function_name, &[], &[]);
    assert!(response.is_err());

    // multi_party_task, empty collaborator_list
    let mut client = setup_tms_external_client(&USER_THREE);
    let function_name = "psi";
    let response = client.request_create_task(function_name, &[], &["fake_file_record"]);
    assert!(response.is_err());

    // multi_party_task, empty file
    let mut client = setup_tms_external_client(&USER_ERR);
    let function_name = "psi";
    let response = client.request_create_task(function_name, &[USER_TWO.user_id], &[]);
    assert!(response.is_err());

    // multi_party_task, invalid file
    let function_name = "psi";
    let mut client = setup_tms_external_client(&USER_ERR);
    let response =
        client.request_create_task(function_name, &[USER_TWO.user_id], &["non_exist_file"]);
    assert!(response.is_err());
}

pub fn api_get_task() {
    trace!("Test tms: get task.");
    let mut client = setup_tms_external_client(&USER_ONE);

    let function_name = "abc";
    let collaborator_list = vec!["collaborator_id"];

    let launch_info = client
        .request_create_task(function_name, &collaborator_list, &[])
        .unwrap();
    let task_info = client
        .request_get_task(&launch_info.task_id)
        .unwrap()
        .task_info;

    assert_eq!(task_info.user_id.as_str(), USER_ONE.user_id);
    assert_eq!(&task_info.function_name, &function_name);
    assert_eq!(task_info.status, TaskStatus::Ready);
    assert_eq!(task_info.function_type, FunctionType::Single);
    assert_eq!(&task_info.ip, &launch_info.ip);
    assert_eq!(&task_info.port, &launch_info.port);
    assert_eq!(&task_info.task_token, &launch_info.task_token);
    assert!(&task_info.user_private_result_file_id.is_empty());

    let resp = client.request_get_task("null");
    assert!(resp.is_err());

    // invalid user
    let mut client = setup_tms_external_client(&USER_ERR);
    let response = client.request_get_task(&launch_info.task_id);
    assert!(response.is_err());

    // no permission
    let mut client = setup_tms_external_client(&USER_TWO);
    let response = client.request_get_task(&launch_info.task_id);
    assert!(response.is_err());
}

pub fn api_update_task() {
    trace!("Test tms: update task.");
    let mut user_tms_client = setup_tms_external_client(&USER_ONE);

    //Create file
    let file_id = save_file_for_user(&USER_ONE, b"abc", "./tdfs_1");

    // Create task
    let function_name = "psi";
    let collaborator_list = vec![USER_TWO.user_id];
    let input_list = vec![file_id.as_str()];
    let launch_info = user_tms_client
        .request_create_task(function_name, &collaborator_list, &input_list)
        .unwrap();

    // Check task
    let task_info = user_tms_client
        .request_get_task(&launch_info.task_id)
        .unwrap()
        .task_info;
    assert_eq!(task_info.function_type, FunctionType::Multiparty);
    assert_eq!(task_info.collaborator_list.len(), 1);
    assert_eq!(task_info.collaborator_list[0].user_id, USER_TWO.user_id);

    // Not a collaborator
    let mut collaborator_tms_client = setup_tms_external_client(&USER_THREE);
    let response =
        collaborator_tms_client.request_update_task(&launch_info.task_id, &["fake_file_record"]);
    assert!(response.is_err());

    let mut collaborator_tms_client = setup_tms_external_client(&USER_TWO);
    // invalid file
    let response =
        collaborator_tms_client.request_update_task(&launch_info.task_id, &["fake_file_record"]);
    assert!(response.is_err());

    let mut collaborator_tms_client = setup_tms_external_client(&USER_TWO);
    let collaborator_file_id = save_file_for_user(&USER_TWO, b"abc", "./tdfs_2");

    // collaborator check task
    let task_info = collaborator_tms_client
        .request_get_task(&launch_info.task_id)
        .unwrap()
        .task_info;
    assert_eq!(task_info.user_id.as_str(), USER_ONE.user_id);
    assert_eq!(&task_info.function_name, &function_name);
    assert_eq!(task_info.status, TaskStatus::Created);
    assert_eq!(task_info.function_type, FunctionType::Multiparty);
    assert_eq!(task_info.collaborator_list.len(), 1);
    assert_eq!(task_info.collaborator_list[0].user_id, USER_TWO.user_id);

    // collaborator update task
    let input_list = vec![collaborator_file_id.as_str()];
    let response = collaborator_tms_client
        .request_update_task(&launch_info.task_id, &input_list)
        .unwrap();
    assert!(response.success);
    assert_eq!(response.status, TaskStatus::Ready);

    // collaborator check task
    let task_info = user_tms_client
        .request_get_task(&launch_info.task_id)
        .unwrap()
        .task_info;
    assert_eq!(task_info.status, TaskStatus::Ready);

    // invalid user
    let mut collaborator_tms_client = setup_tms_external_client(&USER_ERR);
    let response = collaborator_tms_client.request_update_task(&launch_info.task_id, &[]);
    assert!(response.is_err());

    // No Task
    let mut collaborator_tms_client = setup_tms_external_client(&USER_THREE);
    let response = collaborator_tms_client.request_update_task("NULL", &["fake_file_record"]);
    assert!(response.is_err());

    // Task is Single
    let mut collaborator_tms_client = setup_tms_external_client(&USER_THREE);
    let response = collaborator_tms_client.request_update_task("fake", &["fake_file_record"]);
    assert!(response.is_err());

    // Task status is ready
    let mut collaborator_tms_client = setup_tms_external_client(&USER_THREE);
    let response =
        collaborator_tms_client.request_update_task(&launch_info.task_id, &["fake_file_record"]);
    assert!(response.is_err());
}

pub fn api_list_task() {
    trace!("Test tms: list task.");
    let mut user_tms_client = setup_tms_external_client(&USER_FAKE);

    let mut list = user_tms_client.request_list_task().unwrap().list;

    list.sort_by(|a, b| a.cmp(b));

    assert_eq!(list.len(), 2);
    assert_eq!(list[0], "fake");
    assert_eq!(list[1], "fake_multi_task");

    let mut user_tms_client = setup_tms_external_client(&USER_THREE);
    let list = user_tms_client.request_list_task().unwrap().list;
    assert_eq!(list.len(), 1);
    assert_eq!(list[0], "fake_multi_task");

    let mut user_tms_client = setup_tms_external_client(&USER_TWO);
    let list = user_tms_client.request_list_task().unwrap().list;
    assert!(!list.is_empty());

    let mut user_tms_client = setup_tms_external_client(&USER_FOUR);
    let list = user_tms_client.request_list_task().unwrap().list;
    assert_eq!(list.len(), 0);
}
