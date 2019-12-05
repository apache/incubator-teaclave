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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use super::common_setup::setup_tdfs_internal_client;
use crate::log::trace;

pub fn read_not_exist_file() {
    trace!("Test tdfs: read file.");
    let mut client = setup_tdfs_internal_client();
    let resp = client.read_file("xx", "fake", "fake");
    assert!(resp.is_err());
}

pub fn save_and_read() {
    trace!("Test tdfs: save and read file.");
    let mut client = setup_tdfs_internal_client();

    let data = b"abc";
    let user_id = "fake";
    let disallowed_user = "disallowed_user";
    let task_id = "fake";
    let task_token = "fake";
    let allow_policy = 0;

    let file_id = client
        .save_file(data, user_id, task_id, task_token, &[], allow_policy)
        .unwrap();

    let plaintxt = client.read_file(&file_id, task_id, task_token).unwrap();
    assert_eq!(plaintxt, b"abc");

    let accessible = client.check_access_permission(&file_id, user_id).unwrap();
    assert_eq!(accessible, true);

    let accessible = client
        .check_access_permission(&file_id, disallowed_user)
        .unwrap();
    assert_eq!(accessible, false);
}

pub fn check_user_permission() {
    trace!("Test tdfs: check user permission.");
    let mut client = setup_tdfs_internal_client();

    let data = b"abcd";
    let user_id = "fake";
    let disallowed_user = "user2";
    let task_id = "fake";
    let task_token = "fake";
    let allow_policy = 0;

    let file_id = client
        .save_file(data, user_id, task_id, task_token, &[], allow_policy)
        .unwrap();

    let plaintxt = client.read_file(&file_id, task_id, task_token).unwrap();
    assert_eq!(plaintxt, b"abcd");

    let accessible = client.check_access_permission(&file_id, &user_id).unwrap();
    assert!(accessible);

    let accessible = client
        .check_access_permission(&file_id, &disallowed_user)
        .unwrap();
    assert!(!accessible);
}

pub fn check_write_permission() {
    trace!("Test tdfs: check write permission");
    let data = b"bcd";
    let task_id = "fake_multi_task";
    let task_token = "fake";
    let allow_policy = 1;

    let mut client = setup_tdfs_internal_client();
    let user_id = "fake";
    let collaborator = "fake_file_owner";
    let collorabor_list = vec![collaborator];
    let disallowed_user = "user3";
    let disallowed_collorabor_list = vec![disallowed_user];

    let result = client.save_file(
        data,
        user_id,
        task_id,
        task_token,
        &collorabor_list,
        allow_policy,
    );
    assert!(result.is_ok());

    let mut client = setup_tdfs_internal_client();
    let result = client.save_file(
        data,
        disallowed_user,
        task_id,
        task_token,
        &collorabor_list,
        allow_policy,
    );
    assert!(result.is_err());

    let mut client = setup_tdfs_internal_client();
    let result = client.save_file(
        data,
        user_id,
        task_id,
        task_token,
        &disallowed_collorabor_list,
        allow_policy,
    );
    assert!(result.is_err());
}
pub fn task_share_file() {
    trace!("Test tdfs: save a file for user and collaborator.");
    let mut client = setup_tdfs_internal_client();

    let data = b"bcd";
    let user_id = "fake";
    let collaborator = "fake_file_owner";
    let collorabor_list = vec![collaborator];
    let disallowed_user = "user3";

    let task_id = "fake_multi_task";
    let task_token = "fake";
    let allow_policy = 1;

    let file_id = client
        .save_file(
            data,
            user_id,
            task_id,
            task_token,
            &collorabor_list,
            allow_policy,
        )
        .unwrap();

    let plaintxt = client.read_file(&file_id, task_id, task_token).unwrap();
    assert_eq!(plaintxt, b"bcd");

    let accessible = client.check_access_permission(&file_id, user_id).unwrap();
    assert!(accessible);

    let accessible = client
        .check_access_permission(&file_id, collaborator)
        .unwrap();
    assert!(accessible);

    let accessible = client
        .check_access_permission(&file_id, disallowed_user)
        .unwrap();
    assert!(!accessible);
}

pub fn global_share_file() {
    trace!("Test tdfs: global share file.");
    let mut client = setup_tdfs_internal_client();

    let data = b"cde";
    let user_id = "fake";
    let another_user = "user2";
    let task_id = "fake";
    let task_token = "fake";
    let allow_policy = 2;

    let file_id = client
        .save_file(data, user_id, task_id, task_token, &[], allow_policy)
        .unwrap();

    let plaintxt = client.read_file(&file_id, task_id, task_token).unwrap();
    assert_eq!(plaintxt, b"cde");

    let accessible = client.check_access_permission(&file_id, user_id).unwrap();
    assert_eq!(accessible, true);

    let accessible = client
        .check_access_permission(&file_id, another_user)
        .unwrap();
    assert_eq!(accessible, true);
}
