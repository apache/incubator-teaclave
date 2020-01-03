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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use mesatee_core::config;
use mesatee_core::prelude::*;
use mesatee_core::rpc::server::SgxTrustedServer;
use mesatee_core::Result;

use crate::kms::KMSEnclave;

register_ecall_handler!(
    type ECallCommand,
    (ECallCommand::ServeConnection, ServeConnectionInput, ServeConnectionOutput),
    (ECallCommand::InitEnclave, InitEnclaveInput, InitEnclaveOutput),
    (ECallCommand::FinalizeEnclave, FinalizeEnclaveInput, FinalizeEnclaveOutput),
);

#[handle_ecall]
fn handle_serve_connection(args: &ServeConnectionInput) -> Result<ServeConnectionOutput> {
    debug!("Enclave [KMS]: Serve Connection.");

    let kms_config = config::Internal::kms();
    assert_eq!(args.port, kms_config.addr.port());

    let enclave_attr = match kms_config.inbound_desc {
        config::InboundDesc::Sgx(enclave_attr) => Some(enclave_attr),
        _ => unreachable!(),
    };
    let server = match SgxTrustedServer::new(KMSEnclave::default(), args.socket_fd, enclave_attr) {
        Ok(s) => s,
        Err(e) => {
            error!("New server failed: {:?}.", e);
            return Ok(ServeConnectionOutput::default());
        }
    };
    let _ = server.start();

    // We discard all enclave internal errors here.
    Ok(ServeConnectionOutput::default())
}

#[handle_ecall]
fn handle_init_enclave(_args: &InitEnclaveInput) -> Result<InitEnclaveOutput> {
    mesatee_core::init_service(env!("CARGO_PKG_NAME"))?;

    Ok(InitEnclaveOutput::default())
}

#[handle_ecall]
fn handle_finalize_enclave(_args: &FinalizeEnclaveInput) -> Result<FinalizeEnclaveOutput> {
    #[cfg(feature = "cov")]
    sgx_cov::cov_writeout();

    debug!("Enclave [KMS]: Finalized.");
    Ok(FinalizeEnclaveOutput::default())
}
