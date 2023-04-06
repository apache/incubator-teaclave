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

// Depend on occlum(https://github.com/occlum/occlum.git)

use super::super::{PlatformError, Result};
use libc::*;
use log::debug;
use sgx_crypto::ecc::EcPublicKey;
use sgx_rand::{RdRand, Rng};
use sgx_types::types::*;
use std::ffi::CString;

// From occlum/src/libos/src/fs/dev_fs/dev_sgx/consts.rs.
const SGX_CMD_NUM_GEN_EPID_QUOTE: u64 = (2u32
    | ('s' as u32) << 8
    | (std::mem::size_of::<IoctlGenEPIDQuoteArg>() as u32) << 16
    | 3u32 << 30) as u64;
// for dcap
const IOCTL_MAX_RETRIES: u32 = 20;
const SGXIOC_GET_DCAP_QUOTE_SIZE: u64 = 0x80047307;
const SGXIOC_GEN_DCAP_QUOTE: u64 = 0xc0187308;

// From occlum/src/libos/src/fs/dev_fs/dev_sgx/mod.rs
#[repr(C)]
pub struct IoctlGenDCAPQuoteArg {
    pub report_data: *const ReportData, // Input
    pub quote_size: *mut u32,           // Input/output
    pub quote_buf: *mut u8,             // Output
}

// From occlum/src/libos/src/fs/dev_fs/dev_sgx/mod.rs
#[repr(C)]
struct IoctlGenEPIDQuoteArg {
    report_data: ReportData,   // Input
    quote_type: QuoteSignType, // Input
    spid: Spid,                // Input
    nonce: QuoteNonce,         // Input
    sigrl_ptr: *const u8,      // Input (optional)
    sigrl_len: u32,            // Input (optional)
    quote_buf_len: u32,        // Input
    quote_buf: *mut u8,        // Output
}

fn get_dev_fd() -> libc::c_int {
    let path = CString::new("/dev/sgx").unwrap();
    let fd = unsafe { libc::open(path.as_ptr(), O_RDONLY) };
    if fd > 0 {
        fd
    } else {
        panic!("Open /dev/sgx failed")
    }
}

/// Create report data.
pub(crate) fn create_sgx_report_data(pub_k: EcPublicKey) -> ReportData {
    debug!("create_sgx_report_data");
    let mut report_data: ReportData = ReportData::default();
    let mut pub_k_gx = pub_k.public_key().gx;
    pub_k_gx.reverse();
    let mut pub_k_gy = pub_k.public_key().gy;
    pub_k_gy.reverse();
    report_data.d[..32].clone_from_slice(&pub_k_gx);
    report_data.d[32..].clone_from_slice(&pub_k_gy);
    report_data
}

macro_rules! do_ioctl {
    ($cmd:expr,$arg:expr) => {
        let mut retries = 0;
        let mut ret = -1;
        let fd = get_dev_fd();
        log::debug!("begin do_ioctl {}", stringify!($cmd));
        while retries < IOCTL_MAX_RETRIES {
            ret = unsafe { libc::ioctl(fd, $cmd, $arg) };
            if ret == 0 {
                break;
                // EBUSY 16
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
            retries += 1;
        }
        if retries == IOCTL_MAX_RETRIES {
            return Err(PlatformError::Ioctl(stringify!($cmd).to_string(), ret));
        }
    };
}

/// Get quote with attestation key ID and enclave's local report.
pub(crate) fn get_sgx_epid_quote(spid: &Spid, report_data: ReportData) -> Result<Vec<u8>> {
    let sigrl_ptr: *const u8 = std::ptr::null();
    let quote_len: u32 = 4096;
    let mut quote = vec![0u8; quote_len as usize];
    let mut qe_report_info = QeReportInfo::default();
    let mut quote_nonce = QuoteNonce::default();

    let mut rng = RdRand::new().map_err(PlatformError::RngError)?;
    rng.fill_bytes(&mut quote_nonce.rand);
    qe_report_info.nonce = quote_nonce;

    debug!("SGX_CMD_NUM_GEN_EPID_QUOTE");
    let mut quote_arg = IoctlGenEPIDQuoteArg {
        report_data,                         // Input
        quote_type: QuoteSignType::Linkable, // Input
        spid: spid.to_owned(),               // Input
        nonce: quote_nonce,                  // Input
        sigrl_ptr,                           // Input (optional)
        sigrl_len: 0,                        // Input (optional)
        quote_buf_len: quote_len,            // Input
        quote_buf: quote.as_mut_ptr(),       // Output
    };

    // gen quote and check qe_quote and quote nonce
    do_ioctl!(SGX_CMD_NUM_GEN_EPID_QUOTE, &mut quote_arg);
    let sgx_quote = unsafe { &*(quote.as_ptr() as *const Quote) };
    let quote_size = std::mem::size_of::<Quote>() + sgx_quote.signature_len as usize;
    if quote_size > quote.len() {
        return Err(PlatformError::GetQuote("wrong quote size".to_string()));
    }
    let quote_buf = quote[..quote_size].to_vec();
    Ok(quote_buf)
}

/// Get dcap quote
pub(crate) fn get_sgx_dcap_quote(_spid: &Spid, report_data: ReportData) -> Result<Vec<u8>> {
    let mut quote_len: u32 = 0;
    do_ioctl!(SGXIOC_GET_DCAP_QUOTE_SIZE, &mut quote_len);
    let mut quote_buf = vec![0; quote_len as usize];
    let mut quote_arg: IoctlGenDCAPQuoteArg = IoctlGenDCAPQuoteArg {
        report_data: &report_data as _,
        quote_size: &mut quote_len,
        quote_buf: quote_buf.as_mut_ptr(),
    };
    do_ioctl!(SGXIOC_GEN_DCAP_QUOTE, &mut quote_arg);
    Ok(quote_buf)
}
