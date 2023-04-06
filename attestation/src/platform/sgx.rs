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
#![cfg(feature = "mesalock_sgx")]

use super::Result;
use log::debug;
use sgx_crypto::ecc::EcPublicKey;
use sgx_crypto::sha::Sha256;
use sgx_rand::{RdRand, Rng};
use sgx_tse::{EnclaveReport, EnclaveTarget};
use sgx_types::error::SgxStatus;
use sgx_types::error::SgxStatus::Success;
use sgx_types::types::*;
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
        p_retval: *mut SgxStatus,
        p_sgx_att_key_id: *mut AttKeyId,
        p_target_info: *mut TargetInfo,
    ) -> SgxStatus;

    /// Ocall to get the required buffer size for the quote.
    fn ocall_sgx_get_quote_size(
        p_retval: *mut SgxStatus,
        p_sgx_att_key_id: *const AttKeyId,
        p_quote_size: *mut u32,
    ) -> SgxStatus;

    /// Ocall to use sgx_get_quote_ex to generate a quote with enclave's report.
    fn ocall_sgx_get_quote(
        p_retval: *mut SgxStatus,
        p_report: *const Report,
        p_sgx_att_key_id: *const AttKeyId,
        p_qe_report_info: *mut QeReportInfo,
        p_quote: *mut u8,
        quote_size: u32,
    ) -> SgxStatus;
}

/// Initialize SGX quote, return attestation key ID selected by the platform and
/// target information for creating report that only QE can verify.
pub(crate) fn init_sgx_quote() -> Result<(AttKeyId, TargetInfo)> {
    debug!("init_quote");
    let mut ti = TargetInfo::default();
    let mut ak_id = AttKeyId::default();
    let mut rt = SgxStatus::Unexpected;

    let res = unsafe { ocall_sgx_init_quote(&mut rt as _, &mut ak_id as _, &mut ti as _) };

    if res != Success {
        return Err(PlatformError::OCallError(
            "ocall_sgx_init_quote".to_string(),
            res,
        ));
    }
    if rt != Success {
        return Err(PlatformError::InitQuoteError(rt));
    }

    Ok((ak_id, ti))
}

/// Create report of the enclave with target_info.
pub(crate) fn create_sgx_isv_enclave_report(
    pub_k: EcPublicKey,
    target_info: TargetInfo,
) -> Result<Report> {
    debug!("create_report");
    let mut report_data = ReportData::default();
    let public_key = pub_k.public_key();
    let mut pub_k_gx = public_key.gx;
    pub_k_gx.reverse();
    let mut pub_k_gy = public_key.gy;
    pub_k_gy.reverse();
    report_data.d[..32].clone_from_slice(&pub_k_gx);
    report_data.d[32..].clone_from_slice(&pub_k_gy);

    let report =
        Report::for_target(&target_info, &report_data).map_err(PlatformError::CreateReportError)?;

    Ok(report)
}

/// Get quote with attestation key ID and enclave's local report.
pub(crate) fn get_sgx_quote(ak_id: &AttKeyId, report: Report) -> Result<Vec<u8>> {
    let mut rt = SgxStatus::Unexpected;
    let mut quote_len: u32 = 0;

    let res = unsafe { ocall_sgx_get_quote_size(&mut rt as _, ak_id as _, &mut quote_len as _) };

    if res != Success {
        return Err(PlatformError::OCallError(
            "ocall_sgx_get_quote_size".to_string(),
            res,
        ));
    }
    if rt != Success {
        return Err(PlatformError::GetQuoteError(rt));
    }

    let mut qe_report_info = QeReportInfo::default();
    let mut quote_nonce = QuoteNonce::default();

    let mut rng = RdRand::new().map_err(PlatformError::SgxRngError)?;
    rng.fill_bytes(&mut quote_nonce.rand);
    qe_report_info.nonce = quote_nonce;

    debug!("sgx self target");
    // Provide the target information of ourselves so that we can verify the QE report
    // returned with the quote
    let tmp_target_info = TargetInfo::for_self().map_err(PlatformError::GetSelfTargetInfoError)?;

    qe_report_info.app_enclave_target_info = tmp_target_info;

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

    if res != Success {
        return Err(PlatformError::OCallError(
            "ocall_sgx_get_quote".to_string(),
            res,
        ));
    }
    if rt != Success {
        return Err(PlatformError::GetQuoteError(rt));
    }

    debug!("sgx verify report");
    let qe_report = qe_report_info.qe_report;
    // Perform a check on qe_report to verify if the qe_report is valid.
    qe_report
        .verify()
        .map_err(PlatformError::VerifyReportError)?;

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
    debug!("sgx sha256 slice");
    let rhs_hash = Sha256::digest(rhs_vec.as_slice()).map_err(PlatformError::Others)?;
    let lhs_hash = &qe_report.body.report_data.d[..32];
    if rhs_hash.as_ref() != lhs_hash {
        return Err(PlatformError::ReportReplay(
            rhs_hash.to_vec(),
            lhs_hash.to_vec(),
        ));
    }

    Ok(quote)
}

#[cfg(all(feature = "enclave_unit_test", feature = "mesalock_sgx"))]
pub mod tests {
    use super::*;
    use crate::key;

    pub fn test_init_sgx_quote() {
        assert!(init_sgx_quote().is_ok());
    }

    pub fn test_create_sgx_isv_enclave_report() {
        let (_ak_id, qe_target_info) = init_sgx_quote().unwrap();
        let key_pair = key::NistP256KeyPair::new().unwrap();
        let sgx_report_result = create_sgx_isv_enclave_report(key_pair.pub_k(), qe_target_info);
        assert!(sgx_report_result.is_ok());
    }

    pub fn test_get_sgx_quote() {
        let (ak_id, qe_target_info) = init_sgx_quote().unwrap();
        let key_pair = key::NistP256KeyPair::new().unwrap();
        let sgx_report = create_sgx_isv_enclave_report(key_pair.pub_k(), qe_target_info).unwrap();
        let quote_result = get_sgx_quote(&ak_id, sgx_report);
        assert!(quote_result.is_ok());
    }
}
