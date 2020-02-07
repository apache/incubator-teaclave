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

use std::prelude::v1::*;
use teaclave_attestation::verifier;
use teaclave_attestation::{AttestationConfig, RemoteAttestation};
use teaclave_binder::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_binder::{handle_ecall, register_ecall_handler};
use teaclave_config::BUILD_CONFIG;
use teaclave_proto::teaclave_management_service::{
    TeaclaveManagementRequest, TeaclaveManagementResponse,
};
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_rpc::config::SgxTrustedTlsServerConfig;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_rpc::server::SgxTrustedTlsServer;
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_types::{TeeServiceError, TeeServiceResult};
mod file;
mod service;

fn start_service(args: &StartServiceInput) -> anyhow::Result<()> {
    let listen_address = args.config.internal_endpoints.management.listen_address;
    let ias_config = args.config.ias.as_ref().unwrap();
    let attestation = RemoteAttestation::generate_and_endorse(&AttestationConfig::ias(
        &ias_config.ias_key,
        &ias_config.ias_spid,
    ))
    .unwrap();
    let config = SgxTrustedTlsServerConfig::new_without_verifier(
        &attestation.cert,
        &attestation.private_key,
    )
    .unwrap();

    let mut server =
        SgxTrustedTlsServer::<TeaclaveManagementResponse, TeaclaveManagementRequest>::new(
            listen_address,
            &config,
        );

    let enclave_info = teaclave_types::EnclaveInfo::from_bytes(
        &args.config.audit.enclave_info_bytes.as_ref().unwrap(),
    );
    let enclave_attr = enclave_info
        .get_enclave_attr("teaclave_storage_service")
        .expect("storage");
    let config = SgxTrustedTlsClientConfig::new()
        .client_cert(&attestation.cert, &attestation.private_key)
        .attestation_report_verifier(
            vec![enclave_attr],
            BUILD_CONFIG.ias_root_ca_cert,
            verifier::universal_quote_verifier,
        );
    let storage_service_address = &args.config.internal_endpoints.storage.advertised_address;

    let storage_service_endpoint = Endpoint::new(storage_service_address).config(config);

    let service = service::TeaclaveManagementService::new(storage_service_endpoint)?;
    match server.start(service) {
        Ok(_) => (),
        Err(e) => {
            error!("Service exit, error: {}.", e);
        }
    }
    Ok(())
}

#[handle_ecall]
fn handle_start_service(args: &StartServiceInput) -> TeeServiceResult<StartServiceOutput> {
    start_service(args).map_err(|_| TeeServiceError::ServiceError)?;
    Ok(StartServiceOutput::default())
}

#[handle_ecall]
fn handle_init_enclave(_args: &InitEnclaveInput) -> TeeServiceResult<InitEnclaveOutput> {
    ServiceEnclave::init(env!("CARGO_PKG_NAME"))?;
    Ok(InitEnclaveOutput::default())
}

#[handle_ecall]
fn handle_finalize_enclave(
    _args: &FinalizeEnclaveInput,
) -> TeeServiceResult<FinalizeEnclaveOutput> {
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
            service::tests::handle_input_file,
            service::tests::handle_output_file,
        )
    }
}
