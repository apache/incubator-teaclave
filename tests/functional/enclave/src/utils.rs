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

use std::prelude::v1::*;
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::*;

pub(crate) fn get_scheduler_client() -> TeaclaveSchedulerClient {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let address = runtime_config
        .internal_endpoints
        .scheduler
        .advertised_address;
    let channel = Endpoint::new(&address).connect().unwrap();
    let metadata = hashmap!(
        "id" => "mock_user",
        "token" => "",
    );
    TeaclaveSchedulerClient::new_with_metadata(channel, metadata).unwrap()
}

pub(crate) fn get_storage_client() -> TeaclaveStorageClient {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let address = runtime_config.internal_endpoints.storage.advertised_address;
    let channel = Endpoint::new(&address).connect().unwrap();
    let metadata = hashmap!(
        "id" => "mock_user",
        "token" => "",
    );
    TeaclaveStorageClient::new_with_metadata(channel, metadata).unwrap()
}
