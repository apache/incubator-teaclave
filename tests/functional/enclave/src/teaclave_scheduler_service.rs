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

use std::collections::HashMap;
use std::prelude::v1::*;
use teaclave_attestation::verifier;
use teaclave_config::RuntimeConfig;
use teaclave_config::BUILD_CONFIG;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::*;

pub fn run_tests() -> bool {
    use teaclave_test_utils::*;

    run_tests!(test_pull_task, test_update_task_status_result)
}

fn get_client(user_id: &str) -> TeaclaveSchedulerClient {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let enclave_info =
        EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes.as_ref().unwrap());
    let enclave_attr = enclave_info
        .get_enclave_attr("teaclave_scheduler_service")
        .expect("scheduler");
    let config = SgxTrustedTlsClientConfig::new().attestation_report_verifier(
        vec![enclave_attr],
        BUILD_CONFIG.as_root_ca_cert,
        verifier::universal_quote_verifier,
    );

    let channel = Endpoint::new(
        &runtime_config
            .internal_endpoints
            .scheduler
            .advertised_address,
    )
    .config(config)
    .connect()
    .unwrap();

    let mut metadata = HashMap::new();
    metadata.insert("id".to_string(), user_id.to_string());
    metadata.insert("token".to_string(), "".to_string());

    TeaclaveSchedulerClient::new_with_metadata(channel, metadata).unwrap()
}

fn test_pull_task() {
    let mut client = get_client("mock_user");
    let request = PullTaskRequest {};
    let response = client.pull_task(request);
    log::debug!("response: {:?}", response);
    assert!(response.is_ok());
}

fn test_update_task_status_result() {
    let mut client = get_client("mock_user");
    let request = PullTaskRequest {};
    let response = client.pull_task(request).unwrap();
    log::debug!("response: {:?}", response);
    let task_id = response.staged_task.task_id;

    let request = UpdateTaskStatusRequest::new(task_id, TaskStatus::Finished);
    let response = client.update_task_status(request);
    assert!(response.is_ok());

    let request =
        UpdateTaskResultRequest::new(task_id, "return".to_string().as_bytes(), HashMap::new());
    let response = client.update_task_result(request);

    assert!(response.is_ok());
}
