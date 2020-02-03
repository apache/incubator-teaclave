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

use anyhow::Result;
use rand::RngCore;
use std::prelude::v1::*;
use std::sync::Arc;
use std::thread;
use teaclave_attestation::verifier;
use teaclave_attestation::{AttestationConfig, RemoteAttestation};
use teaclave_config::BUILD_CONFIG;
use teaclave_ipc::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_ipc::{handle_ecall, register_ecall_handler};
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationApiRequest, TeaclaveAuthenticationApiResponse,
    TeaclaveAuthenticationInternalRequest, TeaclaveAuthenticationInternalResponse,
};
use teaclave_rpc::config::SgxTrustedTlsServerConfig;
use teaclave_rpc::server::SgxTrustedTlsServer;
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_types::EnclaveInfo;

mod api_service;
mod internal_service;
mod user_db;
mod user_info;

fn start_internal_endpoint(
    addr: std::net::SocketAddr,
    db_client: user_db::DbClient,
    jwt_secret: Vec<u8>,
    attestation: Arc<RemoteAttestation>,
    accepted_enclave_attrs: Vec<teaclave_types::EnclaveAttr>,
) {
    let config = if cfg!(test_mode) {
        SgxTrustedTlsServerConfig::new_without_verifier(&attestation.cert, &attestation.private_key)
            .unwrap()
    } else {
        SgxTrustedTlsServerConfig::new_with_attestation_report_verifier(
            accepted_enclave_attrs,
            &attestation.cert,
            &attestation.private_key,
            BUILD_CONFIG.ias_root_ca_cert,
            verifier::universal_quote_verifier,
        )
        .unwrap()
    };

    let mut server = SgxTrustedTlsServer::<
        TeaclaveAuthenticationInternalResponse,
        TeaclaveAuthenticationInternalRequest,
    >::new(addr, &config);

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
    attestation: Arc<RemoteAttestation>,
) {
    let config = SgxTrustedTlsServerConfig::new_without_verifier(
        &attestation.cert,
        &attestation.private_key,
    )
    .unwrap();

    let mut server = SgxTrustedTlsServer::<
        TeaclaveAuthenticationApiResponse,
        TeaclaveAuthenticationApiRequest,
    >::new(addr, &config);

    let service = api_service::TeaclaveAuthenticationApiService::new(db_client, jwt_secret);

    match server.start(service) {
        Ok(_) => (),
        Err(e) => {
            error!("Service exit, error: {}.", e);
        }
    }
}

#[handle_ecall]
fn handle_start_service(args: &StartServiceInput) -> Result<StartServiceOutput> {
    debug!("handle_start_service");
    let enclave_info = EnclaveInfo::verify_and_new(
        args.config
            .audit
            .enclave_info_bytes
            .as_ref()
            .expect("enclave_info"),
        BUILD_CONFIG.auditor_public_keys,
        args.config
            .audit
            .auditor_signatures_bytes
            .as_ref()
            .expect("auditor signatures"),
    )?;
    let inbound_services = args
        .config
        .internal_endpoints
        .authentication
        .inbound_services
        .as_ref()
        .expect("inbound_service");
    let accepted_enclave_attrs: Vec<teaclave_types::EnclaveAttr> = inbound_services
        .iter()
        .map(|service| {
            enclave_info
                .get_enclave_attr(&format!("teaclave_{}_service", service))
                .expect("enclave_info")
        })
        .collect();
    let api_listen_address = args.config.api_endpoints.authentication.listen_address;
    let internal_listen_address = args.config.internal_endpoints.authentication.listen_address;
    let ias_config = args.config.ias.as_ref().unwrap();
    let attestation = Arc::new(
        RemoteAttestation::generate_and_endorse(&AttestationConfig::ias(
            &ias_config.ias_key,
            &ias_config.ias_spid,
        ))
        .unwrap(),
    );
    let database = user_db::Database::open()?;
    let mut api_jwt_secret = vec![0; user_info::JWT_SECRET_LEN];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut api_jwt_secret);
    let internal_jwt_secret = api_jwt_secret.to_owned();

    let attestation_ref = attestation.clone();
    let client = database.get_client();
    let api_endpoint_thread_handler = thread::spawn(move || {
        start_api_endpoint(api_listen_address, client, api_jwt_secret, attestation_ref);
    });

    let client = database.get_client();
    let internal_endpoint_thread_handler = thread::spawn(move || {
        start_internal_endpoint(
            internal_listen_address,
            client,
            internal_jwt_secret,
            attestation,
            accepted_enclave_attrs,
        );
    });

    api_endpoint_thread_handler.join().unwrap();
    internal_endpoint_thread_handler.join().unwrap();

    Ok(StartServiceOutput::default())
}

#[handle_ecall]
fn handle_init_enclave(_args: &InitEnclaveInput) -> Result<InitEnclaveOutput> {
    ServiceEnclave::init(env!("CARGO_PKG_NAME"))?;
    Ok(InitEnclaveOutput::default())
}

#[handle_ecall]
fn handle_finalize_enclave(_args: &FinalizeEnclaveInput) -> Result<FinalizeEnclaveOutput> {
    ServiceEnclave::finalize()?;
    Ok(FinalizeEnclaveOutput::default())
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
