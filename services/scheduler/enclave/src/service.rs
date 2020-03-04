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

#![allow(unused_imports)]
#![allow(unused_variables)]

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;
use std::sync::{Arc, SgxMutex as Mutex};

use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::Request;
use teaclave_service_enclave_utils::teaclave_service;
use teaclave_types::{
    StagedTask, Storable, TeaclaveServiceResponseError, TeaclaveServiceResponseResult,
};

use anyhow::Result;
use thiserror::Error;

#[teaclave_service(teaclave_scheduler_service, TeaclaveScheduler, TeaclaveSchedulerError)]
#[derive(Clone)]
pub(crate) struct TeaclaveSchedulerService {
    storage_client: Arc<Mutex<TeaclaveStorageClient>>,
}

impl TeaclaveSchedulerService {
    pub(crate) fn new(storage_service_endpoint: Endpoint) -> Result<Self> {
        let mut i = 0;
        let channel = loop {
            match storage_service_endpoint.connect() {
                Ok(channel) => break channel,
                Err(_) => {
                    anyhow::ensure!(i < 3, "failed to connect to storage service");
                    log::debug!("Failed to connect to storage service, retry {}", i);
                    i += 1;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        };
        let storage_client = Arc::new(Mutex::new(TeaclaveStorageClient::new(channel)?));
        let service = Self { storage_client };

        Ok(service)
    }
}

impl TeaclaveScheduler for TeaclaveSchedulerService {
    // Publisher
    fn publish_task(
        &self,
        request: Request<PublishTaskRequest>,
    ) -> TeaclaveServiceResponseResult<PublishTaskResponse> {
        unimplemented!()
    }

    // Subscriber
    fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> TeaclaveServiceResponseResult<SubscribeResponse> {
        unimplemented!()
    }

    fn pull_task(
        &self,
        request: Request<PullTaskRequest>,
    ) -> TeaclaveServiceResponseResult<PullTaskResponse> {
        unimplemented!()
    }

    fn update_task(
        &self,
        request: Request<UpdateTaskRequest>,
    ) -> TeaclaveServiceResponseResult<UpdateTaskResponse> {
        unimplemented!()
    }
}

#[cfg(test_mode)]
mod test_mode {
    use super::*;
}
