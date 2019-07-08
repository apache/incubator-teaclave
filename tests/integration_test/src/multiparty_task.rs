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

use crate::common::{
    assert_file_content, assert_private_result_content, assert_shared_result_content,
    two_party_task_launch,
};
use crate::config::{USER_ONE, USER_TWO};
use log::trace;

pub fn test_concat_files() {
    trace!(">>>>> concat file");

    let user_file_path = "./test_data/file1.txt";
    let collaborator_file_path = "./test_data/file2.txt";
    let function_name = "concat";
    let expected_result = "abcdefgfaas";

    let launch_result = two_party_task_launch(
        &USER_ONE,
        user_file_path,
        &USER_TWO,
        collaborator_file_path,
        function_name,
    );
    assert_eq!(expected_result, launch_result.result.as_str());
}

pub fn test_swap_file() {
    trace!(">>>>> swap file");
    let user_file_path = "./test_data/file1.txt";
    let collaborator_file_path = "./test_data/file2.txt";
    let function_name = "swap_file";

    let launch_result = two_party_task_launch(
        &USER_ONE,
        user_file_path,
        &USER_TWO,
        collaborator_file_path,
        function_name,
    );

    // user reads shared result
    assert_file_content(&USER_ONE, &launch_result.result, b"abcdefgfaas");

    // user reads private result
    assert_private_result_content(&USER_ONE, &launch_result.task_id, b"faas");

    // collaborator reads shared result
    assert_shared_result_content(&USER_TWO, &launch_result.task_id, b"abcdefgfaas");

    // collaborator reads private result
    assert_private_result_content(&USER_TWO, &launch_result.task_id, b"abcdefg");
}
