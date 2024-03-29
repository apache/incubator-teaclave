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
use tokio::sync::Mutex;

use std::sync::Arc;

use teaclave_attestation::verifier;
use teaclave_attestation::{AttestationConfig, RemoteAttestation};
use teaclave_binder::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_binder::{handle_ecall, register_ecall_handler};
use teaclave_config::build::AS_ROOT_CA_CERT;
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_access_control_service::TeaclaveAccessControlClient;
use teaclave_proto::teaclave_authentication_service::TeaclaveAuthenticationInternalClient;
use teaclave_proto::teaclave_frontend_service::TeaclaveFrontendServer;
use teaclave_proto::teaclave_management_service::TeaclaveManagementClient;
use teaclave_rpc::{config::SgxTrustedTlsServerConfig, transport::Server};
use teaclave_service_enclave_utils::{
    create_trusted_access_control_endpoint, create_trusted_authentication_endpoint,
    create_trusted_management_endpoint, ServiceEnclave,
};
use teaclave_types::{TeeServiceError, TeeServiceResult};

mod audit;
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

    let authentication_channel = authentication_service_endpoint
        .connect()
        .await
        .map_err(|e| anyhow!("Failed to connect to authentication service, retry {:?}", e))?;
    let authentication_client = Arc::new(Mutex::new(
        TeaclaveAuthenticationInternalClient::new_with_builtin_config(authentication_channel),
    ));

    info!(" Starting FrontEnd: setup authentication client finished ...");

    let management_service_endpoint = create_trusted_management_endpoint(
        &config.internal_endpoints.management.advertised_address,
        &enclave_info,
        AS_ROOT_CA_CERT,
        verifier::universal_quote_verifier,
        attested_tls_config.clone(),
    )?;

    let management_channel = management_service_endpoint
        .connect()
        .await
        .map_err(|e| anyhow!("Failed to connect to management service, {:?}", e))?;
    let management_client = Arc::new(Mutex::new(
        TeaclaveManagementClient::new_with_builtin_config(management_channel),
    ));

    info!(" Starting FrontEnd: setup management client finished ...");

    let access_control_service_endpoint = create_trusted_access_control_endpoint(
        &config.internal_endpoints.access_control.advertised_address,
        &enclave_info,
        AS_ROOT_CA_CERT,
        verifier::universal_quote_verifier,
        attested_tls_config.clone(),
    )?;

    let access_control_channel = access_control_service_endpoint
        .connect()
        .await
        .map_err(|e| anyhow!("Failed to connect to access_control service, retry {:?}", e))?;
    let access_control_client = Arc::new(Mutex::new(TeaclaveAccessControlClient::new(
        access_control_channel,
    )));

    info!(" Starting FrontEnd: setup access_control client finished ...");

    let log_buffer = Arc::new(Mutex::new(Vec::new()));
    let audit_agent = audit::AuditAgent::new(management_client.clone(), log_buffer.clone());
    let agent_handle = tokio::spawn(async move {
        audit_agent.run().await;
    });

    let service = service::TeaclaveFrontendService::new(
        authentication_client,
        management_client,
        access_control_client,
        log_buffer,
    )
    .await?;

    info!(" Starting FrontEnd: start listening ...");
    Server::builder()
        .tls_config(server_config)
        .map_err(|_| anyhow!("TeaclaveFrontendServer tls config error"))?
        .add_service(TeaclaveFrontendServer::new_with_builtin_config(service))
        .serve(listen_address)
        .await?;

    agent_handle.await?;

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
    pub fn run_tests() -> bool {
        true
    }
}
