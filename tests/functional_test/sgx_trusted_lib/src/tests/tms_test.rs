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

use super::common_setup::setup_tms_internal_client;
use std::net::{IpAddr, Ipv4Addr};
use std::prelude::v1::*;
use tms_internal_proto::{FunctionType, TaskFile, TaskStatus};

pub fn get_task() {
    trace!("Test TMS: get_task.");
    let mut client = setup_tms_internal_client();
    let resp = client.request_get_task("fake").unwrap();
    let task_info = resp.task_info;
    assert_eq!("fake", task_info.user_id.as_str());
    assert_eq!("echo", task_info.function_name.as_str());
    assert_eq!(FunctionType::Single, task_info.function_type);
    assert_eq!(TaskStatus::Ready, task_info.status);
    assert_eq!(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), task_info.ip);
    assert_eq!(0, task_info.port);
    assert_eq!("fake", task_info.task_token.as_str());
    assert!(task_info.input_files.is_empty());
    assert!(task_info.collaborator_list.is_empty());
    assert!(task_info.task_result_file_id.is_none());
    assert!(task_info.output_files.is_empty());
}

pub fn update_task_result() {
    trace!("Test TMS: update_task_result.");
    let mut client = setup_tms_internal_client();

    let resp = client
        .request_update_task("fake", Some("task_result"), &[], None)
        .unwrap();
    assert!(resp.success);

    let resp = client.request_get_task("fake").unwrap();
    let task_info = resp.task_info;
    assert_eq!(
        "task_result",
        task_info.task_result_file_id.unwrap().as_str()
    );

    // task not exists
    let resp = client
        .request_update_task("NULL", Some("task_result"), &[], None)
        .unwrap();
    assert!(!resp.success);
}

pub fn update_private_result() {
    trace!("Test TMS: update_client_private_result.");
    let mut client = setup_tms_internal_client();

    let task_file = TaskFile {
        user_id: "fake".to_owned(),
        file_id: "client_private_result".to_owned(),
    };
    let update_input = [&task_file];
    let resp = client
        .request_update_task("fake", None, &update_input, None)
        .unwrap();
    assert!(resp.success);

    let resp = client.request_get_task("fake").unwrap();
    let task_info = resp.task_info;
    assert_eq!(
        "client_private_result",
        task_info.output_files[0].file_id.as_str()
    );
}

pub fn update_status() {
    trace!("Test TMS: update_status.");
    let mut client = setup_tms_internal_client();

    let resp = client
        .request_update_task("fake", None, &[], Some(&TaskStatus::Finished))
        .unwrap();
    assert!(resp.success);

    let resp = client.request_get_task("fake").unwrap();
    let task_info = resp.task_info;
    assert_eq!(TaskStatus::Finished, task_info.status);
}
