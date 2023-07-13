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

#[macro_use]
extern crate log;
extern crate sgx_types;
use anyhow::{anyhow, Result};

use rand::RngCore;
use std::sync::{Arc, RwLock};

use teaclave_attestation::{verifier, AttestationConfig, AttestedTlsConfig, RemoteAttestation};
use teaclave_binder::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_binder::{handle_ecall, register_ecall_handler};
use teaclave_config::build::{
    AS_ROOT_CA_CERT, AUDITOR_PUBLIC_KEYS, AUTHENTICATION_INBOUND_SERVICES,
};
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationApiServer, TeaclaveAuthenticationInternalServer,
};
use teaclave_rpc::{config::SgxTrustedTlsServerConfig, transport::Server};
use teaclave_service_enclave_utils::{base_dir_for_db, ServiceEnclave};
use teaclave_types::{EnclaveInfo, TeeServiceError, TeeServiceResult, UserRole};

mod api_service;
mod error;
mod internal_service;
mod user_db;
mod user_info;

async fn start_internal_endpoint(
    addr: std::net::SocketAddr,
    db_client: user_db::DbClient,
    jwt_secret: Vec<u8>,
    attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
    accepted_enclave_attrs: Vec<teaclave_types::EnclaveAttr>,
) -> Result<()> {
    let server_config = SgxTrustedTlsServerConfig::from_attested_tls_config(attested_tls_config)?
        .attestation_report_verifier(
            accepted_enclave_attrs,
            AS_ROOT_CA_CERT,
            verifier::universal_quote_verifier,
        )?
        .into();
    let service =
        internal_service::TeaclaveAuthenticationInternalService::new(db_client, jwt_secret);
    Server::builder()
        .tls_config(server_config)
        .map_err(|_| anyhow!("TeaclaveFrontendServer tls config error"))?
        .add_service(TeaclaveAuthenticationInternalServer::new_with_builtin_config(service))
        .serve(addr)
        .await?;
    Ok(())
}

async fn start_api_endpoint(
    addr: std::net::SocketAddr,
    db_client: user_db::DbClient,
    jwt_secret: Vec<u8>,
    attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
) -> Result<()> {
    let tls_config =
        SgxTrustedTlsServerConfig::from_attested_tls_config(attested_tls_config)?.into();

    let service = api_service::TeaclaveAuthenticationApiService::new(db_client, jwt_secret);
    Server::builder()
        .tls_config(tls_config)
        .map_err(|_| anyhow!("TeaclaveAuthenticationApiServer tls config error"))?
        .add_service(TeaclaveAuthenticationApiServer::new_with_builtin_config(
            service,
        ))
        .serve(addr)
        .await?;
    Ok(())
}

async fn start_service(config: &RuntimeConfig) -> Result<()> {
    info!("Starting Authentication...");

    let enclave_info = EnclaveInfo::verify_and_new(
        &config.audit.enclave_info_bytes,
        AUDITOR_PUBLIC_KEYS,
        &config.audit.auditor_signatures_bytes,
    )?;
    let accepted_enclave_attrs: Vec<teaclave_types::EnclaveAttr> = AUTHENTICATION_INBOUND_SERVICES
        .iter()
        .map(|service| match enclave_info.get_enclave_attr(service) {
            Some(attr) => Ok(attr),
            None => Err(anyhow!("cannot get enclave attribute of {}", service)),
        })
        .collect::<Result<_>>()?;
    let api_listen_address = config.api_endpoints.authentication.listen_address;
    let internal_listen_address = config.internal_endpoints.authentication.listen_address;
    let attestation_config = AttestationConfig::from_teaclave_config(config)?;
    let attested_tls_config = RemoteAttestation::new(attestation_config)
        .generate_and_endorse()?
        .attested_tls_config()
        .ok_or_else(|| anyhow!("cannot get attested TLS config"))?;

    info!(" Starting Authentication: Self attestation finished ...");

    let db_base = base_dir_for_db(config)?;
    let database = user_db::Database::open(&db_base)?;

    let mut api_jwt_secret = vec![0; user_info::JWT_SECRET_LEN];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut api_jwt_secret);
    let internal_jwt_secret = api_jwt_secret.to_owned();

    let attested_tls_config_ref = attested_tls_config.clone();
    {
        let client = database.get_client();
        if create_platform_admin_user(client, "admin", "teaclave").is_ok() {
            info!(" Starting Authentication: Platform first launch, admin user created ...");
        }
    }

    let client = database.get_client();
    let api_endpoint_thread_handler = tokio::spawn(start_api_endpoint(
        api_listen_address,
        client,
        api_jwt_secret,
        attested_tls_config_ref,
    ));

    info!(" Starting Authentication: setup API endpoint finished ...");

    let client = database.get_client();
    let internal_endpoint_thread_handler = tokio::spawn(start_internal_endpoint(
        internal_listen_address,
        client,
        internal_jwt_secret,
        attested_tls_config,
        accepted_enclave_attrs,
    ));
    info!(" Starting Authentication: setup Internal endpoint finished ...");

    let _ = api_endpoint_thread_handler
        .await
        .expect("cannot join API endpoint thread");
    let _ = internal_endpoint_thread_handler
        .await
        .expect("cannot join internal endpoint thread");

    info!(" Starting Authentication: start listening ...");
    Ok(())
}

pub(crate) fn create_platform_admin_user(
    db_client: user_db::DbClient,
    id: &str,
    password: &str,
) -> Result<()> {
    let new_user = user_info::UserInfo::new(id, password, UserRole::PlatformAdmin);
    db_client.create_user(&new_user)?;

    Ok(())
}

#[handle_ecall]
fn handle_start_service(input: &StartServiceInput) -> TeeServiceResult<StartServiceOutput> {
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| TeeServiceError::SgxError)?
        .block_on(start_service(&input.config));

    match result {
        Ok(_) => Ok(StartServiceOutput),
        Err(e) => {
            error!("Failed to run service: {}", e);
            Err(TeeServiceError::ServiceError)
        }
    }
}

#[handle_ecall]
fn handle_init_enclave(_: &InitEnclaveInput) -> TeeServiceResult<InitEnclaveOutput> {
    ServiceEnclave::init(env!("CARGO_PKG_NAME"))?;
    Ok(InitEnclaveOutput)
}

#[handle_ecall]
fn handle_finalize_enclave(_: &FinalizeEnclaveInput) -> TeeServiceResult<FinalizeEnclaveOutput> {
    ServiceEnclave::finalize()?;
    Ok(FinalizeEnclaveOutput)
}

register_ecall_handler!(
    type ECallCommand,
    (ECallCommand::StartService, StartServiceInput, StartServiceOutput),
    (ECallCommand::InitEnclave, InitEnclaveInput, InitEnclaveOutput),
    (ECallCommand::FinalizeEnclave, FinalizeEnclaveInput, FinalizeEnclaveOutput),
);

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_test_utils::*;

    pub fn run_tests() -> bool {
        run_async_tests!(
            api_service::tests::test_user_login,
            api_service::tests::test_user_register,
            api_service::tests::test_user_update,
            api_service::tests::test_user_change_password,
            api_service::tests::test_reset_user_password,
            api_service::tests::test_delete_user,
            internal_service::tests::test_user_authenticate,
            internal_service::tests::test_invalid_algorithm,
            internal_service::tests::test_invalid_issuer,
            internal_service::tests::test_expired_token,
            internal_service::tests::test_invalid_user,
            internal_service::tests::test_wrong_secret,
        )
    }
}
