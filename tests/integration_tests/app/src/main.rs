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

use anyhow;
use teaclave_binder::TeeBinder;
use teaclave_ipc::proto::{ECallCommand, RunTestInput, RunTestOutput};

mod teaclave_config_tests;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let tee = TeeBinder::new(env!("CARGO_PKG_NAME"), 1)?;
    run_app_integration_tests();
    start_enclave_integration_tests(&tee)?;

    Ok(())
}

fn start_enclave_integration_tests(tee: &TeeBinder) -> anyhow::Result<()> {
    let cmd = ECallCommand::RunTest;
    tee.invoke::<RunTestInput, RunTestOutput>(cmd.into(), RunTestInput)?;

    Ok(())
}

fn run_app_integration_tests() {
    let ret = teaclave_config_tests::run_tests();
    assert_eq!(ret, true);
}
