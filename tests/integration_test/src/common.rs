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

use crate::config::User;
use fns_client::FNSClient;
use mesatee_core::config::{get_trusted_enclave_attr, OutboundDesc, TargetDesc};
use std::net::{IpAddr, Ipv4Addr};
use tdfs_external_client::TDFSClient;
use tms_external_client::TMSClient;
use tms_external_proto::{FunctionType, TaskStatus};

#[inline]
fn target_tms() -> TargetDesc {
    TargetDesc::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        5554,
        OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["tms"])),
    )
}

#[inline]
fn target_tdfs() -> TargetDesc {
    TargetDesc::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        5065,
        OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["tdfs"])),
    )
}

#[inline]
fn outbound_default() -> OutboundDesc {
    OutboundDesc::Sgx(get_trusted_enclave_attr(vec!["fns"]))
}

pub(crate) fn setup_tms_client(user: &User) -> TMSClient {
    let target = target_tms();
    TMSClient::new(&target, user.user_id, user.user_token).unwrap()
}

pub(crate) fn setup_tdfs_client(user: &User) -> TDFSClient {
    let target = target_tdfs();
    TDFSClient::new(&target, user.user_id, user.user_token).unwrap()
}

pub(crate) fn setup_fns_client(ip: IpAddr, port: u16) -> FNSClient {
    let desc = outbound_default();
    let target = TargetDesc::new(ip, port, desc);
    FNSClient::new(&target).unwrap()
}

pub(crate) fn save_file_for_user(user: &User, file_path: &str) -> String {
    let mut tdfs_client = setup_tdfs_client(user);
    let file_name = "file";
    tdfs_client.save_file(file_path, file_name).unwrap()
}

pub(crate) fn launch_single_task(
    user: &User,
    function_name: &str,
    payload: Option<&str>,
) -> String {
    let mut tms_client = setup_tms_client(user);
    let launch_info = tms_client
        .request_create_task(function_name, &[], &[])
        .unwrap();

    let mut fns_client = setup_fns_client(launch_info.ip, launch_info.port);
    let response = fns_client
        .invoke_task(
            &launch_info.task_id,
            function_name,
            &launch_info.task_token,
            payload,
        )
        .unwrap();

    response.result
}

pub(crate) fn check_single_task_response(
    user: &User,
    function_name: &str,
    payload: Option<&str>,
    expected_result: &str,
) {
    let result = launch_single_task(user, function_name, payload);

    assert_eq!(expected_result, result.as_str());
}

pub(crate) struct TwoPartyTaskLaunchResult {
    pub result: String,
    pub task_id: String,
}

pub(crate) fn two_party_task_launch(
    user: &User,
    user_file_path: &str,
    collaborator: &User,
    collaborator_file_path: &str,
    function_name: &str,
) -> TwoPartyTaskLaunchResult {
    trace!(">>>>> concat file");

    let user_file_id = save_file_for_user(user, user_file_path);
    let collaborator_file_id = save_file_for_user(collaborator, collaborator_file_path);

    // user creates task
    let mut user_tms_client = setup_tms_client(user);
    let collaborator_list = &[collaborator.user_id];
    let input_list = &[user_file_id.as_str()];
    let launch_info = user_tms_client
        .request_create_task(function_name, collaborator_list, input_list)
        .unwrap();

    // collaborator checks task
    let mut collaborator_tms_client = setup_tms_client(collaborator);
    let task_info = collaborator_tms_client
        .request_get_task(&launch_info.task_id)
        .unwrap()
        .task_info;

    assert_eq!(task_info.function_name.as_str(), function_name);
    assert_eq!(task_info.user_id.as_str(), user.user_id);
    assert_eq!(task_info.status, TaskStatus::Created);
    assert_eq!(task_info.function_type, FunctionType::Multiparty);
    assert!(!task_info.collaborator_list.is_empty());
    assert_eq!(task_info.collaborator_list[0].user_id, collaborator.user_id);

    // collaborator adds data
    let input_list = &[collaborator_file_id.as_str()];
    let response = collaborator_tms_client
        .request_update_task(&launch_info.task_id, input_list)
        .unwrap();

    assert_eq!(response.success, true);
    assert_eq!(response.status, TaskStatus::Ready);
    assert_eq!(&response.ip, &launch_info.ip);
    assert_eq!(response.port, launch_info.port);
    assert_eq!(&response.task_token, &launch_info.task_token);

    // user checks task
    let task_info = user_tms_client
        .request_get_task(&launch_info.task_id)
        .unwrap()
        .task_info;
    assert_eq!(task_info.status, TaskStatus::Ready);

    // user/collaborator starts task
    let mut fns_client = setup_fns_client(launch_info.ip, launch_info.port);
    let payload = None;
    let response = fns_client
        .invoke_task(
            &launch_info.task_id,
            function_name,
            &launch_info.task_token,
            payload,
        )
        .unwrap();

    TwoPartyTaskLaunchResult {
        result: response.result,
        task_id: launch_info.task_id,
    }
}

pub(crate) fn assert_file_content(user: &User, file_id: &str, expected_result: &[u8]) {
    let mut user_tdfs_client = setup_tdfs_client(user);
    let plaintxt = user_tdfs_client.read_file(file_id).unwrap();
    assert_eq!(expected_result, plaintxt.as_slice());
}

pub(crate) fn assert_private_result_content(user: &User, task_id: &str, expected_result: &[u8]) {
    let mut user_tms_client = setup_tms_client(user);

    let task_info = user_tms_client.request_get_task(task_id).unwrap().task_info;

    let file_id = &task_info.user_private_result_file_id[0];
    assert_file_content(user, file_id, expected_result);
}

pub(crate) fn assert_shared_result_content(user: &User, task_id: &str, expected_result: &[u8]) {
    let mut user_tms_client = setup_tms_client(user);

    let task_info = user_tms_client.request_get_task(task_id).unwrap().task_info;

    let file_id = task_info.task_result_file_id.unwrap();
    assert_file_content(user, &file_id, expected_result);
}
