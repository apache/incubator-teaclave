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

#[macro_use]
extern crate log;

use std::prelude::v1::*;

use anyhow;
use anyhow::Result;
use teaclave_types;

use teaclave_ipc::protos::ecall::{
    FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    ServeConnectionInput, ServeConnectionOutput, StartServiceInput, StartServiceOutput,
};
use teaclave_ipc::protos::ECallCommand;

use teaclave_ipc::attribute::handle_ecall;
use teaclave_ipc::register_ecall_handler;

use teaclave_service_config as config;
use teaclave_service_sgx_utils;

use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::Arc;

use lazy_static::lazy_static;
use std::os::raw::c_int;
use std::sync::SgxMutex as Mutex;
use teaclave_attestation;
use teaclave_attestation::RemoteAttestation;
use teaclave_frontend_proto::*;
use teaclave_rpc::server::SgxTrustedTlsServer;

mod service;

struct MpscFdQueue {
    sender: mpsc::SyncSender<c_int>,
    receiver: Mutex<mpsc::Receiver<c_int>>,
}

lazy_static! {
    static ref FD_QUEUE: MpscFdQueue = {
        let (sender, receiver) = mpsc::sync_channel::<c_int>(1);
        MpscFdQueue {
            sender,
            receiver: Mutex::new(receiver),
        }
    };
}

#[handle_ecall]
fn handle_start_service(_args: &StartServiceInput) -> Result<StartServiceOutput> {
    debug!("handle_start_service");
    loop {
        let receiver = FD_QUEUE.receiver.lock().unwrap();
        let fd = receiver.recv().unwrap();

        let stream = TcpStream::new(fd).unwrap();
        let attestation = RemoteAttestation::generate_and_endorse(
            &config::runtime_config().env.ias_key,
            &config::runtime_config().env.ias_spid,
        )
        .unwrap();
        let config = teaclave_attestation::rpc::new_tls_server_config_without_verifier(
            attestation.cert,
            attestation.private_key,
        )
        .unwrap();
        let rc_config = Arc::new(config);

        let mut server = SgxTrustedTlsServer::<
            proto::TeaclaveFrontendResponse,
            proto::TeaclaveFrontendRequest,
        >::new_with_config(stream, &rc_config);
        server.start(service::TeaclaveFrontendService)?;
    }
}

#[handle_ecall]
fn handle_serve_connection(args: &ServeConnectionInput) -> Result<ServeConnectionOutput> {
    debug!("handle_serve_connection");

    let sender = &FD_QUEUE.sender;
    sender.send(args.socket_fd).unwrap();

    Ok(ServeConnectionOutput::default())
}

#[handle_ecall]
fn handle_init_enclave(_args: &InitEnclaveInput) -> Result<InitEnclaveOutput> {
    debug!("handle_init_enclave");

    teaclave_service_sgx_utils::init_service(env!("CARGO_PKG_NAME"))?;

    Ok(InitEnclaveOutput::default())
}

#[handle_ecall]
fn handle_finalize_enclave(_args: &FinalizeEnclaveInput) -> Result<FinalizeEnclaveOutput> {
    #[cfg(feature = "cov")]
    sgx_cov::cov_writeout();

    debug!("handle_finalize_enclave");
    Ok(FinalizeEnclaveOutput::default())
}

register_ecall_handler!(
    type ECallCommand,
    (ECallCommand::StartService, StartServiceInput, StartServiceOutput),
    (ECallCommand::ServeConnection, ServeConnectionInput, ServeConnectionOutput),
    (ECallCommand::InitEnclave, InitEnclaveInput, InitEnclaveOutput),
    (ECallCommand::FinalizeEnclave, FinalizeEnclaveInput, FinalizeEnclaveOutput),
);
