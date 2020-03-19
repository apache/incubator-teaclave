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

use teaclave_attestation::verifier;
use teaclave_binder::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_binder::{handle_ecall, register_ecall_handler};
use teaclave_config::RuntimeConfig;
use teaclave_config::BUILD_CONFIG;
use teaclave_service_enclave_utils::create_trusted_scheduler_endpoint;
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_types::{EnclaveInfo, TeeServiceError, TeeServiceResult};

mod ocall;
mod service;

const AS_ROOT_CA_CERT: &[u8] = BUILD_CONFIG.as_root_ca_cert;
const AUDITOR_PUBLIC_KEYS_LEN: usize = BUILD_CONFIG.auditor_public_keys.len();
const AUDITOR_PUBLIC_KEYS: &[&[u8]; AUDITOR_PUBLIC_KEYS_LEN] = BUILD_CONFIG.auditor_public_keys;

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
    let scheduler_service_address = &config.internal_endpoints.scheduler.advertised_address;
    let scheduler_service_endpoint = create_trusted_scheduler_endpoint(
        &scheduler_service_address,
        &enclave_info,
        AS_ROOT_CA_CERT,
        verifier::universal_quote_verifier,
    );

    let mut service = service::TeaclaveExecutionService::new(scheduler_service_endpoint).unwrap();
    let _ = service.start();

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
            ocall::tests::test_handle_file_request,
            service::tests::test_invoke_echo_function,
            service::tests::test_invoke_gbdt_training,
            service::tests::test_invoke_gbdt_prediction
        )
    }
}
