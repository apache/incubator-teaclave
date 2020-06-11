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
use teaclave_proto::teaclave_storage_service::*;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_test_utils::test_case;

fn get_client() -> TeaclaveStorageClient {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let channel = Endpoint::new(&runtime_config.internal_endpoints.storage.advertised_address)
        .connect()
        .unwrap();
    TeaclaveStorageClient::new(channel).unwrap()
}

#[test_case]
fn test_get_success() {
    let mut client = get_client();
    let request = GetRequest::new("test_get_key");
    let response_result = client.get(request);
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());
}

#[test_case]
fn test_get_fail() {
    let mut client = get_client();
    let request = GetRequest::new("test_key_not_exist");
    let response_result = client.get(request);
    assert!(response_result.is_err());
}

#[test_case]
fn test_put_success() {
    let mut client = get_client();
    let request = PutRequest::new("test_put_key", "test_put_value");
    let response_result = client.put(request);
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());

    let request = GetRequest::new("test_put_key");
    let response_result = client.get(request);
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"test_put_value");
}

#[test_case]
fn test_delete_success() {
    let mut client = get_client();
    let request = DeleteRequest::new("test_delete_key");
    let response_result = client.delete(request);
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());

    let request = GetRequest::new("test_delete_key");
    let response_result = client.get(request);
    assert!(response_result.is_err());
}

#[test_case]
fn test_enqueue_success() {
    let mut client = get_client();
    let request = EnqueueRequest::new("test_enqueue_key", "test_enqueue_value");
    let response_result = client.enqueue(request);
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());
}

#[test_case]
fn test_dequeue_success() {
    let mut client = get_client();
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request);
    assert!(response_result.is_err());
    let request = EnqueueRequest::new("test_dequeue_key", "1");
    let response_result = client.enqueue(request);
    assert!(response_result.is_ok());
    let request = EnqueueRequest::new("test_dequeue_key", "2");
    let response_result = client.enqueue(request);
    assert!(response_result.is_ok());
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"1");
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"2");
}

#[test_case]
fn test_dequeue_fail() {
    let mut client = get_client();
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request);
    assert!(response_result.is_err());

    let request = EnqueueRequest::new("test_dequeue_key", "1");
    let response_result = client.enqueue(request);
    assert!(response_result.is_ok());
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request);
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().value, b"1");
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request);
    assert!(response_result.is_err());
}
