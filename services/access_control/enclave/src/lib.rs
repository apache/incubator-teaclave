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
use std::prelude::v1::*;
use teaclave_attestation::{AttestationConfig, RemoteAttestation};
use teaclave_ipc::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_ipc::{handle_ecall, register_ecall_handler};
use teaclave_proto::teaclave_access_control_service::{
    TeaclaveAccessControlRequest, TeaclaveAccessControlResponse,
};
use teaclave_rpc::config::SgxTrustedTlsServerConfig;
use teaclave_rpc::server::SgxTrustedTlsServer;
use teaclave_service_enclave_utils::ServiceEnclave;

mod acs;
mod service;

#[handle_ecall]
fn handle_start_service(args: &StartServiceInput) -> Result<StartServiceOutput> {
    debug!("handle_start_service");
    let listen_address = args.config.internal_endpoints.access_control.listen_address;
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

    acs::init_acs().unwrap();
    let mut server = SgxTrustedTlsServer::<
        TeaclaveAccessControlResponse,
        TeaclaveAccessControlRequest,
    >::new(listen_address, &config);
    let service = service::TeaclaveAccessControlService::new();
    match server.start(service) {
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
    use teaclave_test_utils::*;

    pub fn run_tests() -> bool {
        if crate::acs::init_acs().is_err() {
            return false;
        }
        run_tests!(
            service::tests::user_access_data,
            service::tests::user_access_function,
            service::tests::user_access_task,
            service::tests::task_access_function,
            service::tests::task_access_data,
        )
    }
}
