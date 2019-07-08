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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use mesatee_core::config;
use mesatee_core::prelude::*;
use mesatee_core::Result;

use env_logger;
use std::backtrace::{self, PrintFormat};

use crate::fns::FNSEnclave;

mod executor;
pub use executor::Executor;

register_ecall_handler!(
    type ECallCommand,
    (ECallCommand::ServeConnection, ServeConnectionInput, ServeConnectionOutput),
    (ECallCommand::InitEnclave, InitEnclaveInput, InitEnclaveOutput),
    (ECallCommand::FinalizeEnclave, FinalizeEnclaveInput, FinalizeEnclaveOutput),
);

#[handle_ecall]
fn handle_init_enclave(_args: &InitEnclaveInput) -> Result<InitEnclaveOutput> {
    debug!("Enclave [FNS]: Initializing...");

    env_logger::init();
    let _ = backtrace::enable_backtrace(
        concat!(include_str!("../../../pkg_name"), ".enclave.signed.so"),
        PrintFormat::Full,
    );

    mesatee_core::rpc::sgx::prelude();
    Ok(InitEnclaveOutput::default())
}

#[handle_ecall]
fn handle_finalize_enclave(_args: &FinalizeEnclaveInput) -> Result<FinalizeEnclaveOutput> {
    #[cfg(feature = "cov")]
    sgx_cov::cov_writeout();

    debug!("Enclave [FNS]: Finalized.");
    Ok(FinalizeEnclaveOutput::default())
}

#[handle_ecall]
fn handle_serve_connection(args: &ServeConnectionInput) -> Result<ServeConnectionOutput> {
    debug!("Enclave [FNS]: Serve Connection.");

    let server_instance = FNSEnclave::default();

    let fns_config = config::External::fns();
    assert_eq!(args.port, fns_config.addr.port());

    let enclave_attr = match fns_config.inbound_desc {
        config::InboundDesc::External => None,
        _ => unreachable!(),
    };

    let config = PipeConfig {
        fd: args.socket_fd,
        retry: 0,
        client_attr: enclave_attr,
    };

    let mut server = match Pipe::start(config) {
        Ok(s) => s,
        Err(e) => {
            error!("Start Pipe failed: {}", e);
            return Ok(ServeConnectionOutput::default());
        }
    };

    // We discard all enclave internal errors here.
    let _ = server.serve(server_instance);

    Ok(ServeConnectionOutput::default())
}
