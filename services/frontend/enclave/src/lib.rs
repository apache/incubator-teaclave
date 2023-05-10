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

use teaclave_attestation::verifier;
use teaclave_attestation::{AttestationConfig, RemoteAttestation};
use teaclave_binder::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_binder::{handle_ecall, register_ecall_handler};
use teaclave_config::build::AS_ROOT_CA_CERT;
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_frontend_service::TeaclaveFrontendServer;
use teaclave_rpc::{config::SgxTrustedTlsServerConfig, transport::Server};
use teaclave_service_enclave_utils::{
    create_trusted_authentication_endpoint, create_trusted_management_endpoint, ServiceEnclave,
};
use teaclave_types::{TeeServiceError, TeeServiceResult};

mod error;
mod service;

// Sets the number of worker threads the Runtime will use.
const N_WORKERS: usize = 8;

async fn start_service(config: &RuntimeConfig) -> Result<()> {
    info!("Starting FrontEnd ...");

    let listen_address = config.api_endpoints.frontend.listen_address;
    let attestation_config = AttestationConfig::from_teaclave_config(config)?;
    let attested_tls_config = RemoteAttestation::new(attestation_config)
        .generate_and_endorse()?
        .attested_tls_config()
        .ok_or_else(|| anyhow!("cannot get attested TLS config"))?;

    info!(" Starting FrontEnd: Self attestation finished ...");

    let server_config =
        SgxTrustedTlsServerConfig::from_attested_tls_config(attested_tls_config.clone())?.into();
    info!(" Starting FrontEnd: Server config setup finished ...");

    let enclave_info = teaclave_types::EnclaveInfo::from_bytes(&config.audit.enclave_info_bytes);
    let authentication_service_endpoint = create_trusted_authentication_endpoint(
        &config.internal_endpoints.authentication.advertised_address,
        &enclave_info,
        AS_ROOT_CA_CERT,
        verifier::universal_quote_verifier,
        attested_tls_config.clone(),
    )?;

    info!(" Starting FrontEnd: setup authentication endpoint finished ...");

    let management_service_endpoint = create_trusted_management_endpoint(
        &config.internal_endpoints.management.advertised_address,
        &enclave_info,
        AS_ROOT_CA_CERT,
        verifier::universal_quote_verifier,
        attested_tls_config,
    )?;

    info!(" Starting FrontEnd: setup management endpoint finished ...");

    let service = service::TeaclaveFrontendService::new(
        authentication_service_endpoint,
        management_service_endpoint,
    )
    .await?;

    info!(" Starting FrontEnd: start listening ...");
    Server::builder()
        .tls_config(server_config)
        .map_err(|_| anyhow!("TeaclaveFrontendServer tls config error"))?
        .add_service(TeaclaveFrontendServer::new(service))
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
        run_tests!(
            service::tests::test_authorize_platform_admin,
            service::tests::test_authorize_function_owner,
            service::tests::test_authorize_data_owner,
        )
    }
}
