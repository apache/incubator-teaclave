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

use std::prelude::v1::*;

use teaclave_access_control_service_enclave;
use teaclave_authentication_service_enclave;
use teaclave_binder::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    RunTestInput, RunTestOutput,
};
use teaclave_binder::{handle_ecall, register_ecall_handler};
use teaclave_execution_service_enclave;
use teaclave_management_service_enclave;
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_test_utils::check_all_passed;
use teaclave_types::{self, TeeServiceResult};
use teaclave_worker;

#[handle_ecall]
fn handle_run_test(_: &RunTestInput) -> TeeServiceResult<RunTestOutput> {
    let ret = check_all_passed!(
        teaclave_management_service_enclave::tests::run_tests(),
        teaclave_storage_service_enclave::tests::run_tests(),
        teaclave_access_control_service_enclave::tests::run_tests(),
        teaclave_execution_service_enclave::tests::run_tests(),
        teaclave_authentication_service_enclave::tests::run_tests(),
        teaclave_worker::tests::run_tests(),
        teaclave_types::tests::run_tests(),
    );

    assert!(ret);

    #[cfg(not(sgx_sim))]
    assert!(teaclave_attestation::tests::run_tests());

    Ok(RunTestOutput)
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
    (ECallCommand::RunTest, RunTestInput, RunTestOutput),
    (ECallCommand::InitEnclave, InitEnclaveInput, InitEnclaveOutput),
    (ECallCommand::FinalizeEnclave, FinalizeEnclaveInput, FinalizeEnclaveOutput),
);
