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
use teaclave_proto::teaclave_access_control_service::*;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::*;

macro_rules! impl_get_service_client_fn {
    ($service_name:ident, $fn_name:ident, $return:ident) => {
        pub(crate) fn $fn_name() -> $return {
            let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
            let address = runtime_config
                .internal_endpoints
                .$service_name
                .advertised_address;
            let channel = Endpoint::new(&address).connect().unwrap();
            let metadata = hashmap!(
                "id" => "mock_user",
                "token" => "",
            );
            $return::new_with_metadata(channel, metadata).unwrap()
        }
    };
}

impl_get_service_client_fn!(scheduler, get_scheduler_client, TeaclaveSchedulerClient);
impl_get_service_client_fn!(storage, get_storage_client, TeaclaveStorageClient);
impl_get_service_client_fn!(
    access_control,
    get_access_control_client,
    TeaclaveAccessControlClient
);
