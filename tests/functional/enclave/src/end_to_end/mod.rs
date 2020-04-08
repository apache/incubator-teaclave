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
use teaclave_proto::teaclave_frontend_service::*;
use teaclave_types::*;
use url::Url;

mod mesapy_echo;
mod native_echo;
mod native_gbdt_training;

const USERNAME: &str = "alice";
const PASSWORD: &str = "daHosldOdker0sS";

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;
    setup();
    run_tests!(
        native_gbdt_training::test_gbdt_training_task,
        native_echo::test_echo_task_success,
        mesapy_echo::test_echo_task_success,
    )
}

fn setup() {
    // Register user for the first time
    let mut api_client =
        create_authentication_api_client(shared_enclave_info(), AUTH_SERVICE_ADDR).unwrap();
    register_new_account(&mut api_client, USERNAME, PASSWORD).unwrap();
}
