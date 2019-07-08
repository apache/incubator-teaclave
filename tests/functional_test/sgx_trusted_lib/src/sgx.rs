// Copyright 2019 MesaTEE Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use mesatee_core::ipc::protos::ecall::{RunFunctionalTestInput, RunFunctionalTestOutput};
use mesatee_core::prelude::*;
use mesatee_core::Result;

use env_logger;
use std::backtrace::{self, PrintFormat};

use crate::tests;
use sgx_tunittest::*;

register_ecall_handler!(
    type ECallCommand,
    (ECallCommand::RunFunctionalTest, RunFunctionalTestInput, RunFunctionalTestOutput),
    (ECallCommand::InitEnclave, InitEnclaveInput, InitEnclaveOutput),
    (ECallCommand::FinalizeEnclave, FinalizeEnclaveInput, FinalizeEnclaveOutput),
);

#[handle_ecall]
fn handle_run_functional_test(_args: &RunFunctionalTestInput) -> Result<RunFunctionalTestOutput> {
    let nfailed = rsgx_unit_tests!(
        tests::kms_test::api_create_key,
        tests::tdfs_test::read_not_exist_file,
        tests::tdfs_test::save_and_read,
        tests::tdfs_test::check_file_permission,
        tests::tdfs_test::task_share_file,
        tests::tdfs_test::global_share_file,
        tests::tms_test::get_task,
        tests::tms_test::update_task_result,
        tests::tms_test::update_private_result,
        tests::tms_test::update_status,
    );

    Ok(RunFunctionalTestOutput::new(nfailed))
}

#[handle_ecall]
fn handle_init_enclave(_args: &InitEnclaveInput) -> Result<InitEnclaveOutput> {
    info!("Enclave [Functional Test]: Initialized.");

    env_logger::init();
    let _ = backtrace::enable_backtrace(
        concat!(include_str!("../../pkg_name"), ".enclave.signed.so"),
        PrintFormat::Full,
    );
    mesatee_core::rpc::sgx::prelude();

    Ok(InitEnclaveOutput::default())
}

#[handle_ecall]
fn handle_finalize_enclave(_args: &FinalizeEnclaveInput) -> Result<FinalizeEnclaveOutput> {
    #[cfg(feature = "cov")]
    sgx_cov::cov_writeout();

    info!("Enclave [Functional Test]: Finalized.");
    Ok(FinalizeEnclaveOutput::default())
}
