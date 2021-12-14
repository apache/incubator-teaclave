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

#![cfg_attr(feature = "mesalock_sgx", no_std)]
#[cfg(feature = "mesalock_sgx")]
#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate log;
use anyhow::{anyhow, Result};

use rand::RngCore;
use std::prelude::v1::*;
use std::sync::{Arc, SgxRwLock as RwLock};
use std::thread;

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
    TeaclaveAuthenticationApiRequest, TeaclaveAuthenticationApiResponse,
    TeaclaveAuthenticationInternalRequest, TeaclaveAuthenticationInternalResponse,
};
use teaclave_rpc::config::SgxTrustedTlsServerConfig;
use teaclave_rpc::server::SgxTrustedTlsServer;
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_types::{EnclaveInfo, TeeServiceError, TeeServiceResult, UserRole};

mod api_service;
mod error;
mod internal_service;
mod user_db;
mod user_info;

fn start_internal_endpoint(
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
    )?;

    let mut server = SgxTrustedTlsServer::<
        TeaclaveAuthenticationInternalResponse,
        TeaclaveAuthenticationInternalRequest,
    >::new(addr, server_config);

    let service =
        internal_service::TeaclaveAuthenticationInternalService::new(db_client, jwt_secret);

    match server.start(service) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Service exit, error: {}.", e);
            Err(anyhow!("cannot start internal endpoint"))
        }
    }
}

fn start_api_endpoint(
    addr: std::net::SocketAddr,
    db_client: user_db::DbClient,
    jwt_secret: Vec<u8>,
    attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
) -> Result<()> {
    let server_config = SgxTrustedTlsServerConfig::from_attested_tls_config(attested_tls_config)?;

    let mut server = SgxTrustedTlsServer::<
        TeaclaveAuthenticationApiResponse,
        TeaclaveAuthenticationApiRequest,
    >::new(addr, server_config);

    let service = api_service::TeaclaveAuthenticationApiService::new(db_client, jwt_secret);

    match server.start(service) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Service exit, error: {}.", e);
            Err(anyhow!("cannot start API endpoint"))
        }
    }
}

fn start_service(config: &RuntimeConfig) -> Result<()> {
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
    let attestation_config = AttestationConfig::from_teaclave_config(&config)?;
    let attested_tls_config = RemoteAttestation::new(attestation_config)
        .generate_and_endorse()?
        .attested_tls_config()
        .ok_or_else(|| anyhow!("cannot get attested TLS config"))?;
    let database = user_db::Database::open()?;
    let mut api_jwt_secret = vec![0; user_info::JWT_SECRET_LEN];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut api_jwt_secret);
    let internal_jwt_secret = api_jwt_secret.to_owned();

    let attested_tls_config_ref = attested_tls_config.clone();
    {
        let client = database.get_client();
        create_platform_admin_user(client, "admin", "teaclave")?;
    }

    let client = database.get_client();
    let api_endpoint_thread_handler = thread::spawn(move || {
        let _ = start_api_endpoint(
            api_listen_address,
            client,
            api_jwt_secret,
            attested_tls_config_ref,
        );
    });

    let client = database.get_client();
    let internal_endpoint_thread_handler = thread::spawn(move || {
        let _ = start_internal_endpoint(
            internal_listen_address,
            client,
            internal_jwt_secret,
            attested_tls_config,
            accepted_enclave_attrs,
        );
    });

    api_endpoint_thread_handler
        .join()
        .expect("cannot join API endpoint thread");
    internal_endpoint_thread_handler
        .join()
        .expect("cannot join internal endpoint thread");

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
    match start_service(&input.config) {
        Ok(_) => Ok(StartServiceOutput),
        Err(e) => {
            log::error!("Failed to start the service: {}", e);
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
        run_tests!(
            api_service::tests::test_user_login,
            api_service::tests::test_user_register,
            api_service::tests::test_user_update,
            internal_service::tests::test_user_authenticate,
            internal_service::tests::test_invalid_algorithm,
            internal_service::tests::test_invalid_issuer,
            internal_service::tests::test_expired_token,
            internal_service::tests::test_invalid_user,
            internal_service::tests::test_wrong_secret,
        )
    }
}
