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

use teaclave_ipc::proto::{ECallCommand, StartServiceInput, StartServiceOutput};

use anyhow::Result;
use std::net::TcpListener;
use std::os::unix::io::IntoRawFd;

use teaclave_binder::TeeBinder;

fn main() -> Result<()> {
    env_logger::init();
    let tee = TeeBinder::new(env!("CARGO_PKG_NAME"), 1)?;
    run(&tee)?;

    Ok(())
}

fn start_enclave_service(tee: &TeeBinder) -> Result<()> {
    info!("Start enclave service");
    let config = teaclave_config::RuntimeConfig::from_toml("runtime.config.toml")?;
    let api_listen_address = &config.api_endpoints.authentication.listen_address;
    let internal_listen_address = &config.internal_endpoints.authentication.listen_address;
    let api_listener = TcpListener::bind(api_listen_address)?;
    let internal_listener = TcpListener::bind(internal_listen_address)?;
    let fds = vec![api_listener.into_raw_fd(), internal_listener.into_raw_fd()];

    let input = StartServiceInput { fds, config };
    let cmd = ECallCommand::StartService;
    let _ = tee.invoke::<StartServiceInput, StartServiceOutput>(cmd.into(), input);

    Ok(())
}

fn run(tee: &TeeBinder) -> Result<()> {
    start_enclave_service(tee)?;

    Ok(())
}
