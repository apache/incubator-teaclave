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

use super::common_setup::{setup_tdfs_external_client, USER_ERR, USER_ONE, USER_THREE, USER_TWO};
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
