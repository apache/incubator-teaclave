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

use mesatee_core::ipc::protos::ecall::{RunFunctionalTestInput, RunFunctionalTestOutput};
use mesatee_core::prelude::*;
use mesatee_core::Result;

mod tests;

#[macro_use]
mod unittest;
use unittest::*;

use teaclave_binder::TeeBinder;
use std::sync::Arc;

fn run_test_in_tee(tee: &TeeBinder) -> Result<()> {
    trace!("Running as Functional Test Client ...");
    let args_info = RunFunctionalTestInput::default();
    let ret_info = tee.invoke::<RunFunctionalTestInput, RunFunctionalTestOutput>(
        ECallCommand::RunFunctionalTest.into(),
        args_info,
    )?;
    assert_eq!(ret_info.failed_count, 0);
    Ok(())
}

fn test_from_unstrusted() {
    unit_tests!(
        tests::tdfs_test::read_not_exist_file,
        tests::tdfs_test::save_and_read,
        tests::tdfs_test::list_file_api,
        tests::tdfs_test::delete_file_api,
        tests::tms_test::api_get_task,
        tests::tms_test::api_create_task,
        tests::tms_test::api_update_task,
        tests::tms_test::api_list_task,
        tests::fns_test::api_invoke_task,
        tests::fns_test::api_invoke_multiparty_task,
    );
}

fn test_in_tee() -> Result<()> {
    let tee = match TeeBinder::new("functional_test", 1) {
        Ok(r) => {
            info!("Init TEE Successfully!");
            r
        }
        Err(x) => {
            error!("Init TEE Failed {}!", x);
            std::process::exit(-1)
        }
    };

    let tee = Arc::new(tee);

    {
        let ref_tee = tee.clone();
        ctrlc::set_handler(move || {
            info!("\nCTRL+C pressed. Destroying server enclave");
            ref_tee.finalize();
            std::process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");
    }

    match run_test_in_tee(&tee) {
        Ok(_) => {
            info!("Run Enclave Exit Successfully!");
            Ok(())
        }
        Err(x) => {
            error!("Run Enclave Launch Failed {}!", x);
            Err(x)
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();

    test_from_unstrusted();
    test_in_tee()?;

    Ok(())
}
