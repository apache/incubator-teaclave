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

use crate::error::TeaclaveStorageError;
use std::prelude::v1::*;
use std::sync::mpsc::{channel, Sender};
use teaclave_proto::teaclave_storage_service::{TeaclaveStorageRequest, TeaclaveStorageResponse};
use teaclave_rpc::Request;
use teaclave_types::TeaclaveServiceResponseResult;

#[derive(Clone)]
pub(crate) struct ProxyService {
    sender: Sender<ProxyRequest>,
}

impl ProxyService {
    pub(crate) fn new(sender: Sender<ProxyRequest>) -> Self {
        Self { sender }
    }
}

impl teaclave_rpc::TeaclaveService<TeaclaveStorageRequest, TeaclaveStorageResponse>
    for ProxyService
{
    fn handle_request(
        &self,
        request: Request<TeaclaveStorageRequest>,
    ) -> TeaclaveServiceResponseResult<TeaclaveStorageResponse> {
        let (sender, receiver) = channel();
        self.sender
            .send(ProxyRequest { sender, request })
            .map_err(|_| TeaclaveStorageError::Connection)?;
        receiver
            .recv()
            .map_err(|_| TeaclaveStorageError::Connection)?
    }
}

#[derive(Clone)]
pub(crate) struct ProxyRequest {
    pub sender: Sender<TeaclaveServiceResponseResult<TeaclaveStorageResponse>>,
    pub request: Request<TeaclaveStorageRequest>,
}
