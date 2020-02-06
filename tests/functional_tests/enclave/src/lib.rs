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

use teaclave_types;

use teaclave_ipc::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    RunTestInput, RunTestOutput,
};
use teaclave_ipc::{handle_ecall, register_ecall_handler};
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_test_utils::check_all_passed;
use teaclave_types::TeeServiceResult;

mod teaclave_access_control_service;
mod teaclave_authentication_service;
mod teaclave_execution_service;
mod teaclave_frontend_service;
mod teaclave_storage_service;

#[handle_ecall]
fn handle_run_test(_args: &RunTestInput) -> TeeServiceResult<RunTestOutput> {
    let ret = check_all_passed!(
        teaclave_access_control_service::run_tests(),
        teaclave_authentication_service::run_tests(),
        teaclave_storage_service::run_tests(),
        teaclave_execution_service::run_tests(),
        teaclave_frontend_service::run_tests(),
    );

    assert_eq!(ret, true);
    Ok(RunTestOutput::default())
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
    (ECallCommand::RunTest, RunTestInput, RunTestOutput),
    (ECallCommand::InitEnclave, InitEnclaveInput, InitEnclaveOutput),
    (ECallCommand::FinalizeEnclave, FinalizeEnclaveInput, FinalizeEnclaveOutput),
);
