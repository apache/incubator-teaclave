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

#![feature(strict_provenance)]

extern crate sgx_types;
use anyhow::{anyhow, ensure, Result};
use log::info;
use teaclave_attestation::{verifier, AttestationConfig, RemoteAttestation};
use teaclave_config::build::{AS_ROOT_CA_CERT, AUDITOR_PUBLIC_KEYS};
use teaclave_config::RuntimeConfig;
use teaclave_service_enclave_utils::create_trusted_scheduler_endpoint;
use teaclave_types::EnclaveInfo;

#[cfg(feature = "mesalock_sgx")]
use std::untrusted::path::PathEx;

#[cfg(feature = "mesalock_sgx")]
mod ecall;
mod file_handler;
mod service;
mod task_file_manager;

pub async fn start_service(config: &RuntimeConfig) -> Result<()> {
    info!("Starting Execution...");

    let attestation_config = AttestationConfig::from_teaclave_config(config)?;
    let attested_tls_config = RemoteAttestation::new(attestation_config)
        .generate_and_endorse()?
        .attested_tls_config()
        .ok_or_else(|| anyhow!("cannot get attested TLS config"))?;
    info!(" Starting Execution: Self attestation finished ...");

    let enclave_info = EnclaveInfo::verify_and_new(
        &config.audit.enclave_info_bytes,
        AUDITOR_PUBLIC_KEYS,
        &config.audit.auditor_signatures_bytes,
    )?;
    let scheduler_service_address = &config.internal_endpoints.scheduler.advertised_address;
    let scheduler_service_endpoint = create_trusted_scheduler_endpoint(
        scheduler_service_address,
        &enclave_info,
        AS_ROOT_CA_CERT,
        verifier::universal_quote_verifier,
        attested_tls_config,
    )?;

    let fusion_base = config.mount.fusion_base_dir.clone();

    // We only create this base directory in test_mode
    // This directory should be mounted in release mode
    #[cfg(all(test_mode, feature = "mesalock_sgx"))]
    std::untrusted::fs::create_dir_all(&fusion_base)?;
    #[cfg(all(test_mode, not(feature = "mesalock_sgx")))]
    std::fs::create_dir_all(&fusion_base)?;

    ensure!(
        fusion_base.exists(),
        "Fusion base directory is not mounted: {}",
        fusion_base.display()
    );

    info!(" Starting Execution: start ...");
    let mut service =
        service::TeaclaveExecutionService::new(scheduler_service_endpoint, fusion_base).await?;

    service.start().await
}

#[cfg(feature = "enclave_unit_test")]
pub mod tests {
    use super::*;
    use teaclave_test_utils::*;

    pub fn run_tests() -> bool {
        run_tests!(
            file_handler::tests::test_handle_file_request,
            service::tests::test_invoke_echo,
            service::tests::test_invoke_gbdt_train,
            task_file_manager::tests::test_input,
        )
    }
}
