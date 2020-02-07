use crate::AttestationError;
use anyhow::bail;
use anyhow::Error;
use anyhow::Result;
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
        p_sgx_att_key_id: *mut sgx_att_key_id_t,
        p_target_info: *mut sgx_target_info_t,
    ) -> sgx_status_t;

    fn ocall_sgx_get_quote_size(
        p_retval: *mut sgx_status_t,
        p_sgx_att_key_id: *const sgx_att_key_id_t,
        p_quote_size: *mut u32,
    ) -> sgx_status_t;

    fn ocall_sgx_get_quote(
        p_retval: *mut sgx_status_t,
        p_report: *const sgx_report_t,
        p_sgx_att_key_id: *const sgx_att_key_id_t,
        p_qe_report_info: *mut sgx_qe_report_info_t,
        p_quote: *mut u8,
        quote_size: u32,
    ) -> sgx_status_t;

    fn ocall_sgx_get_remote_socket(
        p_retval: *mut i32,
        p_url: *const u8,
        len: usize,
    ) -> sgx_status_t;

    fn sgx_self_target(p_target_info: *mut sgx_target_info_t) -> sgx_status_t;
}

pub(crate) fn init_sgx_quote() -> Result<(sgx_att_key_id_t, sgx_target_info_t)> {
    debug!("init_quote");
    let mut ti = sgx_target_info_t::default();
    let mut ak_id = sgx_att_key_id_t::default();
    let mut rt = sgx_status_t::SGX_ERROR_UNEXPECTED;

    let res = unsafe { ocall_sgx_init_quote(&mut rt as _, &mut ak_id as _, &mut ti as _) };

    if res != sgx_status_t::SGX_SUCCESS {
        bail!(AttestationError::OCallError)
    }

    if rt != sgx_status_t::SGX_SUCCESS {
        bail!(AttestationError::PlatformError)
    }

    Ok((ak_id, ti))
}

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

    Ok(rsgx_create_report(&target_info, &report_data)
        .map_err(|_| Error::new(AttestationError::PlatformError))
        .unwrap())
}

pub(crate) fn get_sgx_quote(ak_id: &sgx_att_key_id_t, report: sgx_report_t) -> Result<Vec<u8>> {
    let mut rt: sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
    let mut quote_len: u32 = 0;

    let res = unsafe { ocall_sgx_get_quote_size(&mut rt as _, ak_id as _, &mut quote_len as _) };

    if res != sgx_status_t::SGX_SUCCESS {
        bail!(AttestationError::OCallError);
    }

    if rt != sgx_status_t::SGX_SUCCESS {
        bail!(AttestationError::PlatformError);
    }

    let mut qe_report_info = sgx_qe_report_info_t::default();
    let mut quote_nonce = sgx_quote_nonce_t::default();

    let mut rng = SgxRng::new()?;
    rng.fill_bytes(&mut quote_nonce.rand);
    qe_report_info.nonce = quote_nonce;

    debug!("sgx_self_target");
    // Provide the target information of ourselves so that we can verify the QE report
    // returned with the quote
    let res = unsafe { sgx_self_target(&mut qe_report_info.app_enclave_target_info as _) };

    if res != sgx_status_t::SGX_SUCCESS {
        bail!(AttestationError::PlatformError);
    }

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

    if res != sgx_status_t::SGX_SUCCESS || rt != sgx_status_t::SGX_SUCCESS {
        bail!(AttestationError::OCallError);
    }

    debug!("rsgx_verify_report");
    let qe_report = qe_report_info.qe_report;
    // Perform a check on qe_report to verify if the qe_report is valid.
    rsgx_verify_report(&qe_report).map_err(|_| Error::new(AttestationError::PlatformError))?;

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
    let rhs_hash = rsgx_sha256_slice(&rhs_vec)
        .map_err(|_| Error::new(AttestationError::PlatformError))
        .expect("sha256 failed");
    let lhs_hash = &qe_report.body.report_data.d[..32];
    if rhs_hash != lhs_hash {
        bail!(AttestationError::PlatformError);
    }

    Ok(quote)
}

pub(crate) fn get_attestation_service_socket(url: &str) -> Result<c_int> {
    debug!("get_attestation_service_socket");
    let mut fd: c_int = -1;
    let res =
        unsafe { ocall_sgx_get_remote_socket(&mut fd as _, url.as_bytes().as_ptr(), url.len()) };

    if res != sgx_status_t::SGX_SUCCESS || fd < 0 {
        bail!(AttestationError::OCallError)
    } else {
        Ok(fd)
    }
}
