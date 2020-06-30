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

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

#[macro_use]
extern crate log;
use anyhow::{anyhow, Result};

use teaclave_attestation::{verifier, AttestationConfig, RemoteAttestation};
use teaclave_binder::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_binder::{handle_ecall, register_ecall_handler};
use teaclave_config::build::{AS_ROOT_CA_CERT, AUDITOR_PUBLIC_KEYS, SCHEDULER_INBOUND_SERVICES};
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_scheduler_service::{
    TeaclaveSchedulerRequest, TeaclaveSchedulerResponse,
};
use teaclave_rpc::config::SgxTrustedTlsServerConfig;
use teaclave_rpc::server::SgxTrustedTlsServer;
use teaclave_service_enclave_utils::create_trusted_storage_endpoint;
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_types::{EnclaveInfo, TeeServiceError, TeeServiceResult};

mod error;
mod publisher;
mod service;

fn start_service(config: &RuntimeConfig) -> Result<()> {
    let listen_address = config.internal_endpoints.scheduler.listen_address;
    let attestation_config = AttestationConfig::from_teaclave_config(&config)?;
    let attested_tls_config = RemoteAttestation::new(attestation_config)
        .generate_and_endorse()?
        .attested_tls_config()
        .ok_or_else(|| anyhow!("cannot get attested TLS config"))?;
    let enclave_info = EnclaveInfo::verify_and_new(
        &config.audit.enclave_info_bytes,
        AUDITOR_PUBLIC_KEYS,
        &config.audit.auditor_signatures_bytes,
    )?;
    let accepted_enclave_attrs: Vec<teaclave_types::EnclaveAttr> = SCHEDULER_INBOUND_SERVICES
        .iter()
        .map(|service| match enclave_info.get_enclave_attr(service) {
            Some(attr) => Ok(attr),
            None => Err(anyhow!("cannot get enclave attribute of {}", service)),
        })
        .collect::<Result<_>>()?;
    let server_config =
        SgxTrustedTlsServerConfig::from_attested_tls_config(attested_tls_config.clone())?
            .attestation_report_verifier(
                accepted_enclave_attrs,
                AS_ROOT_CA_CERT,
                verifier::universal_quote_verifier,
            )?;

    let mut server =
        SgxTrustedTlsServer::<TeaclaveSchedulerResponse, TeaclaveSchedulerRequest>::new(
            listen_address,
            server_config,
        );

    let storage_service_address = &config.internal_endpoints.storage.advertised_address;
    let storage_service_endpoint = create_trusted_storage_endpoint(
        &storage_service_address,
        &enclave_info,
        AS_ROOT_CA_CERT,
        verifier::universal_quote_verifier,
        attested_tls_config,
    )?;

    let service = service::TeaclaveSchedulerService::new(storage_service_endpoint)?;
    match server.start(service) {
        Ok(_) => (),
        Err(e) => {
            error!("Service exit, error: {}.", e);
        }
    }
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

    pub fn run_tests() -> bool {
        true
    }
}
