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

use crate::utils::create_client_config;
use futures::FutureExt;
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_rpc::{
    transport::{Channel, Uri},
    CredentialService,
};
use teaclave_test_utils::async_test_case;

async fn get_client() -> TeaclaveStorageClient<CredentialService> {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let enclave_info =
        teaclave_types::EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes);
    let tls_config = create_client_config(&enclave_info, "teaclave_storage_service").unwrap();
    let service_addr = runtime_config.internal_endpoints.storage.advertised_address;
    let channel = Channel::builder(service_addr.parse::<Uri>().unwrap())
        .tls_config(tls_config)
        .unwrap()
        .connect()
        .await
        .unwrap();
    TeaclaveStorageClient::with_interceptor(channel, teaclave_rpc::UserCredential::default())
}

#[async_test_case]
async fn test_get_success() {
    let mut client = get_client().await;
    let request = GetRequest::new("test_get_key");
    let response_result = client.get(request).await;
    println!("{:?}", response_result);
    assert!(response_result.is_ok());
}

#[async_test_case]
async fn test_get_fail() {
    let mut client = get_client().await;
    let request = GetRequest::new("test_key_not_exist");
    let response_result = client.get(request).await;
    assert!(response_result.is_err());
}

#[async_test_case]
async fn test_put_success() {
    let mut client = get_client().await;
    let request = PutRequest::new("test_put_key", "test_put_value");
    let response_result = client.put(request).await;
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());

    let request = GetRequest::new("test_put_key");
    let response_result = client.get(request).await;
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());
    assert_eq!(
        response_result.unwrap().into_inner().value,
        b"test_put_value"
    );
}

#[async_test_case]
async fn test_delete_success() {
    let mut client = get_client().await;
    let request = DeleteRequest::new("test_delete_key");
    let response_result = client.delete(request).await;
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());

    let request = GetRequest::new("test_delete_key");
    let response_result = client.get(request).await;
    assert!(response_result.is_err());
}

#[async_test_case]
async fn test_enqueue_success() {
    let mut client = get_client().await;
    let request = EnqueueRequest::new("test_enqueue_key", "test_enqueue_value");
    let response_result = client.enqueue(request).await;
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());
}

#[async_test_case]
async fn test_dequeue_success() {
    let mut client = get_client().await;
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request).await;
    assert!(response_result.is_err());
    let request = EnqueueRequest::new("test_dequeue_key", "1");
    let response_result = client.enqueue(request).await;
    assert!(response_result.is_ok());
    let request = EnqueueRequest::new("test_dequeue_key", "2");
    let response_result = client.enqueue(request).await;
    assert!(response_result.is_ok());
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request).await;
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().into_inner().value, b"1");
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request).await;
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().into_inner().value, b"2");
}

#[async_test_case]
async fn test_dequeue_fail() {
    let mut client = get_client().await;
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request).await;
    assert!(response_result.is_err());

    let request = EnqueueRequest::new("test_dequeue_key", "1");
    let response_result = client.enqueue(request).await;
    assert!(response_result.is_ok());
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request).await;
    assert!(response_result.is_ok());
    assert_eq!(response_result.unwrap().into_inner().value, b"1");
    let request = DequeueRequest::new("test_dequeue_key");
    let response_result = client.dequeue(request).await;
    assert!(response_result.is_err());
}
