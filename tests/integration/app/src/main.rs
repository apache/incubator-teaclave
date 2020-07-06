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

use log::error;
use teaclave_binder::proto::{ECallCommand, RunTestInput, RunTestOutput};
use teaclave_binder::TeeBinder;
use teaclave_types::TeeServiceResult;

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter_or("TEACLAVE_LOG", "RUST_LOG")
            .write_style_or("TEACLAVE_LOG_STYLE", "RUST_LOG_STYLE"),
    );
    let tee = TeeBinder::new(env!("CARGO_PKG_NAME"))?;
    run(&tee)?;
    tee.finalize();

    Ok(())
}

fn start_enclave_unit_test_driver(tee: &TeeBinder) -> anyhow::Result<()> {
    let cmd = ECallCommand::RunTest;
    let input = RunTestInput::default();
    match tee.invoke::<RunTestInput, TeeServiceResult<RunTestOutput>>(cmd, input) {
        Err(e) => error!("{:?}", e),
        Ok(Err(e)) => error!("{:?}", e),
        _ => (),
    }

    Ok(())
}

fn run(tee: &TeeBinder) -> anyhow::Result<()> {
    start_enclave_unit_test_driver(tee)?;

    Ok(())
}
