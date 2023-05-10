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

use anyhow::Result;
use lazy_static::lazy_static;
use teaclave_attestation::verifier;
use teaclave_config::build::AS_ROOT_CA_CERT;
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_access_control_service::*;
use teaclave_proto::teaclave_authentication_service::*;
use teaclave_proto::teaclave_common::*;
use teaclave_proto::teaclave_frontend_service::*;
use teaclave_proto::teaclave_management_service::*;
use teaclave_proto::teaclave_scheduler_service::*;
use teaclave_proto::teaclave_storage_service::*;
use teaclave_rpc::transport::{Channel, ClientTlsConfig, Uri};
use teaclave_rpc::CredentialService;
use teaclave_types::*;

macro_rules! impl_get_internal_service_client_fn {
    ($service_name:ident, $fn_name:ident, $return:ty) => {
        pub(crate) async fn $fn_name(username: &str) -> $return {
            let runtime_config = RuntimeConfig::from_toml("runtime.config.toml").expect("runtime");
            let address = runtime_config
                .internal_endpoints
                .$service_name
                .advertised_address;
            let dst = address.parse::<Uri>().unwrap();
            let dst = if dst.scheme().is_none() {
                format!("https://{}", address).parse().unwrap()
            } else {
                dst
            };
            let endpoint = Channel::builder(dst);
            let tls_config = teaclave_rpc::config::SgxTrustedTlsClientConfig::new().into();
            let channel = endpoint
                .tls_config(tls_config)
                .unwrap()
                .connect()
                .await
                .unwrap();
            let cred =
                teaclave_rpc::UserCredential::with_role(username, "", UserRole::PlatformAdmin);
            <$return>::new(teaclave_rpc::InterceptedService::new(channel, cred))
        }
    };
}

impl_get_internal_service_client_fn!(
    management,
    get_management_client,
    TeaclaveManagementClient<CredentialService>
);
impl_get_internal_service_client_fn!(
    scheduler,
    get_scheduler_client_internal,
    TeaclaveSchedulerClient<CredentialService>
);
impl_get_internal_service_client_fn!(
    storage,
    get_storage_client_internal,
    TeaclaveStorageClient<CredentialService>
);
impl_get_internal_service_client_fn!(
    access_control,
    get_access_control_client_internal,
    TeaclaveAccessControlClient<CredentialService>
);

pub async fn get_scheduler_client() -> TeaclaveSchedulerClient<CredentialService> {
    get_scheduler_client_internal("mock_user").await
}

pub async fn get_storage_client() -> TeaclaveStorageClient<CredentialService> {
    get_storage_client_internal("mock_user").await
}

pub async fn get_access_control_client() -> TeaclaveAccessControlClient<CredentialService> {
    get_access_control_client_internal("mock_user").await
}

pub const CONFIG_FILE: &str = "runtime.config.toml";
pub const AUTH_SERVICE_ADDR: &str = "https://localhost:7776";
pub const FRONTEND_SERVICE_ADDR: &str = "https://localhost:7777";

lazy_static! {
    static ref ENCLAVE_INFO: EnclaveInfo = {
        let runtime_config = RuntimeConfig::from_toml(CONFIG_FILE).expect("runtime config");
        EnclaveInfo::from_bytes(&runtime_config.audit.enclave_info_bytes)
    };
}

pub async fn get_api_client_with_admin_credential(
) -> TeaclaveAuthenticationApiClient<CredentialService> {
    create_authentication_api_client_with_credential(
        shared_enclave_info(),
        AUTH_SERVICE_ADDR,
        "admin",
        "teaclave",
    )
    .await
    .unwrap()
}

pub fn shared_enclave_info() -> &'static EnclaveInfo {
    &ENCLAVE_INFO
}

pub fn create_client_config(
    enclave_info: &EnclaveInfo,
    service_name: &str,
) -> Result<ClientTlsConfig> {
    let enclave_attr = enclave_info
        .get_enclave_attr(service_name)
        .expect("enclave attr");
    let config = teaclave_rpc::config::SgxTrustedTlsClientConfig::new()
        .attestation_report_verifier(
            vec![enclave_attr],
            AS_ROOT_CA_CERT,
            verifier::universal_quote_verifier,
        )
        .into();
    Ok(config)
}

pub async fn create_frontend_client(
    enclave_info: &EnclaveInfo,
    service_addr: &str,
    cred: UserCredential,
) -> Result<TeaclaveFrontendClient<CredentialService>> {
    let tls_config = create_client_config(enclave_info, "teaclave_frontend_service")?;
    let endpoint = Channel::builder(service_addr.parse::<Uri>()?);
    let channel = endpoint
        .tls_config(tls_config)
        .unwrap()
        .connect()
        .await
        .unwrap();
    let cred = teaclave_rpc::UserCredential::new(cred.id, cred.token);
    let client = TeaclaveFrontendClient::with_interceptor(channel, cred);
    Ok(client)
}

pub async fn create_authentication_api_client(
    enclave_info: &EnclaveInfo,
    service_addr: &str,
) -> Result<TeaclaveAuthenticationApiClient<CredentialService>> {
    let tls_config = create_client_config(enclave_info, "teaclave_authentication_service")?;
    let endpoint = Channel::builder(service_addr.parse::<Uri>()?);
    let channel = endpoint.tls_config(tls_config).unwrap().connect_lazy();
    let client = TeaclaveAuthenticationApiClient::with_interceptor(
        channel,
        teaclave_rpc::UserCredential::default(),
    );
    Ok(client)
}

pub async fn create_authentication_api_client_with_credential(
    enclave_info: &EnclaveInfo,
    service_addr: &str,
    username: &str,
    password: &str,
) -> Result<TeaclaveAuthenticationApiClient<CredentialService>> {
    let tls_config = create_client_config(enclave_info, "teaclave_authentication_service")?;
    let endpoint = Channel::builder(service_addr.parse::<Uri>()?);
    let channel = endpoint
        .tls_config(tls_config)
        .unwrap()
        .connect()
        .await
        .unwrap();

    let mut client = TeaclaveAuthenticationApiClient::new(channel.clone());
    let request = UserLoginRequest::new(username, password);
    let token = client.user_login(request).await?.into_inner().token;
    let cred = teaclave_rpc::UserCredential::new(username, token);
    let client = TeaclaveAuthenticationApiClient::with_interceptor(channel, cred);
    Ok(client)
}

pub async fn register_new_account(
    api_client: &mut TeaclaveAuthenticationApiClient<CredentialService>,
    username: &str,
    password: &str,
    role: &str,
    attribute: &str,
) -> Result<()> {
    let request = UserRegisterRequest::new(username, password, role, attribute);
    let response = api_client.user_register(request).await;
    log::debug!("User register: {:?}", response);
    Ok(())
}

pub async fn login(
    api_client: &mut TeaclaveAuthenticationApiClient<CredentialService>,
    username: &str,
    password: &str,
) -> Result<UserCredential> {
    let request = UserLoginRequest::new(username, password);
    let response = api_client.user_login(request).await?.into_inner();
    log::debug!("User login: {:?}", response);
    Ok(UserCredential::new(username, response.token))
}

pub const USERNAME: &str = "frontend_user";
pub const USERNAME1: &str = "frontend_user1";
pub const USERNAME2: &str = "frontend_user2";
pub const USERNAME3: &str = "frontend_user3";

pub const TEST_PASSWORD: &str = "test_password";

pub async fn setup() {
    // Register user for the first time
    let mut api_client = get_api_client_with_admin_credential().await;

    // Ignore error if register failed.
    for uname in vec![USERNAME, USERNAME1, USERNAME2, USERNAME3].iter() {
        let _ =
            register_new_account(&mut api_client, uname, TEST_PASSWORD, "PlatformAdmin", "").await;
    }
}
