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

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::collections::VecDeque;
#[cfg(feature = "mesalock_sgx")]
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_rpc::transport::{channel::Endpoint, Channel};
use teaclave_rpc::Request;
use teaclave_types::{StagedTask, Storable, TeaclaveServiceResponseResult};
use thiserror::Error;

#[derive(Clone)]
pub(crate) struct PublisherService {
    storage_client: Arc<Mutex<TeaclaveStorageClient<Channel>>>,
    scheduler_client: Arc<Mutex<TeaclaveSchedulerClient<Channel>>>,
}

impl PublisherService {
    pub(crate) async fn new(
        storage_service_endpoint: Endpoint,
        scheduler_service_endpoint: Endpoint,
    ) -> Result<Self> {
        let channel = storage_service_endpoint.connect().await?;
        let storage_client = Arc::new(Mutex::new(TeaclaveStorageClient::new(channel)));
        let channel = scheduler_service_endpoint.connect().await?;

        let scheduler_client = Arc::new(Mutex::new(TeaclaveSchedulerClient::new(channel)));

        let service = Self {
            storage_client,
            scheduler_client,
        };

        Ok(service)
    }

    pub(crate) fn start(&mut self) {
        loop {
            thread::sleep(Duration::from_secs(10));
        }
    }
}

#[cfg(test_mode)]
mod test_mode {
    use super::*;
}
