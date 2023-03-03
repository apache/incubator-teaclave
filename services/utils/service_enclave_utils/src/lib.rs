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

#![feature(concat_idents)]

#[cfg(feature = "mesalock_sgx")]
extern crate sgx_trts;

use anyhow::Result;
use log::debug;
use log::error;
use std::backtrace;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
#[cfg(feature = "mesalock_sgx")]
use std::untrusted::path::PathEx;
use teaclave_attestation::verifier::AttestationReportVerificationFn;
use teaclave_attestation::AttestedTlsConfig;
use teaclave_config::RuntimeConfig;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::{EnclaveInfo, TeeServiceError, TeeServiceResult};

mod macros;

#[cfg(feature = "cov")]
#[sgx_macros::global_dtor]
fn cov_exit() {
    println!("sgx_cov finished!");
    sgx_cov::cov_writeout();
}

extern "C" {
    pub static g_peak_heap_used: isize;
    pub static g_peak_rsrv_mem_committed: isize;
}

pub struct ServiceEnclave;

impl ServiceEnclave {
    pub fn init(_name: &str) -> TeeServiceResult<()> {
        let env = env_logger::Env::new()
            .filter_or("TEACLAVE_LOG", "RUST_LOG")
            .write_style_or("TEACLAVE_LOG_STYLE", "RUST_LOG_STYLE");
        let env_logger = env_logger::Builder::from_env(env).build();
        teaclave_logger::Builder::new()
            .secondary_logger(env_logger)
            .init();

        debug!("Enclave initializing");

        if backtrace::enable_backtrace(backtrace::PrintFormat::Full).is_err() {
            error!("Cannot enable backtrace");
            return Err(TeeServiceError::SgxError);
        }

        Ok(())
    }

    pub fn finalize() -> TeeServiceResult<()> {
        debug!("Enclave finalizing");
        unsafe {
            debug!("g_peak_heap_used: {}", g_peak_heap_used);
            debug!("g_peak_rsrv_mem_committed: {}", g_peak_rsrv_mem_committed);
        }

        #[cfg(feature = "cov")]
        sgx_cov::cov_writeout();

        Ok(())
    }
}

pub fn base_dir_for_db(config: &RuntimeConfig) -> Result<PathBuf> {
    base_dir(config, "database")
}

pub fn base_dir_for_offload_functions(config: &RuntimeConfig) -> Result<PathBuf> {
    base_dir(config, "functions")
}

fn base_dir(config: &RuntimeConfig, sub_name: &str) -> Result<PathBuf> {
    let fusion_base = config.mount.fusion_base_dir.as_path();
    // We only create this base directory in test_mode
    // This directory should be mounted in release mode
    #[cfg(test_mode)]
    std::untrusted::fs::create_dir_all(fusion_base)?;
    if !fusion_base.exists() {
        error!(
            "Fusion base directory is not mounted: {}",
            fusion_base.display()
        );
        anyhow::bail!("fusion_base not mounted");
    }

    let sub_base = fusion_base.join(sub_name);
    std::untrusted::fs::create_dir_all(&sub_base)?;

    if !sub_base.exists() {
        error!(
            "Offload base directory is not mounted: {}",
            sub_base.display()
        );
        anyhow::bail!("sub_base not mounted");
    }
    Ok(sub_base)
}

pub use teaclave_service_enclave_utils_proc_macro::teaclave_service;

macro_rules! impl_create_trusted_endpoint_fn {
    ($fn_name:ident, $enclave_attr:literal) => {
        pub fn $fn_name(
            advertised_address: &str,
            enclave_info: &EnclaveInfo,
            as_root_ca_cert: &[u8],
            verifier: AttestationReportVerificationFn,
            attested_tls_config: Arc<RwLock<AttestedTlsConfig>>,
        ) -> Result<Endpoint> {
            let service_enclave_attrs = enclave_info
                .get_enclave_attr($enclave_attr)
                .expect("enclave_info");
            let service_client_config =
                SgxTrustedTlsClientConfig::from_attested_tls_config(attested_tls_config)?
                    .attestation_report_verifier(
                        vec![service_enclave_attrs],
                        as_root_ca_cert,
                        verifier,
                    );
            let service_address = &advertised_address;

            Ok(Endpoint::new(service_address).config(service_client_config))
        }
    };
}

impl_create_trusted_endpoint_fn!(create_trusted_storage_endpoint, "teaclave_storage_service");
impl_create_trusted_endpoint_fn!(
    create_trusted_authentication_endpoint,
    "teaclave_authentication_service"
);
impl_create_trusted_endpoint_fn!(
    create_trusted_management_endpoint,
    "teaclave_management_service"
);
impl_create_trusted_endpoint_fn!(
    create_trusted_scheduler_endpoint,
    "teaclave_scheduler_service"
);
