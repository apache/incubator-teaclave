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

use std::ffi::CString;
use std::os::raw::c_char;

use mesatee_core::config;
use mesatee_core::prelude::*;
use mesatee_core::{Error, ErrorKind, Result};

use env_logger;
use std::backtrace::{self, PrintFormat};

use crate::acs::ACSEnclave;

register_ecall_handler!(
    type ECallCommand,
    (ECallCommand::ServeConnection, ServeConnectionInput, ServeConnectionOutput),
    (ECallCommand::InitEnclave, InitEnclaveInput, InitEnclaveOutput),
    (ECallCommand::FinalizeEnclave, FinalizeEnclaveInput, FinalizeEnclaveOutput),
);

extern "C" {
    fn acs_setup_model(model_text: *const c_char) -> i32;
}

#[handle_ecall]
fn handle_serve_connection(args: &ServeConnectionInput) -> Result<ServeConnectionOutput> {
    debug!("Enclave [ACS]: Serve Connection.");

    let server_instance = ACSEnclave::default();
    let acs_config = config::Internal::acs();
    assert_eq!(args.port, acs_config.addr.port());

    let enclave_attr = match acs_config.inbound_desc {
        config::InboundDesc::Sgx(enclave_attr) => enclave_attr,
        _ => unreachable!(),
    };

    let config = PipeConfig {
        fd: args.socket_fd,
        retry: 0,
        client_attr: Some(enclave_attr),
    };

    let mut server = match Pipe::start(config) {
        Ok(s) => s,
        Err(e) => {
            error!("Start Pipe failed: {}", e);
            return Ok(ServeConnectionOutput::default());
        }
    };

    let _ = server.serve(server_instance);

    // We discard all enclave internal errors here.
    Ok(ServeConnectionOutput::default())
}

const MODEL_TEXT: &str = include_str!("../../model.conf");

#[handle_ecall]
fn handle_init_enclave(_args: &InitEnclaveInput) -> Result<InitEnclaveOutput> {
    debug!("Enclave [ACS]: Initializing...");

    env_logger::init();
    let _ = backtrace::enable_backtrace(
        concat!(include_str!("../../pkg_name"), ".enclave.signed.so"),
        PrintFormat::Full,
    );
    mesatee_core::rpc::sgx::prelude();

    eprintln!("setting up acs model");

    let ec = unsafe { acs_setup_model(CString::new(MODEL_TEXT).unwrap().as_ptr()) };

    if ec != 0 {
        Err(Error::from(ErrorKind::MesaPyError))
    } else {
        Ok(InitEnclaveOutput::default())
    }
}

#[handle_ecall]
fn handle_finalize_enclave(_args: &FinalizeEnclaveInput) -> Result<FinalizeEnclaveOutput> {
    #[cfg(feature = "cov")]
    sgx_cov::cov_writeout();

    debug!("Enclave [ACS]: Finalized.");
    Ok(FinalizeEnclaveOutput::default())
}
