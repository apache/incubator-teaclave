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

use crate::common::{check_single_task_response, save_file_for_user};
use crate::config::USER_ONE;

pub fn test_echo() {
    trace!(">>>>> echo");
    let function_name = "echo";
    let payload = Some("echo_payload");
    check_single_task_response(&USER_ONE, function_name, payload, "echo_payload");
}

pub fn test_bytes_plus_one() {
    trace!(">>>>> bytes plus one");
    let function_name = "bytes_plus_one";
    let payload = Some("abc");
    check_single_task_response(&USER_ONE, function_name, payload, "bcd");
}

pub fn test_echo_file() {
    trace!(">>>>> echo file");
    let file_path = "./test_data/file1.txt";
    let user_file_id = save_file_for_user(&USER_ONE, file_path);
    let function_name = "echo_file";
    let payload = Some(user_file_id.as_str());
    check_single_task_response(&USER_ONE, function_name, payload, "abcdefg");
}

pub fn test_file_bytes_plus_one() {
    trace!(">>>>> file bytes plus one");
    let file_path = "./test_data/file1.txt";
    let user_file_id = save_file_for_user(&USER_ONE, file_path);
    let function_name = "file_bytes_plus_one";
    let payload = Some(user_file_id.as_str());
    check_single_task_response(&USER_ONE, function_name, payload, "bcdefgh");
}
