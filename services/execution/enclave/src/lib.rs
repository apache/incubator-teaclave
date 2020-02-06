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
extern crate sgx_tstd as std;

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

#[macro_use]
extern crate log;

mod service;

use teaclave_ipc::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_ipc::{handle_ecall, register_ecall_handler};

use teaclave_service_enclave_utils::ServiceEnclave;

use teaclave_attestation::{AttestationConfig, RemoteAttestation};
use teaclave_proto::teaclave_execution_service::{
    TeaclaveExecutionRequest, TeaclaveExecutionResponse,
};
use teaclave_rpc::config::SgxTrustedTlsServerConfig;
use teaclave_rpc::server::SgxTrustedTlsServer;
use teaclave_types::{TeeServiceError, TeeServiceResult};

fn start_service(args: &StartServiceInput) -> anyhow::Result<()> {
    let listen_address = args.config.internal_endpoints.execution.listen_address;
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
        SgxTrustedTlsServer::<TeaclaveExecutionResponse, TeaclaveExecutionRequest>::new(
            listen_address,
            &config,
        );
    match server.start(service::TeaclaveExecutionService::new()) {
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
    info!("Worker started");
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

    pub fn run_tests() -> bool {
        true
    }
}
