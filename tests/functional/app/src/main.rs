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
use log::error;
use structopt::StructOpt;
use teaclave_binder::proto::{ECallCommand, RunTestInput, RunTestOutput};
use teaclave_binder::TeeBinder;
use teaclave_types::TeeServiceResult;

#[derive(Debug, StructOpt)]
struct Cli {
    /// Names of tests to execute.
    #[structopt(short = "t", required = false)]
    test_names: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::from_args();
    env_logger::init();
    let tee = TeeBinder::new(env!("CARGO_PKG_NAME"))?;
    start_enclave_unit_test_driver(&tee, args.test_names)?;
    tee.finalize();

    Ok(())
}

fn start_enclave_unit_test_driver(tee: &TeeBinder, test_names: Vec<String>) -> anyhow::Result<()> {
    let cmd = ECallCommand::RunTest;
    let input = RunTestInput::new(test_names);
    match tee.invoke::<RunTestInput, TeeServiceResult<RunTestOutput>>(cmd, input) {
        Err(e) => error!("{:?}", e),
        Ok(Err(e)) => error!("{:?}", e),
        _ => (),
    }

    Ok(())
}
