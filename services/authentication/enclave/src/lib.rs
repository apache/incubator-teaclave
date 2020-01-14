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

#[macro_use]
extern crate log;

use std::prelude::v1::*;

use anyhow::Result;

use teaclave_ipc::protos::ecall::{
    FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_ipc::protos::ECallCommand;

use teaclave_ipc::{handle_ecall, register_ecall_handler};

use teaclave_service_config as config;
use teaclave_service_enclave_utils::ServiceEnclave;

use teaclave_attestation::RemoteAttestation;
use teaclave_proto::teaclave_authentication_service::{
    TeaclaveAuthenticationRequest, TeaclaveAuthenticationResponse,
};
use teaclave_rpc::config::SgxTrustedTlsServerConfig;
use teaclave_rpc::server::SgxTrustedTlsServer;

mod service;

#[handle_ecall]
fn handle_start_service(args: &StartServiceInput) -> Result<StartServiceOutput> {
    debug!("handle_start_service");
    let listener = std::net::TcpListener::new(args.fd)?;
    let attestation = RemoteAttestation::generate_and_endorse(
        &config::runtime_config().env.ias_key,
        &config::runtime_config().env.ias_spid,
    )
    .unwrap();
    let config = SgxTrustedTlsServerConfig::new_without_verifier(
        &attestation.cert,
        &attestation.private_key,
    )
    .unwrap();

    let mut server = SgxTrustedTlsServer::<
        TeaclaveAuthenticationResponse,
        TeaclaveAuthenticationRequest,
    >::new(listener, &config);
    match server.start(service::TeaclaveAuthenticationService) {
        Ok(_) => (),
        Err(e) => {
            error!("Service exit, error: {}.", e);
        }
    }

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
    use sgx_tunittest::*;

    pub fn run_tests() -> usize {
        rsgx_unit_tests!(
            service::tests::test_user_login,
            service::tests::test_user_authorize,
        )
    }
}
