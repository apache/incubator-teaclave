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
#[macro_use]
extern crate sgx_tstd as std;

use log::debug;
use log::error;
use std::backtrace;
use teaclave_attestation::verifier::AttestationReportVerificationFn;
use teaclave_rpc::config::SgxTrustedTlsClientConfig;
use teaclave_rpc::endpoint::Endpoint;
use teaclave_types::EnclaveInfo;

#[cfg(feature = "cov")]
use sgx_trts::global_dtors_object;
#[cfg(feature = "cov")]
global_dtors_object! {
    SGX_COV_FINALIZE, sgx_cov_exit = {
        debug!("cov_writeout");
        sgx_cov::cov_writeout();
    }
}

pub struct ServiceEnclave;

impl ServiceEnclave {
    pub fn init(name: &str) -> teaclave_types::TeeServiceResult<()> {
        env_logger::init();

        debug!("Enclave initializing");

        if backtrace::enable_backtrace(format!("{}.signed.so", name), backtrace::PrintFormat::Full)
            .is_err()
        {
            error!("Cannot enable backtrace");
            return Err(teaclave_types::TeeServiceError::SgxError);
        }

        Ok(())
    }

    pub fn finalize() -> teaclave_types::TeeServiceResult<()> {
        debug!("Enclave finalizing");

        #[cfg(feature = "cov")]
        sgx_cov::cov_writeout();

        Ok(())
    }
}

pub use teaclave_service_enclave_utils_proc_macro::teaclave_service;

macro_rules! impl_create_trusted_endpoint_fn {
    ($fn_name:ident, $enclave_attr:literal) => {
        pub fn $fn_name(
            advertised_address: &str,
            enclave_info: &EnclaveInfo,
            as_root_ca_cert: &[u8],
            verifier: AttestationReportVerificationFn,
        ) -> Endpoint {
            let service_enclave_attrs = enclave_info
                .get_enclave_attr($enclave_attr)
                .expect("enclave_info");
            let service_client_config = SgxTrustedTlsClientConfig::new()
                .attestation_report_verifier(
                    vec![service_enclave_attrs],
                    as_root_ca_cert,
                    verifier,
                );
            let service_address = &advertised_address;

            Endpoint::new(service_address).config(service_client_config)
        }
    };
}

impl_create_trusted_endpoint_fn!(create_trusted_storage_endpoint, "teaclave_storage_service");
impl_create_trusted_endpoint_fn!(
    create_trusted_scheduler_endpoint,
    "teaclave_scheduler_service"
);
