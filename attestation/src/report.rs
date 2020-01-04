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

use crate::ias::IasClient;
use crate::AttestationError;
use anyhow::Error;
use anyhow::Result;
use hex;
use log::debug;
use sgx_rand::os::SgxRng;
use sgx_rand::Rng;
use sgx_tcrypto::rsgx_sha256_slice;
use sgx_tse::{rsgx_create_report, rsgx_verify_report};
use sgx_types::sgx_ec256_public_t;
use sgx_types::*;
use std::prelude::v1::*;

extern "C" {
    fn ocall_sgx_init_quote(
        p_retval: *mut sgx_status_t,
        p_target_info: *mut sgx_target_info_t,
        p_gid: *mut sgx_epid_group_id_t,
    ) -> sgx_status_t;

    fn ocall_sgx_calc_quote_size(
        p_retval: *mut sgx_status_t,
        p_sig_rl: *const u8,
        sig_rl_size: u32,
        p_quote_size: *mut u32,
    ) -> sgx_status_t;

    fn ocall_sgx_get_quote(
        p_retval: *mut sgx_status_t,
        p_report: *const sgx_report_t,
        quote_type: sgx_quote_sign_type_t,
        p_spid: *const sgx_spid_t,
        p_nonce: *const sgx_quote_nonce_t,
        p_sig_rl: *const u8,
        sig_rl_size: u32,
        p_qe_report: *mut sgx_report_t,
        p_quote: *mut u8,
        quote_size: u32,
    ) -> sgx_status_t;
}

#[derive(Default)]
pub struct IasReport {
    pub report: String,
    pub signature: String,
    pub signing_cert: String,
}

impl IasReport {
    pub fn new(
        pub_k: sgx_ec256_public_t,
        ias_key: &str,
        ias_spid: &str,
        production: bool,
    ) -> Result<Self> {
        let (target_info, epid_group_id) = Self::init_quote()?;
        let mut ias_client = IasClient::new(ias_key, production);
        let sigrl = ias_client.get_sigrl(u32::from_le_bytes(epid_group_id))?;
        let report = Self::create_report(pub_k, target_info)?;
        let quote = Self::get_quote(&sigrl, report, target_info, ias_spid)?;
        let report = ias_client.get_report(&quote)?;
        Ok(report)
    }

    fn init_quote() -> Result<(sgx_target_info_t, sgx_epid_group_id_t)> {
        debug!("init_quote");
        let mut ti: sgx_target_info_t = sgx_target_info_t::default();
        let mut eg: sgx_epid_group_id_t = sgx_epid_group_id_t::default();
        let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;

        let res = unsafe { ocall_sgx_init_quote(&mut rt as _, &mut ti as _, &mut eg as _) };

        if res != sgx_status_t::SGX_SUCCESS || rt != sgx_status_t::SGX_SUCCESS {
            Err(Error::new(AttestationError::OCallError))
        } else {
            Ok((ti, eg))
        }
    }

    fn create_report(
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

        rsgx_create_report(&target_info, &report_data)
            .map_err(|_| Error::new(AttestationError::IasError))
    }

    fn get_quote(
        sigrl: &[u8],
        report: sgx_report_t,
        target_info: sgx_target_info_t,
        ias_spid_str: &str,
    ) -> Result<Vec<u8>> {
        let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
        let (p_sigrl, sigrl_len) = if sigrl.is_empty() {
            (std::ptr::null(), 0)
        } else {
            (sigrl.as_ptr(), sigrl.len() as u32)
        };
        let mut quote_len: u32 = 0;

        let res = unsafe {
            ocall_sgx_calc_quote_size(&mut rt as _, p_sigrl, sigrl_len, &mut quote_len as _)
        };

        if res != sgx_status_t::SGX_SUCCESS || rt != sgx_status_t::SGX_SUCCESS {
            return Err(Error::new(AttestationError::OCallError));
        }

        let mut quote_nonce = sgx_quote_nonce_t { rand: [0; 16] };
        let mut os_rng = SgxRng::new()?;
        os_rng.fill_bytes(&mut quote_nonce.rand);
        let mut qe_report = sgx_report_t::default();

        let quote_type = sgx_quote_sign_type_t::SGX_LINKABLE_SIGNATURE;

        let mut spid = sgx_types::sgx_spid_t::default();
        let hex = hex::decode(ias_spid_str)?;
        spid.id.copy_from_slice(&hex[..16]);

        let mut quote = vec![0; quote_len as usize];

        debug!("ocall_sgx_get_quote");
        let res = unsafe {
            ocall_sgx_get_quote(
                &mut rt as _,
                &report as _,
                quote_type,
                &spid as _,
                &quote_nonce as _,
                p_sigrl,
                sigrl_len,
                &mut qe_report as _,
                quote.as_mut_ptr(),
                quote_len,
            )
        };

        if res != sgx_status_t::SGX_SUCCESS || rt != sgx_status_t::SGX_SUCCESS {
            return Err(Error::new(AttestationError::OCallError));
        }

        debug!("rsgx_verify_report");
        // Perform a check on qe_report to verify if the qe_report is valid.
        rsgx_verify_report(&qe_report).map_err(|_| Error::new(AttestationError::IasError))?;

        // Check if the qe_report is produced on the same platform.
        if target_info.mr_enclave.m != qe_report.body.mr_enclave.m
            || target_info.attributes.flags != qe_report.body.attributes.flags
            || target_info.attributes.xfrm != qe_report.body.attributes.xfrm
        {
            return Err(Error::new(AttestationError::QuoteError));
        }

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
        let rhs_hash =
            rsgx_sha256_slice(&rhs_vec).map_err(|_| Error::new(AttestationError::IasError))?;
        let lhs_hash = &qe_report.body.report_data.d[..32];
        if rhs_hash != lhs_hash {
            return Err(Error::new(AttestationError::QuoteError));
        }

        Ok(quote)
    }
}
