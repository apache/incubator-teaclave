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

use anyhow::{Error, Result};
use std::prelude::v1::*;

use crate::teaclave_storage_service_proto as proto;
pub use proto::TeaclaveStorage;
pub use proto::TeaclaveStorageClient;
pub use proto::TeaclaveStorageRequest;
pub use proto::TeaclaveStorageResponse;
use teaclave_rpc::into_request;

#[into_request(TeaclaveStorageRequest::Get)]
#[derive(Debug)]
pub struct GetRequest {
    pub key: Vec<u8>,
}

impl GetRequest {
    pub fn new(key: impl Into<Vec<u8>>) -> Self {
        Self { key: key.into() }
    }
}

#[into_request(TeaclaveStorageResponse::Get)]
#[derive(Debug)]
pub struct GetResponse {
    pub value: Vec<u8>,
}

impl GetResponse {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[into_request(TeaclaveStorageRequest::Put)]
#[derive(Debug)]
pub struct PutRequest {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl PutRequest {
    pub fn new(key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[into_request(TeaclaveStorageResponse::Put)]
#[derive(Debug, Default)]
pub struct PutResponse;

#[into_request(TeaclaveStorageRequest::Delete)]
#[derive(Debug)]
pub struct DeleteRequest {
    pub key: Vec<u8>,
}

impl DeleteRequest {
    pub fn new(key: impl Into<Vec<u8>>) -> Self {
        Self { key: key.into() }
    }
}

#[into_request(TeaclaveStorageResponse::Delete)]
#[derive(Debug, Default)]
pub struct DeleteResponse;

#[into_request(TeaclaveStorageRequest::Enqueue)]
#[derive(Debug)]
pub struct EnqueueRequest {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl EnqueueRequest {
    pub fn new(key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[into_request(TeaclaveStorageResponse::Enqueue)]
#[derive(Debug, Default)]
pub struct EnqueueResponse;

#[into_request(TeaclaveStorageRequest::Dequeue)]
#[derive(Debug)]
pub struct DequeueRequest {
    pub key: Vec<u8>,
}

impl DequeueRequest {
    pub fn new(key: impl Into<Vec<u8>>) -> Self {
        Self { key: key.into() }
    }
}

#[into_request(TeaclaveStorageResponse::Dequeue)]
#[derive(Debug)]
pub struct DequeueResponse {
    pub value: Vec<u8>,
}

impl DequeueResponse {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl std::convert::TryFrom<proto::GetRequest> for GetRequest {
    type Error = Error;

    fn try_from(proto: proto::GetRequest) -> Result<Self> {
        let ret = Self { key: proto.key };

        Ok(ret)
    }
}

impl From<GetRequest> for proto::GetRequest {
    fn from(request: GetRequest) -> Self {
        Self { key: request.key }
    }
}

impl std::convert::TryFrom<proto::GetResponse> for GetResponse {
    type Error = Error;

    fn try_from(proto: proto::GetResponse) -> Result<Self> {
        let ret = Self { value: proto.value };

        Ok(ret)
    }
}

impl From<GetResponse> for proto::GetResponse {
    fn from(response: GetResponse) -> Self {
        Self {
            value: response.value,
        }
    }
}

impl std::convert::TryFrom<proto::PutRequest> for PutRequest {
    type Error = Error;

    fn try_from(proto: proto::PutRequest) -> Result<Self> {
        let ret = Self {
            key: proto.key,
            value: proto.value,
        };

        Ok(ret)
    }
}

impl From<PutRequest> for proto::PutRequest {
    fn from(request: PutRequest) -> Self {
        Self {
            key: request.key,
            value: request.value,
        }
    }
}

impl std::convert::TryFrom<proto::PutResponse> for PutResponse {
    type Error = Error;

    fn try_from(_proto: proto::PutResponse) -> Result<Self> {
        Ok(Self {})
    }
}

impl From<PutResponse> for proto::PutResponse {
    fn from(_response: PutResponse) -> Self {
        Self {}
    }
}

impl std::convert::TryFrom<proto::DeleteRequest> for DeleteRequest {
    type Error = Error;

    fn try_from(proto: proto::DeleteRequest) -> Result<Self> {
        let ret = Self { key: proto.key };

        Ok(ret)
    }
}

impl From<DeleteRequest> for proto::DeleteRequest {
    fn from(request: DeleteRequest) -> Self {
        Self { key: request.key }
    }
}

impl std::convert::TryFrom<proto::DeleteResponse> for DeleteResponse {
    type Error = Error;

    fn try_from(_proto: proto::DeleteResponse) -> Result<Self> {
        Ok(Self {})
    }
}

impl From<DeleteResponse> for proto::DeleteResponse {
    fn from(_response: DeleteResponse) -> Self {
        Self {}
    }
}

impl std::convert::TryFrom<proto::EnqueueRequest> for EnqueueRequest {
    type Error = Error;

    fn try_from(proto: proto::EnqueueRequest) -> Result<Self> {
        let ret = Self {
            key: proto.key,
            value: proto.value,
        };

        Ok(ret)
    }
}

impl From<EnqueueRequest> for proto::EnqueueRequest {
    fn from(request: EnqueueRequest) -> Self {
        Self {
            key: request.key,
            value: request.value,
        }
    }
}

impl std::convert::TryFrom<proto::EnqueueResponse> for EnqueueResponse {
    type Error = Error;

    fn try_from(_proto: proto::EnqueueResponse) -> Result<Self> {
        Ok(Self {})
    }
}

impl From<EnqueueResponse> for proto::EnqueueResponse {
    fn from(_response: EnqueueResponse) -> Self {
        Self {}
    }
}

impl std::convert::TryFrom<proto::DequeueRequest> for DequeueRequest {
    type Error = Error;

    fn try_from(proto: proto::DequeueRequest) -> Result<Self> {
        let ret = Self { key: proto.key };

        Ok(ret)
    }
}

impl From<DequeueRequest> for proto::DequeueRequest {
    fn from(request: DequeueRequest) -> Self {
        Self { key: request.key }
    }
}

impl std::convert::TryFrom<proto::DequeueResponse> for DequeueResponse {
    type Error = Error;

    fn try_from(proto: proto::DequeueResponse) -> Result<Self> {
        Ok(Self { value: proto.value })
    }
}

impl From<DequeueResponse> for proto::DequeueResponse {
    fn from(response: DequeueResponse) -> Self {
        Self {
            value: response.value,
        }
    }
}
