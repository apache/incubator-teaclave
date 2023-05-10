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

use crate::teaclave_storage_service_proto as proto;
pub use proto::teaclave_storage_client::TeaclaveStorageClient;
pub use proto::teaclave_storage_server::TeaclaveStorage;
pub use proto::teaclave_storage_server::TeaclaveStorageServer;
pub use proto::{
    DeleteRequest, DeleteResponse, DequeueRequest, DequeueResponse, EnqueueRequest,
    EnqueueResponse, GetKeysByPrefixRequest, GetKeysByPrefixResponse, GetRequest, GetResponse,
    PutRequest, PutResponse,
};

impl GetRequest {
    pub fn new(key: impl Into<Vec<u8>>) -> Self {
        Self { key: key.into() }
    }
}

impl GetResponse {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl PutRequest {
    pub fn new(key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl DeleteRequest {
    pub fn new(key: impl Into<Vec<u8>>) -> Self {
        Self { key: key.into() }
    }
}

impl EnqueueRequest {
    pub fn new(key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl DequeueRequest {
    pub fn new(key: impl Into<Vec<u8>>) -> Self {
        Self { key: key.into() }
    }
}

impl DequeueResponse {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

impl GetKeysByPrefixRequest {
    pub fn new(prefix: impl Into<Vec<u8>>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl GetKeysByPrefixResponse {
    pub fn new(keys: Vec<Vec<u8>>) -> Self {
        Self { keys }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TeaclaveStorageRequest {
    Get(GetRequest),
    Put(PutRequest),
    Delete(DeleteRequest),
    Enqueue(EnqueueRequest),
    Dequeue(DequeueRequest),
    GetKeysByPrefix(GetKeysByPrefixRequest),
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "response", content = "content", rename_all = "snake_case")]
pub enum TeaclaveStorageResponse {
    Get(GetResponse),
    Put(PutResponse),
    Delete(DeleteResponse),
    Enqueue(EnqueueResponse),
    Dequeue(DequeueResponse),
    GetKeysByPrefix(GetKeysByPrefixResponse),
}
