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
    save_file_for_user, setup_fns_client, setup_tms_external_client, USER_ONE, USER_TWO,
};

pub fn api_invoke_task() {
    trace!("Test FNS: invoke task.");

    let mut tms_client = setup_tms_external_client(&USER_ONE);

    let function_name = "echo";
    let launch_info = tms_client
        .request_create_task(function_name, &[], &[])
        .unwrap();

    let payload = Some("abc");
    // Bad task id
    let mut fns_client = setup_fns_client(launch_info.ip, launch_info.port);
    let response = fns_client.invoke_task(
        "bad_task_id",
        function_name,
        &launch_info.task_token,
        payload,
    );
    assert!(response.is_err());
    // Bad task token
    let mut fns_client = setup_fns_client(launch_info.ip, launch_info.port);
    let response = fns_client.invoke_task(
        &launch_info.task_id,
        function_name,
        "bad_task_token",
        payload,
    );
    assert!(response.is_err());
    // Bad function name
    let mut fns_client = setup_fns_client(launch_info.ip, launch_info.port);
    let response = fns_client.invoke_task(
        &launch_info.task_id,
        "bad_function_name",
        &launch_info.task_token,
        payload,
    );
    assert!(response.is_err());

    let mut fns_client = setup_fns_client(launch_info.ip, launch_info.port);
    let response = fns_client
        .invoke_task(
            &launch_info.task_id,
            function_name,
            &launch_info.task_token,
            payload,
        )
        .unwrap();
    assert_eq!("abc", response.result.as_str());
    // Bad status
    let mut fns_client = setup_fns_client(launch_info.ip, launch_info.port);
    let response = fns_client.invoke_task(
        &launch_info.task_id,
        function_name,
        &launch_info.task_token,
        payload,
    );
    assert!(response.is_err());

    // Bad Input
    let mut fns_client = setup_fns_client(launch_info.ip, launch_info.port);
    let function_name = "echo";
    let launch_info = tms_client
        .request_create_task(function_name, &[], &[])
        .unwrap();
    let response = fns_client.invoke_task(
        &launch_info.task_id,
        function_name,
        &launch_info.task_token,
        None,
    );
    assert!(response.is_err());
}

pub fn api_invoke_multiparty_task() {
    trace!("Test FNS: invoke multiparty task.");
    let user_file_id = save_file_for_user(&USER_ONE, b"abc", "concat_file1");
    let collaborator_file_id = save_file_for_user(&USER_TWO, b"def", "concat_file2");

    let function = "concat";
    let mut user_tms_client = setup_tms_external_client(&USER_ONE);
    let collaborator_list = vec![USER_TWO.user_id];
    let input_list = vec![user_file_id.as_str()];
    let launch_info = user_tms_client
        .request_create_task(function, &collaborator_list, &input_list)
        .unwrap();

    let mut collaborator_tms_client = setup_tms_external_client(&USER_TWO);

    let input_list = vec![collaborator_file_id.as_str()];
    let _ = collaborator_tms_client
        .request_update_task(&launch_info.task_id, &input_list)
        .unwrap();

    let mut fns_client = setup_fns_client(launch_info.ip, launch_info.port);

    let response = fns_client
        .invoke_task(
            &launch_info.task_id,
            function,
            &launch_info.task_token,
            None,
        )
        .unwrap();

    assert_eq!(response.result.as_str(), "abcdef");
}
