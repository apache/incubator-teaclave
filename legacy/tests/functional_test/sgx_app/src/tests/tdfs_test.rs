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

use super::common_setup::{
    setup_tdfs_external_client, USER_ERR, USER_FAKE, USER_ONE, USER_THREE, USER_TWO,
};
use super::fns_test;
use std::fs;

pub fn read_not_exist_file() {
    trace!("Test tdfs: read file.");
    let mut client = setup_tdfs_external_client(&USER_ONE);
    let resp = client.read_file("xx");
    assert!(resp.is_err());
}

pub fn save_and_read() {
    trace!("Test tdfs: save and read file.");
    let mut client = setup_tdfs_external_client(&USER_ONE);

    let file_path = "./tdfs_functional_test";
    fs::write(file_path, b"abc").unwrap();
    let file_name = "functional_test";
    let file_id = client.save_file(file_path, file_name).unwrap();

    let plaintxt = client.read_file(&file_id).unwrap();
    assert_eq!(plaintxt, b"abc");

    // create file, not a valid user
    let mut client = setup_tdfs_external_client(&USER_ERR);
    let file_path = "./tdfs_functional_test";
    let file_name = "functional_test";
    let resp = client.save_file(file_path, file_name);
    assert!(resp.is_err());

    // read file, not a valid user
    let mut client = setup_tdfs_external_client(&USER_ERR);
    let read_err = client.read_file(&file_id);
    assert!(read_err.is_err());

    // no permission to read
    let mut client = setup_tdfs_external_client(&USER_TWO);
    let read_err = client.read_file(&file_id);
    assert!(read_err.is_err());

    // read file, key not found
    let mut client = setup_tdfs_external_client(&USER_THREE);
    let read_err = client.read_file("fake_file_without_key");
    assert!(read_err.is_err());
}

pub fn delete_file_api() {
    trace!("Test tdfs: delete a file");
    let mut client = setup_tdfs_external_client(&USER_THREE);
    let list = client.request_list_file().unwrap().list;
    assert_eq!(list.len(), 4);
    println!("{:?}", list);

    let mut client = setup_tdfs_external_client(&USER_FAKE);
    let list = client.request_list_file().unwrap().list;
    assert_eq!(list.len(), 2);

    let mut client = setup_tdfs_external_client(&USER_THREE);
    let resp = client.request_del_file("fake_file_to_be_deleted");
    let path = resp.unwrap().file_info.access_path;
    assert!(!path.is_empty());
    let list = client.request_list_file().unwrap().list;
    assert_eq!(list.len(), 3);

    let mut client = setup_tdfs_external_client(&USER_FAKE);
    assert!(!path.is_empty());
    let list = client.request_list_file().unwrap().list;
    assert_eq!(list.len(), 1);
}

pub fn list_file_api() {
    trace!("Test tdfs: list files");
    let mut client = setup_tdfs_external_client(&USER_THREE);
    let mut list = client.request_list_file().unwrap().list;
    list.sort_by(|a, b| a.cmp(b));
    assert_eq!(list.len(), 4);
    assert_eq!(list[0], "fake_file_record");
    assert_eq!(list[1], "fake_file_to_be_deleted");
    assert_eq!(list[2], "fake_file_with_collaborator");
    assert_eq!(list[3], "fake_file_without_key");

    let mut client = setup_tdfs_external_client(&USER_FAKE);
    let mut list = client.request_list_file().unwrap().list;
    list.sort_by(|a, b| a.cmp(b));
    assert_eq!(list.len(), 2);
    assert_eq!(list[0], "fake_file_to_be_deleted");
    assert_eq!(list[1], "fake_file_with_collaborator");

    let mut client = setup_tdfs_external_client(&USER_ONE);
    let list = client.request_list_file().unwrap().list;
    let user_one_file_count = list.len();
    let mut client = setup_tdfs_external_client(&USER_TWO);
    let list = client.request_list_file().unwrap().list;
    let user_two_file_count = list.len();

    fns_test::api_invoke_multiparty_task();

    let mut client = setup_tdfs_external_client(&USER_ONE);
    let list = client.request_list_file().unwrap().list;
    assert_eq!(user_one_file_count + 3, list.len());
    let mut client = setup_tdfs_external_client(&USER_TWO);
    let list = client.request_list_file().unwrap().list;
    assert_eq!(user_two_file_count + 3, list.len());
}
