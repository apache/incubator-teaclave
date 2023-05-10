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

use crate::utils::*;
use futures::FutureExt;
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_authentication_service::*;
use teaclave_proto::teaclave_common::*;
use teaclave_rpc::{
    transport::{Channel, Uri},
    CredentialService,
};
use teaclave_test_utils::async_test_case;
use teaclave_types::EnclaveInfo;
async fn get_api_client() -> TeaclaveAuthenticationApiClient<CredentialService> {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let enclave_info = EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes);
    create_authentication_api_client(&enclave_info, "https://localhost:7776")
        .await
        .unwrap()
}

async fn get_internal_client() -> TeaclaveAuthenticationInternalClient<CredentialService> {
    let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
    let enclave_info = EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes);
    let tls_config =
        create_client_config(&enclave_info, "teaclave_authentication_service").unwrap();
    let service_addr = runtime_config
        .internal_endpoints
        .authentication
        .advertised_address;
    let endpoint = Channel::builder(service_addr.parse::<Uri>().unwrap());
    let channel = endpoint
        .tls_config(tls_config)
        .unwrap()
        .connect()
        .await
        .unwrap();
    TeaclaveAuthenticationInternalClient::with_interceptor(
        channel,
        teaclave_rpc::UserCredential::default(),
    )
}

#[async_test_case]
async fn test_login_success() {
    let mut client = get_api_client_with_admin_credential().await;
    let request = UserRegisterRequest::new("test_login_id1", "test_password", "PlatformAdmin", "");
    let response_result = client.user_register(request).await;
    let _ = response_result.unwrap();

    let mut client = get_api_client().await;
    let request = UserLoginRequest::new("test_login_id1", "test_password");
    let response_result = client.user_login(request).await;
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());
}

#[async_test_case]
async fn test_login_fail() {
    let mut client = get_api_client_with_admin_credential().await;
    let request = UserRegisterRequest::new("test_login_id2", "test_password", "PlatformAdmin", "");
    let response_result = client.user_register(request).await;
    assert!(response_result.is_ok());

    let mut client = get_api_client().await;
    let request = UserLoginRequest::new("test_login_id2", "wrong_password");
    let response_result = client.user_login(request).await;
    debug!("{:?}", response_result);
    assert!(response_result.is_err());
}

#[async_test_case]
async fn test_authenticate_success() {
    let mut api_client = get_api_client_with_admin_credential().await;
    let request = UserRegisterRequest::new(
        "test_authenticate_id1",
        "test_password",
        "PlatformAdmin",
        "",
    );
    let response_result = api_client.user_register(request).await;
    assert!(response_result.is_ok());

    let mut api_client = get_api_client().await;
    let request = UserLoginRequest::new("test_authenticate_id1", "test_password");
    let response_result = api_client.user_login(request).await;
    assert!(response_result.is_ok());

    let mut internal_client = get_internal_client().await;
    let credential = UserCredential::new(
        "test_authenticate_id1",
        response_result.unwrap().into_inner().token,
    );
    let request = UserAuthenticateRequest::new(credential);
    let response_result = internal_client.user_authenticate(request).await;
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());
}

#[async_test_case]
async fn test_authenticate_fail() {
    let mut api_client = get_api_client_with_admin_credential().await;
    let mut internal_client = get_internal_client().await;

    let request = UserRegisterRequest::new(
        "test_authenticate_id2",
        "test_password",
        "PlatformAdmin",
        "",
    );
    let response_result = api_client.user_register(request).await;
    assert!(response_result.is_ok());

    let credential = UserCredential::new("test_authenticate_id2", "wrong_token");
    let request = UserAuthenticateRequest::new(credential);
    let response_result = internal_client.user_authenticate(request).await;
    debug!("{:?}", response_result);
    assert!(response_result.is_err());
}

#[async_test_case]
async fn test_register_success() {
    let mut client = get_api_client_with_admin_credential().await;
    let request =
        UserRegisterRequest::new("test_register_id1", "test_password", "PlatformAdmin", "");
    let response_result = client.user_register(request).await;
    debug!("{:?}", response_result);
    assert!(response_result.is_ok());
}

#[async_test_case]
async fn test_register_fail() {
    let mut client = get_api_client_with_admin_credential().await;
    let request =
        UserRegisterRequest::new("test_register_id2", "test_password", "PlatformAdmin", "");
    let response_result = client.user_register(request).await;
    assert!(response_result.is_ok());
    let request =
        UserRegisterRequest::new("test_register_id2", "test_password", "PlatformAdmin", "");
    let response_result = client.user_register(request).await;
    debug!("{:?}", response_result);
    assert!(response_result.is_err());
}
