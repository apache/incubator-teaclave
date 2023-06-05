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
extern crate sgx_types;

use std::cell::RefCell;
use std::thread;
use tokio::sync::mpsc::unbounded_channel;

use anyhow::{anyhow, Result};
use rusty_leveldb::DB;

use teaclave_attestation::{verifier, AttestationConfig, RemoteAttestation};
use teaclave_binder::proto::{
    ECallCommand, FinalizeEnclaveInput, FinalizeEnclaveOutput, InitEnclaveInput, InitEnclaveOutput,
    StartServiceInput, StartServiceOutput,
};
use teaclave_binder::{handle_ecall, register_ecall_handler};
use teaclave_config::build::{AS_ROOT_CA_CERT, AUDITOR_PUBLIC_KEYS, STORAGE_INBOUND_SERVICES};
use teaclave_config::RuntimeConfig;
use teaclave_proto::teaclave_storage_service::TeaclaveStorageServer;
use teaclave_rpc::config::SgxTrustedTlsServerConfig;
use teaclave_service_enclave_utils::ServiceEnclave;
use teaclave_types::{EnclaveInfo, TeeServiceError, TeeServiceResult};

mod error;
mod proxy;
mod service;

async fn start_service(config: &RuntimeConfig) -> Result<()> {
    info!("Starting Storage...");

    let listen_address = config.internal_endpoints.storage.listen_address;
    let attestation_config = AttestationConfig::from_teaclave_config(config)?;
    let attested_tls_config = RemoteAttestation::new(attestation_config)
        .generate_and_endorse()?
        .attested_tls_config()
        .ok_or_else(|| anyhow!("cannot get attested TLS config"))?;
    info!(" Starting Storage: Self attestation finished ...");

    let enclave_info = EnclaveInfo::verify_and_new(
        &config.audit.enclave_info_bytes,
        AUDITOR_PUBLIC_KEYS,
        &config.audit.auditor_signatures_bytes,
    )?;
    let accepted_enclave_attrs: Vec<teaclave_types::EnclaveAttr> = STORAGE_INBOUND_SERVICES
        .iter()
        .map(|service| match enclave_info.get_enclave_attr(service) {
            Some(attr) => Ok(attr),
            None => Err(anyhow!("cannot get enclave attribute of {}", service)),
        })
        .collect::<Result<_>>()?;

    let server_config =
        SgxTrustedTlsServerConfig::from_attested_tls_config(attested_tls_config.clone())?
            .attestation_report_verifier(
                accepted_enclave_attrs,
                AS_ROOT_CA_CERT,
                verifier::universal_quote_verifier,
            )?
            .into();
    info!(" Starting Storage: Server config setup finished ...");

    let (sender, receiver) = unbounded_channel();
    let storage_handle = thread::spawn(move || {
        info!(" Starting Storage: opening database ...");
        #[cfg(test_mode)]
        let db = test_mode::create_mock_db();
        #[cfg(not(test_mode))]
        let db = create_teaclave_db();

        let mut storage_service = service::TeaclaveStorageService::new(RefCell::new(db), receiver);

        info!(" Starting Storage: database loaded ...");
        storage_service.start();
    });

    let service = proxy::ProxyService::new(sender);

    info!(" Starting Storage: start listening ...");

    teaclave_rpc::transport::Server::builder()
        .tls_config(server_config)
        .map_err(|_| anyhow::anyhow!("TeaclaveFrontendServer tls config error"))?
        .add_service(TeaclaveStorageServer::new(service))
        .serve(listen_address)
        .await?;
    storage_handle.join().unwrap();
    Ok(())
}

#[cfg(not(test_mode))]
pub(crate) fn create_teaclave_db() -> DB {
    let opt = rusty_leveldb::in_memory();
    DB::open("teaclave_db", opt).expect("cannot open teaclave_db")
}

#[cfg(test_mode)]
mod test_mode {
    use super::*;
    pub(crate) fn create_mock_db() -> DB {
        let key = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x0f, 0x0e, 0x0d, 0x0c, 0x0b, 0x0a,
            0x09, 0x08,
        ];
        let opt = rusty_leveldb::Options::new_disk_db_with(key);
        let mut database = DB::open("mock_db", opt).unwrap();
        database.put(b"test_get_key", b"test_get_value").unwrap();
        database
            .put(b"test_delete_key", b"test_delete_value")
            .unwrap();
        database
    }
}

#[handle_ecall]
fn handle_start_service(input: &StartServiceInput) -> TeeServiceResult<StartServiceOutput> {
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| TeeServiceError::SgxError)?
        .block_on(start_service(&input.config));

    match result {
        Ok(_) => Ok(StartServiceOutput),
        Err(e) => {
            error!("Failed to run service: {}", e);
            Err(TeeServiceError::ServiceError)
        }
    }
}

#[handle_ecall]
fn handle_init_enclave(_: &InitEnclaveInput) -> TeeServiceResult<InitEnclaveOutput> {
    ServiceEnclave::init(env!("CARGO_PKG_NAME"))?;
    Ok(InitEnclaveOutput)
}

#[handle_ecall]
fn handle_finalize_enclave(_: &FinalizeEnclaveInput) -> TeeServiceResult<FinalizeEnclaveOutput> {
    ServiceEnclave::finalize()?;
    Ok(FinalizeEnclaveOutput)
}

register_ecall_handler!(
    type ECallCommand,
    (ECallCommand::StartService, StartServiceInput, StartServiceOutput),
    (ECallCommand::InitEnclave, InitEnclaveInput, InitEnclaveOutput),
    (ECallCommand::FinalizeEnclave, FinalizeEnclaveInput, FinalizeEnclaveOutput),
);

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_test_utils::*;

    pub fn run_tests() -> bool {
        run_tests!(
            service::tests::test_get_key,
            service::tests::test_put_key,
            service::tests::test_delete_key,
            service::tests::test_empty_value,
            service::tests::test_enqueue,
            service::tests::test_dequeue,
            service::tests::test_get_keys_by_prefix,
        )
    }
}
