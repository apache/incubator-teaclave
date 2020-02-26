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
use teaclave_config::{RuntimeConfig, BUILD_CONFIG};
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationApiRequest, TeaclaveAuthenticationApiResponse,
    TeaclaveAuthenticationInternalRequest, TeaclaveAuthenticationInternalResponse,
};
use teaclave_rpc::config::SgxTrustedTlsServerConfig;
use teaclave_rpc::server::SgxTrustedTlsServer;
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_types::{EnclaveInfo, TeeServiceError, TeeServiceResult};

mod api_service;
mod internal_service;
mod user_db;
mod user_info;

const AS_ROOT_CA_CERT: &[u8] = BUILD_CONFIG.as_root_ca_cert;
const AUDITOR_PUBLIC_KEYS_LEN: usize = BUILD_CONFIG.auditor_public_keys.len();
const AUDITOR_PUBLIC_KEYS: &[&[u8]; AUDITOR_PUBLIC_KEYS_LEN] = BUILD_CONFIG.auditor_public_keys;
const INBOUND_SERVICES_LEN: usize = BUILD_CONFIG.inbound.authentication.len();
const INBOUND_SERVICES: &[&str; INBOUND_SERVICES_LEN] = BUILD_CONFIG.inbound.authentication;

fn start_internal_endpoint(
    addr: std::net::SocketAddr,
    db_client: user_db::DbClient,
    jwt_secret: Vec<u8>,
    attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
    accepted_enclave_attrs: Vec<teaclave_types::EnclaveAttr>,
) {
    let server_config = SgxTrustedTlsServerConfig::from_attested_tls_config(attested_tls_config)
        .unwrap()
        .attestation_report_verifier(
            accepted_enclave_attrs,
            AS_ROOT_CA_CERT,
            verifier::universal_quote_verifier,
        )
        .unwrap();

    let mut server = SgxTrustedTlsServer::<
        TeaclaveAuthenticationInternalResponse,
        TeaclaveAuthenticationInternalRequest,
    >::new(addr, server_config);

    let service =
        internal_service::TeaclaveAuthenticationInternalService::new(db_client, jwt_secret);

    match server.start(service) {
        Ok(_) => (),
        Err(e) => {
            error!("Service exit, error: {}.", e);
        }
    }
}

fn start_api_endpoint(
    addr: std::net::SocketAddr,
    db_client: user_db::DbClient,
    jwt_secret: Vec<u8>,
    attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
) {
    let server_config =
        SgxTrustedTlsServerConfig::from_attested_tls_config(attested_tls_config).unwrap();

    let mut server = SgxTrustedTlsServer::<
        TeaclaveAuthenticationApiResponse,
        TeaclaveAuthenticationApiRequest,
    >::new(addr, server_config);

    let service = api_service::TeaclaveAuthenticationApiService::new(db_client, jwt_secret);

    match server.start(service) {
        Ok(_) => (),
        Err(e) => {
            error!("Service exit, error: {}.", e);
        }
    }
}

fn start_service(config: &RuntimeConfig) -> anyhow::Result<()> {
    let enclave_info = EnclaveInfo::verify_and_new(
        config
            .audit
            .enclave_info_bytes
            .as_ref()
            .expect("enclave_info"),
        AUDITOR_PUBLIC_KEYS,
        config
            .audit
            .auditor_signatures_bytes
            .as_ref()
            .expect("auditor signatures"),
    )?;
    let accepted_enclave_attrs: Vec<teaclave_types::EnclaveAttr> = INBOUND_SERVICES
        .iter()
        .map(|service| {
            enclave_info
                .get_enclave_attr(service)
                .expect("enclave_info")
        })
        .collect();
    let api_listen_address = config.api_endpoints.authentication.listen_address;
    let internal_listen_address = config.internal_endpoints.authentication.listen_address;
    let as_config = &config.attestation;
    let attestation_config = AttestationConfig::new(
        &as_config.algorithm,
        &as_config.url,
        &as_config.key,
        &as_config.spid,
    );
    let attested_tls_config = RemoteAttestation::new()
        .config(attestation_config)
        .generate_and_endorse()
        .unwrap()
        .attested_tls_config()
        .unwrap();
    let database = user_db::Database::open()?;
    let mut api_jwt_secret = vec![0; user_info::JWT_SECRET_LEN];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut api_jwt_secret);
    let internal_jwt_secret = api_jwt_secret.to_owned();

    let attested_tls_config_ref = attested_tls_config.clone();
    let client = database.get_client();
    let api_endpoint_thread_handler = thread::spawn(move || {
        start_api_endpoint(
            api_listen_address,
            client,
            api_jwt_secret,
            attested_tls_config_ref,
        );
    });

    let client = database.get_client();
    let internal_endpoint_thread_handler = thread::spawn(move || {
        start_internal_endpoint(
            internal_listen_address,
            client,
            internal_jwt_secret,
            attested_tls_config,
            accepted_enclave_attrs,
        );
    });

    api_endpoint_thread_handler.join().unwrap();
    internal_endpoint_thread_handler.join().unwrap();

    Ok(())
}

#[handle_ecall]
fn handle_start_service(input: &StartServiceInput) -> TeeServiceResult<StartServiceOutput> {
    start_service(&input.config).map_err(|_| TeeServiceError::ServiceError)?;
    Ok(StartServiceOutput)
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
            internal_service::tests::test_user_authenticate,
            internal_service::tests::test_invalid_algorithm,
            internal_service::tests::test_invalid_issuer,
            internal_service::tests::test_expired_token,
            internal_service::tests::test_invalid_user,
            internal_service::tests::test_wrong_secret,
        )
    }
}
