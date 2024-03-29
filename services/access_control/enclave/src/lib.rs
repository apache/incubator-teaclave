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

use teaclave_attestation::{verifier, AttestationConfig, RemoteAttestation};
use teaclave_binder::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_binder::{handle_ecall, register_ecall_handler};
use teaclave_config::build::{
    ACCESS_CONTROL_INBOUND_SERVICES, AS_ROOT_CA_CERT, AUDITOR_PUBLIC_KEYS,
};
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_access_control_service::TeaclaveAccessControlServer;
use teaclave_rpc::{config::SgxTrustedTlsServerConfig, transport::Server};
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_types::{EnclaveInfo, TeeServiceError, TeeServiceResult};

mod acs;
mod error;
mod service;

// Sets the number of worker threads the Runtime will use.
const N_WORKERS: usize = 8;

async fn start_service(config: &RuntimeConfig) -> Result<()> {
    info!("Starting Access control...");

    let listen_address = config.internal_endpoints.access_control.listen_address;
    let attestation_config = AttestationConfig::from_teaclave_config(config)?;
    let attested_tls_config = RemoteAttestation::new(attestation_config)
        .generate_and_endorse()?
        .attested_tls_config()
        .ok_or_else(|| anyhow!("cannot get attested TLS config"))?;
    info!(" Starting Access control: Self attestation finished ...");

    let enclave_info = EnclaveInfo::verify_and_new(
        &config.audit.enclave_info_bytes,
        AUDITOR_PUBLIC_KEYS,
        &config.audit.auditor_signatures_bytes,
    )?;
    let accepted_enclave_attrs: Vec<teaclave_types::EnclaveAttr> = ACCESS_CONTROL_INBOUND_SERVICES
        .iter()
        .map(|service| match enclave_info.get_enclave_attr(service) {
            Some(attr) => Ok(attr),
            None => Err(anyhow!("cannot get enclave attribute of {}", service)),
        })
        .collect::<Result<_>>()?;

    let server_config = SgxTrustedTlsServerConfig::from_attested_tls_config(attested_tls_config)?
        .attestation_report_verifier(
            accepted_enclave_attrs,
            AS_ROOT_CA_CERT,
            verifier::universal_quote_verifier,
        )?
        .into();
    info!(" Starting Access control: Server config setup finished ...");

    let service = service::TeaclaveAccessControlService::new().await;

    info!("Starting Access control: start listening ...");
    Server::builder()
        .tls_config(server_config)
        .map_err(|_| anyhow::anyhow!("TeaclaveFrontendServer tls config error"))?
        .add_service(TeaclaveAccessControlServer::new_with_builtin_config(
            service,
        ))
        .serve(listen_address)
        .await?;
    Ok(())
}

#[handle_ecall]
fn handle_start_service(input: &StartServiceInput) -> TeeServiceResult<StartServiceOutput> {
    let result = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(N_WORKERS)
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
        run_async_tests!(acs::tests::test_access_api,)
    }
}
