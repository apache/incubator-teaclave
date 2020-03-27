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

use std::collections::HashMap;
use std::prelude::v1::*;

use teaclave_binder::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    RunTestInput, RunTestOutput,
};
use teaclave_binder::{handle_ecall, register_ecall_handler};
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_types;
use teaclave_types::TeeServiceResult;

mod access_control_service;
mod authentication_service;
mod end_to_end;
mod execution_service;
mod frontend_service;
mod management_service;
mod scheduler_service;
mod storage_service;
mod utils;

type BoxedFnTest = Box<dyn Fn() -> bool>;

#[handle_ecall]
fn handle_run_test(input: &RunTestInput) -> TeeServiceResult<RunTestOutput> {
    let test_names = &input.test_names;
    let v: Vec<(_, BoxedFnTest)> = vec![
        (
            "access_control_service",
            Box::new(access_control_service::run_tests),
        ),
        (
            "authentication_service",
            Box::new(authentication_service::run_tests),
        ),
        ("storage_service", Box::new(storage_service::run_tests)),
        ("frontend_service", Box::new(frontend_service::run_tests)),
        (
            "management_service",
            Box::new(management_service::run_tests),
        ),
        ("scheduler_service", Box::new(scheduler_service::run_tests)),
        ("execution_service", Box::new(execution_service::run_tests)),
        ("end_to_end", Box::new(end_to_end::run_tests)),
    ];
    let test_map: HashMap<_, BoxedFnTest> =
        v.into_iter().map(|(k, v)| (k.to_string(), v)).collect();

    let mut ret = true;

    if test_names.is_empty() {
        for fn_test in test_map.values() {
            ret &= fn_test();
        }
    }

    for name in test_names.iter() {
        let fn_test = test_map.get(name).unwrap();
        ret &= fn_test();
    }

    assert_eq!(ret, true);
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
