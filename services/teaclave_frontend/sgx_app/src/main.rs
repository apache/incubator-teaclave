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

// use mesatee_core::prelude::*;
// use mesatee_core::{config, Result};
pub use teaclave_ipc::protos::ecall::{
    ServeConnectionInput, ServeConnectionOutput, StartServiceInput, StartServiceOutput,
};
pub use teaclave_ipc::protos::ECallCommand;

use anyhow;
use std::net::TcpListener;
use std::os::unix::io::IntoRawFd;
use threadpool::ThreadPool;

use std::sync::Arc;
use teaclave_binder::TeeBinder;
use teaclave_service_app_utils;
use teaclave_service_config;

fn main() -> anyhow::Result<()> {
    let tee = teaclave_service_app_utils::init_tee(env!("CARGO_PKG_NAME"))?;
    run(tee)?;

    Ok(())
}

fn start_enclave_service(tee: Arc<TeeBinder>) {
    std::thread::spawn(move || {
        let cmd = ECallCommand::StartService;
        let _ = tee.invoke::<StartServiceInput, StartServiceOutput>(cmd.into(), StartServiceInput);
    });
}

fn handle_connection(tee: Arc<TeeBinder>) -> anyhow::Result<()> {
    let config = teaclave_service_config::External::teaclave_frontend();
    info!("Get config...");
    let listener = TcpListener::bind(config.addr)?;
    let port = config.addr.port();

    let n_workers = 10;
    let pool = ThreadPool::new(n_workers);
    info!("Listener incoming ...");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let tee = tee.clone();
                pool.execute(move || {
                    debug!("new worker from {:?}", stream.peer_addr());
                    let fd = stream.into_raw_fd();
                    let input = ServeConnectionInput::new(fd, port);
                    let cmd = ECallCommand::ServeConnection;
                    let _ = tee
                        .invoke::<ServeConnectionInput, ServeConnectionOutput>(cmd.into(), input);
                });
            }
            Err(e) => warn!("couldn't get client: {:?}", e),
        }
    }

    Ok(())
}

fn run(tee: Arc<TeeBinder>) -> anyhow::Result<()> {
    info!("Running {}...", env!("CARGO_PKG_NAME"));

    start_enclave_service(tee.clone());
    handle_connection(tee)?;

    Ok(())
}
