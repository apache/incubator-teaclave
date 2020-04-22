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

//! This module provides SGX platform related functions like getting local
//! report and transform into a remotely verifiable quote.

use std::prelude::v1::*;

use anyhow::{ensure, Result};
use log::debug;
use sgx_rand::{os::SgxRng, Rng};
use sgx_tcrypto::rsgx_sha256_slice;
use sgx_tse::{rsgx_create_report, rsgx_verify_report};
use sgx_types::sgx_status_t::SGX_SUCCESS;
use sgx_types::*;

type SgxStatus = sgx_types::sgx_status_t;

#[derive(thiserror::Error, Debug)]
pub enum PlatformError {
    #[error("Failed to call {0}: {1}")]
    OCallError(String, SgxStatus),
    #[error("Failed to initialize quote : {0}")]
    InitQuoteError(SgxStatus),
    #[error("Failed to create the report of the enclave: {0}")]
    CreateReportError(SgxStatus),
    #[error("Failed to get target info of this enclave: {0}")]
    GetSelfTargetInfoError(SgxStatus),
    #[error("Failed to get quote: {0}")]
    GetQuoteError(SgxStatus),
    #[error("Failed to verify quote: {0}")]
    VerifyReportError(SgxStatus),
    #[error(
        "Replay attack on report: quote_nonce.rand {0:?},
        qe_report.body.report_data.d[..32] {1:?}"
    )]
    ReportReplay(Vec<u8>, Vec<u8>),
    #[error("Failed to use SGX rng to generate random number: {0}")]
    SgxRngError(std::io::Error),
    #[error("Other SGX platform error: {0}")]
    Others(SgxStatus),
}

extern "C" {
    /// Ocall to use sgx_init_quote_ex to init the quote and key_id.
    fn ocall_sgx_init_quote(
        p_retval: *mut sgx_status_t,
        p_sgx_att_key_id: *mut sgx_att_key_id_t,
        p_target_info: *mut sgx_target_info_t,
    ) -> sgx_status_t;

    /// Ocall to get the required buffer size for the quote.
    fn ocall_sgx_get_quote_size(
        p_retval: *mut sgx_status_t,
        p_sgx_att_key_id: *const sgx_att_key_id_t,
        p_quote_size: *mut u32,
    ) -> sgx_status_t;

    /// Ocall to use sgx_get_quote_ex to generate a quote with enclave's report.
    fn ocall_sgx_get_quote(
        p_retval: *mut sgx_status_t,
        p_report: *const sgx_report_t,
        p_sgx_att_key_id: *const sgx_att_key_id_t,
        p_qe_report_info: *mut sgx_qe_report_info_t,
        p_quote: *mut u8,
        quote_size: u32,
    ) -> sgx_status_t;

    /// OCall to get target information of myself.
    fn sgx_self_target(p_target_info: *mut sgx_target_info_t) -> sgx_status_t;
}

/// Initialize SGX quote, return attestation key ID selected by the platform and
/// target information for creating report that only QE can verify.
pub(crate) fn init_sgx_quote() -> Result<(sgx_att_key_id_t, sgx_target_info_t)> {
    debug!("init_quote");
    let mut ti = sgx_target_info_t::default();
    let mut ak_id = sgx_att_key_id_t::default();
    let mut rt = sgx_status_t::SGX_ERROR_UNEXPECTED;

    let res = unsafe { ocall_sgx_init_quote(&mut rt as _, &mut ak_id as _, &mut ti as _) };

    ensure!(
        res == SGX_SUCCESS,
        PlatformError::OCallError("ocall_sgx_init_quote".to_string(), res)
    );
    ensure!(rt == SGX_SUCCESS, PlatformError::InitQuoteError(rt));

    Ok((ak_id, ti))
}

/// Create report of the enclave with target_info.
pub(crate) fn create_sgx_isv_enclave_report(
    pub_k: sgx_ec256_public_t,
    target_info: sgx_target_info_t,
) -> Result<sgx_report_t> {
    debug!("create_report");
    let mut report_data: sgx_report_data_t = sgx_report_data_t::default();
    let mut pub_k_gx = pub_k.gx;
    pub_k_gx.reverse();
    let mut pub_k_gy = pub_k.gy;
    pub_k_gy.reverse();
    report_data.d[..32].clone_from_slice(&pub_k_gx);
    report_data.d[32..].clone_from_slice(&pub_k_gy);

    let report =
        rsgx_create_report(&target_info, &report_data).map_err(PlatformError::CreateReportError)?;

    Ok(report)
}

/// Get quote with attestation key ID and enclave's local report.
pub(crate) fn get_sgx_quote(ak_id: &sgx_att_key_id_t, report: sgx_report_t) -> Result<Vec<u8>> {
    let mut rt = sgx_status_t::SGX_ERROR_UNEXPECTED;
    let mut quote_len: u32 = 0;

    let res = unsafe { ocall_sgx_get_quote_size(&mut rt as _, ak_id as _, &mut quote_len as _) };

    ensure!(
        res == SGX_SUCCESS,
        PlatformError::OCallError("ocall_sgx_get_quote_size".to_string(), res)
    );
    ensure!(rt == SGX_SUCCESS, PlatformError::GetQuoteError(rt));

    let mut qe_report_info = sgx_qe_report_info_t::default();
    let mut quote_nonce = sgx_quote_nonce_t::default();

    let mut rng = SgxRng::new().map_err(PlatformError::SgxRngError)?;
    rng.fill_bytes(&mut quote_nonce.rand);
    qe_report_info.nonce = quote_nonce;

    debug!("sgx_self_target");
    // Provide the target information of ourselves so that we can verify the QE report
    // returned with the quote
    let res = unsafe { sgx_self_target(&mut qe_report_info.app_enclave_target_info as _) };

    ensure!(
        res == SGX_SUCCESS,
        PlatformError::GetSelfTargetInfoError(res)
    );

    let mut quote = vec![0; quote_len as usize];

    debug!("ocall_sgx_get_quote");
    let res = unsafe {
        ocall_sgx_get_quote(
            &mut rt as _,
            &report as _,
            ak_id as _,
            &mut qe_report_info as _,
            quote.as_mut_ptr(),
            quote_len,
        )
    };

    ensure!(
        res == SGX_SUCCESS,
        PlatformError::OCallError("ocall_sgx_get_quote".to_string(), res)
    );
    ensure!(rt == SGX_SUCCESS, PlatformError::GetQuoteError(rt));

    debug!("rsgx_verify_report");
    let qe_report = qe_report_info.qe_report;
    // Perform a check on qe_report to verify if the qe_report is valid.
    rsgx_verify_report(&qe_report).map_err(PlatformError::VerifyReportError)?;

    // Check qe_report to defend against replay attack. The purpose of
    // p_qe_report is for the ISV enclave to confirm the QUOTE it received
    // is not modified by the untrusted SW stack, and not a replay. The
    // implementation in QE is to generate a REPORT targeting the ISV
    // enclave (target info from p_report) , with the lower 32Bytes in
    // report.data = SHA256(p_nonce||p_quote). The ISV enclave can verify
    // the p_qe_report and report.data to confirm the QUOTE has not be
    // modified and is not a replay. It is optional.
    let mut rhs_vec: Vec<u8> = quote_nonce.rand.to_vec();
    rhs_vec.extend(&quote);
    debug!("rsgx_sha256_slice");
    let rhs_hash = rsgx_sha256_slice(&rhs_vec).map_err(PlatformError::Others)?;
    let lhs_hash = &qe_report.body.report_data.d[..32];
    ensure!(
        rhs_hash == lhs_hash,
        PlatformError::ReportReplay(rhs_hash.to_vec(), lhs_hash.to_vec())
    );

    Ok(quote)
}

#[cfg(all(feature = "enclave_unit_test", feature = "mesalock_sgx"))]
pub mod tests {
    use super::*;
    use crate::key;
    use teaclave_test_utils::*;

    pub fn run_tests() -> bool {
        run_tests!(
            test_init_sgx_quote,
            test_create_sgx_isv_enclave_report,
            test_get_sgx_quote,
        )
    }

    fn test_init_sgx_quote() {
        assert!(init_sgx_quote().is_ok());
    }

    fn test_create_sgx_isv_enclave_report() {
        let (_ak_id, qe_target_info) = init_sgx_quote().unwrap();
        let key_pair = key::NistP256KeyPair::new().unwrap();
        let sgx_report_result = create_sgx_isv_enclave_report(key_pair.pub_k(), qe_target_info);
        assert!(sgx_report_result.is_ok());
    }

    fn test_get_sgx_quote() {
        let (ak_id, qe_target_info) = init_sgx_quote().unwrap();
        let key_pair = key::NistP256KeyPair::new().unwrap();
        let sgx_report = create_sgx_isv_enclave_report(key_pair.pub_k(), qe_target_info).unwrap();
        let quote_result = get_sgx_quote(&ak_id, sgx_report);
        assert!(quote_result.is_ok());
    }
}
